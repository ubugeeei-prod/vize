import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const ordinaryModuleRoots = [
  "crates/vize_atelier_core",
  "crates/vize_atelier_sfc",
  "crates/vize_atelier_vapor",
  "crates/vize_carton",
  "davinci/vize_l0",
  "crates/vize_croquis",
  "davinci/vize_l3",
  "davinci/vize_l2",
  "crates/vize_patina",
  "davinci/vize_l1_to_l2",
  "davinci/vize_l1",
  "tools/benchmarks/crates/davinci_harness",
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

function identifierEndAt(source: string, cursor: number, identifier: string): number | undefined {
  if (/[_\p{ID_Continue}]/u.test(source[cursor - 1] ?? "")) return undefined;
  const identifierStart = source.startsWith("r#", cursor) ? cursor + 2 : cursor;
  if (!source.startsWith(identifier, identifierStart)) return undefined;
  const identifierEnd = identifierStart + identifier.length;
  return /[_\p{ID_Continue}]/u.test(source[identifierEnd] ?? "") ? undefined : identifierEnd;
}

function cfgAttrContainsPath(source: string, start: number): boolean {
  const cfgAttrEnd = identifierEndAt(source, start, "cfg_attr");
  if (cfgAttrEnd === undefined) return false;
  let cursor = skipTrivia(source, cfgAttrEnd);
  if (source[cursor] !== "(") return false;

  let parenDepth = 1;
  let bracketDepth = 0;
  let braceDepth = 0;
  let nestedMetaStart = false;
  cursor += 1;
  while (cursor < source.length && parenDepth > 0) {
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
    if (nestedMetaStart) {
      nestedMetaStart = false;
      const pathEnd = identifierEndAt(source, cursor, "path");
      if (pathEnd !== undefined && source[skipTrivia(source, pathEnd)] === "=") {
        return true;
      }
      if (cfgAttrContainsPath(source, cursor)) return true;
    }
    if (source[cursor] === "(") {
      parenDepth += 1;
    } else if (source[cursor] === ")") {
      parenDepth -= 1;
    } else if (source[cursor] === "[") {
      bracketDepth += 1;
    } else if (source[cursor] === "]" && bracketDepth > 0) {
      bracketDepth -= 1;
    } else if (source[cursor] === "{") {
      braceDepth += 1;
    } else if (source[cursor] === "}" && braceDepth > 0) {
      braceDepth -= 1;
    } else if (
      source[cursor] === "," &&
      parenDepth === 1 &&
      bracketDepth === 0 &&
      braceDepth === 0
    ) {
      nestedMetaStart = true;
    }
    cursor += 1;
  }
  return false;
}

function pathAttributeAt(source: string, hash: number): boolean {
  let cursor = skipTrivia(source, hash + 1);
  if (source[cursor] === "!") cursor = skipTrivia(source, cursor + 1);
  if (source[cursor] !== "[") return false;

  cursor = skipTrivia(source, cursor + 1);
  const pathEnd = identifierEndAt(source, cursor, "path");
  if (pathEnd !== undefined) {
    return source[skipTrivia(source, pathEnd)] === "=";
  }
  return cfgAttrContainsPath(source, cursor);
}

function attributeEnd(source: string, hash: number): number | undefined {
  let cursor = skipTrivia(source, hash + 1);
  if (source[cursor] !== "[") return undefined;
  let depth = 1;
  cursor += 1;
  while (cursor < source.length && depth > 0) {
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
    if (source[cursor] === '"') {
      cursor = skipQuoted(source, cursor);
      continue;
    }
    if (source[cursor] === "'") {
      const charEnd = charLiteralEnd(source, cursor);
      if (charEnd !== undefined) {
        cursor = charEnd;
        continue;
      }
    }
    if (source[cursor] === "[") depth += 1;
    if (source[cursor] === "]") depth -= 1;
    cursor += 1;
  }
  return depth === 0 ? cursor : undefined;
}

function siblingTestModuleEnd(source: string, start: number): number | undefined {
  let cursor = start;
  let testOnly = false;
  let siblingPath = false;
  while (source[cursor] === "#") {
    const end = attributeEnd(source, cursor);
    if (end === undefined) return undefined;
    const attribute = source.slice(cursor, end);
    if (/^#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]$/u.test(attribute)) testOnly = true;
    if (pathAttributeAt(source, cursor)) {
      if (
        siblingPath ||
        !/^#\s*\[\s*(?:r#)?path\s*=\s*"(?:[A-Za-z_][A-Za-z_0-9]*_)?tests?\.rs"\s*\]$/u.test(
          attribute,
        )
      )
        return undefined;
      siblingPath = true;
    }
    cursor = skipTrivia(source, end);
  }
  if (!testOnly || !siblingPath) return undefined;
  const declaration = /^mod\s+(?:r#)?[A-Za-z_][A-Za-z_0-9]*\s*;/u.exec(source.slice(cursor));
  return declaration === null ? undefined : cursor + declaration[0].length;
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
    if (source[cursor] === "#") {
      const testEnd = siblingTestModuleEnd(source, cursor);
      if (testEnd !== undefined) {
        cursor = testEnd;
        continue;
      }
      if (pathAttributeAt(source, cursor)) return true;
    }
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

test("gated Rust modules use ordinary discovery or test-only sibling files", () => {
  const violations = ordinaryModuleRoots.flatMap((root) =>
    rustFiles(path.join(repoRoot, root))
      .filter((file) => hasPathAttribute(fs.readFileSync(file, "utf8")))
      .map((file) => path.relative(repoRoot, file)),
  );
  assert.deepEqual(
    violations,
    [],
    `use ordinary modules or cfg(test) sibling test files:\n${violations.join("\n")}`,
  );
});

test("a test-only sibling file does not require a dedicated directory", () => {
  for (const file of ["tests.rs", "test.rs", "parser_tests.rs", "parser_test.rs"]) {
    assert.equal(hasPathAttribute(`#[cfg(test)]\n#[path = "${file}"]\nmod tests;`), false);
    assert.equal(hasPathAttribute(`#[path = "${file}"]\n#[cfg(test)]\nmod tests;`), false);
  }
  assert.equal(
    hasPathAttribute(
      '#[cfg(test)]\n// unit tests\n#[allow(dead_code)]\n#[path = "parser_tests.rs"]\nmod tests;',
    ),
    false,
  );
});

test("sibling test exceptions cannot redirect production modules or unrelated paths", () => {
  for (const source of [
    '#[path = "parser_tests.rs"]\nmod tests;',
    '#[cfg(feature = "test")]\n#[path = "parser_tests.rs"]\nmod tests;',
    '#[cfg(any(test, unix))]\n#[path = "parser_tests.rs"]\nmod tests;',
    '#[cfg(test)]\n#[path = "../tests.rs"]\nmod tests;',
    '#[cfg(test)]\n#[path = "nested/tests.rs"]\nmod tests;',
    '#[cfg(test)]\n#[path = "production.rs"]\nmod tests;',
    '#[cfg(test)]\n#[path = "parser_tests.rs"]\n#[path = "production.rs"]\nmod tests;',
    '#[cfg(test)]\n#[path = "parser_tests.rs"]\n#[cfg_attr(unix, path = "production.rs")]\nmod tests;',
    '#[cfg(test)]\nfn unrelated() {}\n#[path = "parser_tests.rs"]\nmod tests;',
    '// #[cfg(test)]\n#[path = "parser_tests.rs"]\nmod tests;',
    'const EXAMPLE: &str = "#[cfg(test)]";\n#[path = "parser_tests.rs"]\nmod tests;',
    '#[cfg(test)]\n#[path = "parser_tests.rs"]\nmod tests;\n#[path = "production.rs"]\nmod production;',
  ])
    assert.equal(hasPathAttribute(source), true, source);
});

test("the module-layout gate recognizes only path attributes", () => {
  assert.equal(hasPathAttribute('  #[path = "nested/file.rs"]\nmod file;'), true);
  assert.equal(hasPathAttribute('  #[r#path = "nested/raw-file.rs"]\nmod file;'), true);
  assert.equal(
    hasPathAttribute('  #[path /* ordinary discovery only */ = "nested/file.rs"]\nmod file;'),
    true,
  );
  assert.equal(
    hasPathAttribute(
      '  #[cfg_attr(feature = "generated", path /* ordinary discovery only */ = "nested/file.rs")]\nmod file;',
    ),
    true,
  );
  assert.equal(
    hasPathAttribute(
      '  #[cfg_attr(feature = "generated", cfg_attr(unix, path = "nested/file.rs"))]\nmod file;',
    ),
    true,
  );
  assert.equal(
    hasPathAttribute(
      '  #[r#cfg_attr(feature = "generated", r#path = "nested/raw-file.rs")]\nmod file;',
    ),
    true,
  );
  assert.equal(hasPathAttribute('#[cfg_attr(feature = "path = \\"not-code.rs\\"")]'), false);
  assert.equal(hasPathAttribute('#[some_macro(path = "metadata.json")]'), false);
  assert.equal(hasPathAttribute('#[r#some_macro(r#path = "metadata.json")]'), false);
  assert.equal(
    hasPathAttribute('#[cfg_attr(feature = "generated", some_macro(path = "metadata.json"))]'),
    false,
  );
  assert.equal(hasPathAttribute('// #[path = "comment.rs"]\nmod file;'), false);
  assert.equal(hasPathAttribute('const EXAMPLE: &str = "#[path = \\"string.rs\\"]";'), false);
  assert.equal(hasPathAttribute('const RAW: &str = r#"#[path = \\"raw.rs\\"]"#;'), false);
});
