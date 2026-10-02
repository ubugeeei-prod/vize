import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { withoutHostRuntimeReferences } from "./support/davinci-host-imports.ts";
import { i18nHostImports } from "./support/davinci-i18n-host-imports.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const source = (file: string) => readFileSync(new URL(`../../${file}`, import.meta.url), "utf8");
const forbiddenStorage = /\bvize_carton::|use vize_carton\b/u;

test("every real i18n host import matches the reviewed scoped declarations", () => {
  for (const [file, permitted] of Object.entries(i18nHostImports)) {
    const imports = source(file).match(/^\s*(?:pub use|use) vize_carton::i18n::[^;]+;$/gmu) ?? [];
    assert.ok(imports.length > 0, file);
    assert.deepEqual(
      [...new Set(imports.map((line) => line.trim()))].sort(),
      [...permitted].sort(),
      file,
    );
    for (const declaration of imports) {
      for (const path of [file, file.replaceAll("/", "\\")]) {
        assert.doesNotMatch(
          withoutHostRuntimeReferences(declaration, path),
          forbiddenStorage,
          file,
        );
      }
    }
  }
});

test("i18n host allowance rejects storage, unreviewed catalog types and grouped escapes", () => {
  for (const file of Object.keys(i18nHostImports)) {
    for (const unreviewed of [
      "use vize_carton::String;",
      "use vize_carton::{String, i18n::Locale};",
      "use vize_carton::i18n::{Locale, String};",
      "use vize_carton::i18n::Message;",
      "use vize_carton::i18n::*;",
      "use vize_carton::i18n::translator_storage;",
    ])
      assert.match(withoutHostRuntimeReferences(unreviewed, file), forbiddenStorage, file);
  }
});

test("each i18n import remains forbidden outside its reviewed source host", () => {
  for (const declarations of Object.values(i18nHostImports)) {
    for (const declaration of declarations) {
      for (const file of [
        "davinci/vize_l2/src/lint.rs",
        "crates/vize_maestro/src/server/state/config.rs",
      ]) {
        assert.match(withoutHostRuntimeReferences(declaration, file), forbiddenStorage);
      }
    }
  }
});

test("all Rust sources retire the L0 locale/global catalog while retaining its portable provider", () => {
  const files = execFileSync("git", ["ls-files", "-z", "--", "*.rs"], {
    cwd: root,
    encoding: "utf8",
  })
    .split("\0")
    .filter(Boolean);
  for (const file of files) assert.doesNotMatch(source(file), /\bvize_l0\s*::\s*i18n\b/u, file);
  const compiler = source("davinci/vize_l0/src/compiler_error.rs");
  assert.doesNotMatch(compiler, /\b(?:Locale|translator)\b/u);
  assert.match(compiler, /messages\.lookup\(&key\)/u);
  assert.match(
    source("crates/vize/src/commands/explain/page.rs"),
    /localized_message_with\(&catalog\.messages\(\)\)/u,
  );
  assert.match(
    source("crates/vize_relief/src/errors/diagnostic.rs"),
    /localized_message_with\(&translator\(\)\.for_locale\(locale\)\)/u,
  );
});
