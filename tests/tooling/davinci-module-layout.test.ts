import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const davinciRoots = [
  "crates/vize_carton",
  "crates/vize_davinci",
  "crates/vize_disegno",
  "crates/vize_ricalco",
  "crates/vize_sinopia",
  "benchmarks/davinci_harness",
  "tests/davinci_test_support",
];

function skipBlockComment(source: string, start: number): number {
  let depth = 1;
  let cursor = start + 2;
  while (cursor < source.length && depth > 0) {
    if (source.startsWith("/*", cursor)) {
      depth += 1;
      cursor += 2;
    } else if (source.startsWith("*/", cursor)) {
      depth -= 1;
      cursor += 2;
    } else {
      cursor += 1;
    }
  }
  return cursor;
}

function skipTrivia(source: string, start: number): number {
  let cursor = start;
  while (cursor < source.length) {
    if (/\s/u.test(source[cursor] ?? "")) {
      cursor += 1;
    } else if (source.startsWith("//", cursor)) {
      const newline = source.indexOf("\n", cursor + 2);
      cursor = newline === -1 ? source.length : newline + 1;
    } else if (source.startsWith("/*", cursor)) {
      cursor = skipBlockComment(source, cursor);
    } else {
      break;
    }
  }
  return cursor;
}

function skipQuoted(source: string, quote: number): number {
  let cursor = quote + 1;
  while (cursor < source.length) {
    if (source[cursor] === "\\") {
      cursor += 2;
    } else if (source[cursor] === source[quote]) {
      return cursor + 1;
    } else {
      cursor += 1;
    }
  }
  return source.length;
}

function rawStringEnd(source: string, start: number): number | undefined {
  let cursor = start;
  if (source.startsWith("br", cursor) || source.startsWith("cr", cursor)) {
    cursor += 2;
  } else if (source[cursor] === "r") {
    cursor += 1;
  } else {
    return undefined;
  }
  const hashesStart = cursor;
  while (source[cursor] === "#") cursor += 1;
  if (source[cursor] !== '"') return undefined;
  const suffix = '"' + "#".repeat(cursor - hashesStart);
  const closing = source.indexOf(suffix, cursor + 1);
  return closing === -1 ? source.length : closing + suffix.length;
}

function charLiteralEnd(source: string, quote: number): number | undefined {
  const first = source[quote + 1];
  if (first === undefined || first === "\n" || first === "\r") return undefined;
  if (first === "\\") return skipQuoted(source, quote);
  const codePoint = source.codePointAt(quote + 1);
  if (codePoint === undefined) return undefined;
  const closing = quote + 1 + (codePoint > 0xffff ? 2 : 1);
  return source[closing] === "'" ? closing + 1 : undefined;
}

function pathAttributeAt(source: string, hash: number): boolean {
  let cursor = skipTrivia(source, hash + 1);
  if (source[cursor] !== "[") return false;
  cursor = skipTrivia(source, cursor + 1);
  if (!source.startsWith("path", cursor)) return false;
  cursor += "path".length;
  if (/[_\p{ID_Continue}]/u.test(source[cursor] ?? "")) return false;
  cursor = skipTrivia(source, cursor);
  return source[cursor] === "=";
}

function hasPathAttribute(source: string): boolean {
  let cursor = 0;
  while (cursor < source.length) {
    const triviaEnd = skipTrivia(source, cursor);
    if (triviaEnd !== cursor) {
      cursor = triviaEnd;
      continue;
    }
    const rawEnd = rawStringEnd(source, cursor);
    if (rawEnd !== undefined) {
      cursor = rawEnd;
      continue;
    }
    const prefixLength = source[cursor] === "b" || source[cursor] === "c" ? 1 : 0;
    const quote = cursor + prefixLength;
    if (source[quote] === '"') {
      cursor = skipQuoted(source, quote);
      continue;
    }
    if (source[quote] === "'") {
      const end = charLiteralEnd(source, quote);
      if (end !== undefined) {
        cursor = end;
        continue;
      }
    }
    if (source[cursor] === "#" && pathAttributeAt(source, cursor)) return true;
    cursor += 1;
  }
  return false;
}

function rustFiles(root: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const absolute = path.join(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...rustFiles(absolute));
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      files.push(absolute);
    }
  }
  return files;
}

test("Davinci-owned Rust modules use ordinary module discovery", () => {
  const violations = davinciRoots.flatMap((root) =>
    rustFiles(path.join(repoRoot, root))
      .filter((file) => hasPathAttribute(fs.readFileSync(file, "utf8")))
      .map((file) => path.relative(repoRoot, file)),
  );
  assert.deepEqual(
    violations,
    [],
    `replace path-attributed modules with ordinary mod declarations:\n${violations.join("\n")}`,
  );
});

test("the Davinci module-layout gate recognizes a path attribute", () => {
  assert.equal(hasPathAttribute('  #[path = "nested/file.rs"]\nmod file;'), true);
  assert.equal(
    hasPathAttribute('  #[path /* ordinary discovery only */ = "nested/file.rs"]\nmod file;'),
    true,
  );
  assert.equal(hasPathAttribute('// #[path = "comment.rs"]\nmod file;'), false);
  assert.equal(hasPathAttribute('const EXAMPLE: &str = "#[path = \\"string.rs\\"]";'), false);
  assert.equal(hasPathAttribute('const RAW: &str = r#"#[path = \\"raw.rs\\"]"#;'), false);
});
