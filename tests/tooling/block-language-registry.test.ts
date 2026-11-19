// The block-language registry (`levels/vize_l1/src/container/lang.rs`) is
// the one place a `<template lang>` / `<style lang>` value or a markup file
// extension is named. A product that spells a language itself drifts: before
// the registry, the compiler matched `pug`/`jade` case-insensitively, the
// maestro outline case-sensitively, patina trimmed the value, and two
// products guessed pug from a whole-file substring scan. This gate rejects a
// new language-name comparison in non-test Rust source anywhere else.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";

const REGISTRY = "levels/vize_l1/src/container/lang.rs";

/** Every spelling the registry resolves. */
const SPELLINGS = [
  "html",
  "htm",
  "pug",
  "jade",
  "css",
  "postcss",
  "scss",
  "sass",
  "less",
  "stylus",
  "styl",
];
/** Spellings that are never an HTML tag or a common word in Rust source. */
const DISTINCT = ["pug", "jade", "postcss", "scss", "sass", "stylus", "styl"];

const any = SPELLINGS.join("|");
const distinct = DISTINCT.join("|");

/** One forbidden shape and why. */
interface Shape {
  readonly name: string;
  readonly pattern: RegExp;
}

const SHAPES: readonly Shape[] = [
  {
    name: "case-insensitive comparison with a language name",
    pattern: new RegExp(
      String.raw`(?:\b\w*(?:lang|language|ext|extension)\w*\)?\s*\.eq_ignore_ascii_case\(\s*b?"(?:${any})"|\.eq_ignore_ascii_case\(\s*b?"(?:${distinct})"\s*\))`,
      "i",
    ),
  },
  {
    name: "equality with a language name",
    pattern: new RegExp(
      String.raw`\b\w*(?:lang|language)\w*\b[^;\n]{0,60}?(?:==|!=)\s*(?:Some\()?"(?:${any})"`,
      "i",
    ),
  },
  {
    name: "pattern match on a language name",
    pattern: new RegExp(
      String.raw`(?:matches!\([^;\n]*\b\w*(?:lang|language)\w*\b[^;\n]*"(?:${any})"|Some\(\s*"(?:${distinct})"|"(?:${distinct})"\s*\||\|\s*"(?:${distinct})")`,
      "i",
    ),
  },
  {
    name: "attribute-tuple or guarded match on a language name",
    pattern: new RegExp(
      String.raw`(?:"lang"\s*,\s*Some\(\s*"(?:${any})"|Some\(\s*"(?:${any})"\s*\)\s*if\b)`,
      "i",
    ),
  },
  {
    name: "list of language names",
    pattern: new RegExp(String.raw`"(?:${distinct})"\s*,\s*"(?:${distinct})"`, "i"),
  },
  {
    name: "byte-window language heuristic",
    pattern: new RegExp(String.raw`b"(?:${distinct})"`, "i"),
  },
];

function isTestPath(relative: string): boolean {
  const parts = relative.split("/");
  const file = parts.at(-1) ?? "";
  return (
    parts.some((part) => part === "tests" || part === "benches" || part === "examples") ||
    file === "tests.rs" ||
    file === "test.rs" ||
    /_tests?\.rs$/.test(file) ||
    /^tests?_/.test(file)
  );
}

/**
 * Per line of a Rust source: the brace depth change outside strings, raw
 * strings, chars and comments, with the lexical state carried across lines
 * (a block comment or string may span several), and whether the line opens
 * inside a comment or string.
 */
export function lexicalLines(source: string): Array<{ delta: number; inside: boolean }> {
  const lines: Array<{ delta: number; inside: boolean }> = [];
  let delta = 0;
  let inside = false;
  // normal | line comment | block comment (depth) | string | raw string (hashes)
  let state: "code" | "line" | "block" | "string" | "raw" = "code";
  let blockDepth = 0;
  let rawHashes = 0;
  for (let i = 0; i < source.length; i += 1) {
    const char = source[i]!;
    const next = source[i + 1];
    if (char === "\n") {
      lines.push({ delta, inside });
      delta = 0;
      if (state === "line") state = "code";
      inside = state !== "code";
      continue;
    }
    if (state === "line") continue;
    if (state === "block") {
      if (char === "/" && next === "*") {
        blockDepth += 1;
        i += 1;
      } else if (char === "*" && next === "/") {
        blockDepth -= 1;
        i += 1;
        if (blockDepth === 0) state = "code";
      }
      continue;
    }
    if (state === "string") {
      if (char === "\\") i += 1;
      else if (char === '"') state = "code";
      continue;
    }
    if (state === "raw") {
      if (char === '"' && source.slice(i + 1, i + 1 + rawHashes) === "#".repeat(rawHashes)) {
        i += rawHashes;
        state = "code";
      }
      continue;
    }
    if (char === "/" && next === "/") {
      state = "line";
      i += 1;
    } else if (char === "/" && next === "*") {
      state = "block";
      blockDepth = 1;
      i += 1;
    } else if (char === '"') {
      state = "string";
    } else if (
      char === "r" &&
      /^r#*"/.test(source.slice(i, i + 64)) &&
      !/\w/.test(source[i - 1] ?? "")
    ) {
      rawHashes = /^r(#*)"/.exec(source.slice(i, i + 64))![1]!.length;
      i += rawHashes + 1;
      state = "raw";
    } else if (char === "'") {
      // A char literal (`'{'`, `'\''`, `'\u{7b}'`); a lifetime has no close.
      const literal = /^'(?:\\(?:u\{[0-9a-fA-F]+\}|.)|[^\\'])'/.exec(source.slice(i, i + 16));
      if (literal) i += literal[0].length - 1;
    } else if (char === "{") {
      delta += 1;
    } else if (char === "}") {
      delta -= 1;
    }
  }
  lines.push({ delta, inside });
  return lines;
}

