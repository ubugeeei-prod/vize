// The block-language registry (`davinci/vize_l1/src/container/lang.rs`) is
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

const REGISTRY = "davinci/vize_l1/src/container/lang.rs";

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
 * The lines of `source` that ship: comments and `#[cfg(test)]` modules
 * dropped, each with its 1-based line number.
 */
export function shippedLines(source: string): Array<[number, string]> {
  const lines = source.split("\n");
  const kept: Array<[number, string]> = [];
  let testDepth = 0;
  let pendingTest = false;
  for (const [index, raw] of lines.entries()) {
    const line = raw.trim();
    if (testDepth > 0) {
      for (const char of raw) {
        if (char === "{") testDepth += 1;
        if (char === "}") testDepth -= 1;
      }
      continue;
    }
    if (/^#\[cfg\((?:all\()?test\b/.test(line)) {
      pendingTest = true;
      continue;
    }
    if (pendingTest) {
      if (line.startsWith("#[")) continue;
      pendingTest = false;
      if (/^(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{/.test(line)) {
        testDepth = 0;
        for (const char of raw) {
          if (char === "{") testDepth += 1;
          if (char === "}") testDepth -= 1;
        }
        continue;
      }
    }
    if (line.startsWith("//")) continue;
    kept.push([index + 1, raw]);
  }
  return kept;
}

/** Every forbidden language check in one Rust source, as `line: shape`. */
export function violations(source: string): string[] {
  const found: string[] = [];
  for (const [line, text] of shippedLines(source)) {
    for (const shape of SHAPES) {
      if (shape.pattern.test(text)) found.push(`${line}: ${shape.name}: ${text.trim()}`);
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
  ]) {
    assert.equal(violations(removed).length > 0, true, removed);
  }
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
  for (const root of ["crates", "davinci"]) rustSources(path.join(repoRoot, root), files);
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
