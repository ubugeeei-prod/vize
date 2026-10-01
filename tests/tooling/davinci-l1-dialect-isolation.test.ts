import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { maskRustNonCode } from "./davinci-storage-rust-syntax.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = path.join(repoRoot, "davinci/vize_l1/src");
const coreModules = new Set(["build", "event", "parse", "render", "slice", "surface"]);
const policy =
  /\b(?:dialect|vue[0-3]|petite|quirks|LegacyCaps|LegacyDialectCapabilities|LegacyVueVersion|VueVersion)\b/u;

function rustFiles(root: string): string[] {
  return fs.readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const file = path.join(root, entry.name);
    if (entry.isDirectory()) return rustFiles(file);
    return entry.isFile() && file.endsWith(".rs") ? [file] : [];
  });
}

test("generic L1 construction and rendering never depend on dialect policy", () => {
  const files = rustFiles(sourceRoot).filter((file) => {
    const top = path.relative(sourceRoot, file).split(path.sep)[0].replace(/\.rs$/u, "");
    return coreModules.has(top);
  });
  assert.ok(files.length > coreModules.size);
  for (const file of files) {
    assert.doesNotMatch(maskRustNonCode(fs.readFileSync(file, "utf8")), policy, file);
  }
});

test("dialect isolation catches grouped and qualified references while retaining examples", () => {
  for (const source of [
    "use crate::{dialect::{self as syntax}};",
    "type Caps = ::vize_l1::dialect::LegacyCaps;",
    '#[cfg(feature = "legacy")] fn f() { crate::dialect::vue2::CAPABILITIES; }',
  ])
    assert.match(maskRustNonCode(source), policy);
  assert.doesNotMatch(
    maskRustNonCode('let example = r#"crate::dialect::vue2"#; /* LegacyCaps */'),
    policy,
  );
});