/**
 * The lines of `source` that ship: comments and `#[cfg(test)]` modules
 * dropped, each with its 1-based line number.
 */
export function shippedLines(source: string): Array<[number, string]> {
  const lines = source.split("\n");
  const lexical = lexicalLines(source);
  const kept: Array<[number, string]> = [];
  let testDepth = 0;
  let pendingTest = false;
  for (const [index, raw] of lines.entries()) {
    const line = raw.trim();
    const { delta, inside } = lexical[index] ?? { delta: 0, inside: false };
    if (testDepth > 0) {
      testDepth += delta;
      continue;
    }
    if (inside) continue;
    if (/^#\[cfg\((?:all\()?test\b/.test(line)) {
      pendingTest = true;
      continue;
    }
    if (pendingTest) {
      if (line.startsWith("#[")) continue;
      pendingTest = false;
      if (/^(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{/.test(line)) {
        testDepth = delta;
        continue;
      }
    }
    if (line.startsWith("//") || line.startsWith("/*")) continue;
    kept.push([index + 1, raw]);
  }
  return kept;
}

/**
 * Every forbidden language check in one Rust source, as `line: shape`. A
 * check rustfmt wrapped over several lines is matched on the shipped lines
 * joined with single spaces, and reported at its first line.
 */
export function violations(source: string): string[] {
  const shipped = shippedLines(source);
  const window = 4;
  const joined = (from: number, count: number) =>
    shipped
      .slice(from, from + count)
      .map(([, piece]) => piece.trim())
      .join(" ");
  // The fewest lines from `from` that complete a match, or 0 for none.
  const reach = (pattern: RegExp, from: number) => {
    for (let count = 1; count <= window && from + count <= shipped.length; count += 1) {
      if (pattern.test(joined(from, count))) return count;
    }
    return 0;
  };
  const found: string[] = [];
  for (const [index, [line, text]] of shipped.entries()) {
    for (const shape of SHAPES) {
      const count = reach(shape.pattern, index);
      // A match the next line completes alone belongs to that line.
      if (count === 0 || (count > 1 && reach(shape.pattern, index + 1) === count - 1)) continue;
      found.push(`${line}: ${shape.name}: ${text.trim()}`);
    }
  }
  return found;
}

function rustSources(dir: string, out: string[]): void {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === "target" || entry.name === "node_modules" || entry.name.startsWith(".")) {
      continue;
    }
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) rustSources(full, out);
    else if (entry.name.endsWith(".rs")) out.push(full);
  }
}

void test("the gate recognizes every ad-hoc language check it replaced", () => {
  for (const removed of [
    `lang.eq_ignore_ascii_case("pug") || lang.eq_ignore_ascii_case("jade")`,
    `Some(lang) if lang.eq_ignore_ascii_case("jade") => "jade",`,
    `Some("pug" | "jade") => {`,
    `.any(|window| window.eq_ignore_ascii_case(b"pug"))`,
    `|| template.lang.as_deref().is_some_and(|lang| lang != "html")`,
    `&& template.lang.as_deref().is_none_or(|lang| lang == "html")`,
    `None | Some("css" | "scss" | "less" | "postcss") => Syntax::Braced,`,
    `matches!(lang, Some("scss" | "sass" | "less" | "stylus" | "styl"))`,
    `if lang == "css" {`,
    `.any(|attr| attr.name == "lang" && attr.value != Some("css"))`,
    `if matches!(value, "pug" | "jade") {`,
    `const PREPROCESSOR_LANGS: [&str; 5] = ["scss", "sass", "less", "stylus", "styl"];`,
    `("scoped", None) | ("lang", Some("css"))`,
    `Some("html") if template => {}`,
    `let pug = lang.eq_ignore_ascii_case(\n    "pug"\n);`,
  ]) {
    assert.equal(violations(removed).length > 0, true, removed);
  }
});

void test("a brace in a test module's string does not hide the shipped code after it", () => {
  const source = [
    "#[cfg(test)]",
    "mod tests {",
    '    const OPEN: &str = "{";',
    "    const BRACE: char = '{';",
    "    /* { */",
    '    const MULTI: &str = "{',
    '    }{";',
    '    const RAW: &str = r#"{"#;',
    "}",
    "fn shipped(lang: &str) -> bool {",
    '    lang == "pug"',
    "}",
  ].join("\n");
  assert.deepEqual(violations(source), ['11: equality with a language name: lang == "pug"']);
});

void test("the gate leaves tag names, test modules and comments alone", () => {
  const accepted = [
    `if tag.eq_ignore_ascii_case("html") || tag.eq_ignore_ascii_case("body") {`,
    `// lang.eq_ignore_ascii_case("pug") in a comment`,
    `let names = ["css", "scss"];`,
    `#[cfg(test)]\nmod tests {\n    for language in ["pug", "jade"] { assert!(lang == "pug"); }\n}`,
  ].join("\n");
  assert.deepEqual(violations(accepted), []);
});

void test("only the block-language registry names a template or style language", () => {
  const files: string[] = [];
  for (const root of ["crates", "levels"]) rustSources(path.join(repoRoot, root), files);
  const found: string[] = [];
  for (const file of files) {
    const relative = path.relative(repoRoot, file).split(path.sep).join("/");
    if (relative === REGISTRY || isTestPath(relative)) continue;
    for (const violation of violations(fs.readFileSync(file, "utf8"))) {
      found.push(`${relative}:${violation}`);
    }
  }
  assert.deepEqual(
    found,
    [],
    "resolve the lang through vize_l1::container::lang instead of spelling it",
  );
});
