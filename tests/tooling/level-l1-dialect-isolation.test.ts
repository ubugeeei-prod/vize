import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
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

function genericFiles(root: string): string[] {
  return rustFiles(root).filter((file) => {
    const relative = path.relative(root, file).split(path.sep).join("/");
    const top = relative.split("/")[0].replace(/\.rs$/u, "");
    return (
      coreModules.has(top) ||
      relative === "markup/profile.rs" ||
      relative === "markup/lex.rs" ||
      relative.startsWith("markup/lex/")
    );
  });
}

function assertGenericIsolation(root: string): void {
  for (const file of genericFiles(root)) {
    assert.doesNotMatch(maskRustNonCode(fs.readFileSync(file, "utf8")), policy, file);
  }
}

test("generic L1 construction, rendering and lexer never depend on dialect policy", () => {
  assert.ok(genericFiles(sourceRoot).length > coreModules.size);
  assertGenericIsolation(sourceRoot);
});

test("generic lexer and profile policy mutations fail without rejecting dialect markup hooks", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-l1-isolation-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const genericMarkupFiles = [
    "markup/profile.rs",
    "markup/lex.rs",
    "markup/lex/native.rs",
    "markup/lex/compat.rs",
    "markup/lex/nested/token.rs",
  ];
  for (const relative of ["markup.rs", "markup/directive.rs", ...genericMarkupFiles]) {
    const file = path.join(root, relative);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(
      file,
      genericMarkupFiles.includes(relative) ? "fn generic() {}" : "use crate::dialect;",
    );
  }
  assertGenericIsolation(root);
  for (const relative of genericMarkupFiles) {
    const file = path.join(root, relative);
    fs.writeFileSync(file, "use crate::dialect::LegacyCaps;");
    assert.throws(
      () => assertGenericIsolation(root),
      (error) => error instanceof assert.AssertionError && error.message.includes(file),
    );
    fs.writeFileSync(file, "fn generic() {}");
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
