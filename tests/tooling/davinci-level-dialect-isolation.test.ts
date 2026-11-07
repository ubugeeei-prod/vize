// #6841: dialect code lives in each level's `dialect/`, `markup/`, `lang/` or `framework/`
// modules; core modules never reference dialect policy. L1 has its own stricter gate
// (davinci-l1-dialect-isolation.test.ts). L0 is exempt: it owns the configuration that
// selects a dialect. Test code is exempt. `owed` only shrinks.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { maskRustSource } from "./davinci-storage-rust-syntax.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const crates = ["vize_l1_to_l2", "vize_l2", "vize_l2_to_l3", "vize_l3", "vize_l4"];
const dialectModules = new Set(["dialect", "markup", "lang", "framework"]);
// #6829: L4 keeps one target per framework runtime.
const frameworkTargets = new Set(["vize_l4/src/targets/vue2.rs", "vize_l4/src/targets/vue2"]);
const policy =
  /\b(?:dialect|vue[0-3]|petite|quirks|LegacyCaps|LegacyDialectCapabilities|LegacyVueVersion|VueVersion|VueDialect)\b/u;
const moduleDeclaration = /^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*;/gmu;

// Core files that still reference dialect policy. Remove an entry when its file is fixed.
const owed = [
  "vize_l1_to_l2/src/emit/budget.rs",
  "vize_l1_to_l2/src/emit/budget/preflight.rs",
  "vize_l1_to_l2/src/emit/budget/slot_scope.rs",
  "vize_l1_to_l2/src/emit/entry.rs",
  "vize_l1_to_l2/src/lib.rs",
  "vize_l1_to_l2/src/lower.rs",
  "vize_l1_to_l2/src/lower/caps.rs",
  "vize_l1_to_l2/src/lower/cx.rs",
  "vize_l1_to_l2/src/lower/expr.rs",
  "vize_l1_to_l2/src/lower/foreign.rs",
  "vize_l1_to_l2/src/lower/forop.rs",
  "vize_l1_to_l2/src/lower/frozen.rs",
  "vize_l1_to_l2/src/native/element.rs",
  "vize_l1_to_l2/src/native/pattern.rs",
  "vize_l1_to_l2/src/native_file/selected_scoped.rs",
  "vize_l1_to_l2/src/native_vue2.rs",
  "vize_l1_to_l2/src/native_vue2/children.rs",
  "vize_l1_to_l2/src/native_vue2/element.rs",
  "vize_l1_to_l2/src/pass/legacy.rs",
  "vize_l1_to_l2/src/pass/plan.rs",
  "vize_l2/src/artifact/check/original_for.rs",
  "vize_l2/src/dump/owned/expr.rs",
  "vize_l2/src/dump/parse/expr_token.rs",
  "vize_l2/src/dump/print.rs",
  "vize_l2/src/expr/foreign.rs",
  "vize_l2/src/file/region/native/body/header.rs",
  "vize_l2/src/file/region/native/interruption.rs",
  "vize_l2_to_l3/src/lower/operands.rs",
  "vize_l4/src/targets/ts/vue.rs",
];

function rustFiles(root: string): string[] {
  return fs.readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const file = path.join(root, entry.name);
    if (entry.isDirectory()) return rustFiles(file);
    return entry.isFile() && file.endsWith(".rs") ? [file] : [];
  });
}

function isCoreFile(relative: string): boolean {
  const segments = relative.split("/");
  const top = segments[2]?.replace(/\.rs$/u, "");
  if (top === undefined || dialectModules.has(top)) return false;
  if (
    [...frameworkTargets].some((target) => relative === target || relative.startsWith(target + "/"))
  )
    return false;
  return !segments.includes("tests") && !/(?:^|_)tests?\.rs$/u.test(segments.at(-1) ?? "");
}

function referencesDialect(source: string): boolean {
  return policy.test(maskRustSource(source).replace(moduleDeclaration, ""));
}

function offenders(root: string): string[] {
  return crates
    .flatMap((crate) => rustFiles(path.join(root, crate, "src")))
    .map((file) => path.relative(root, file).split(path.sep).join("/"))
    .filter((relative) => isCoreFile(relative))
    .filter((relative) => referencesDialect(fs.readFileSync(path.join(root, relative), "utf8")))
    .sort();
}

test("no core module outside the owed list references dialect policy", () => {
  const found = offenders(path.join(repoRoot, "davinci"));
  assert.deepEqual(
    found.filter((file) => !owed.includes(file)),
    [],
    "new dialect references in core modules; move them into a dialect module",
  );
});

test("the owed list only shrinks", () => {
  const found = new Set(offenders(path.join(repoRoot, "davinci")));
  assert.deepEqual(
    owed.filter((file) => !found.has(file)),
    [],
    "fixed files must leave the owed list",
  );
  assert.deepEqual([...owed].sort(), owed, "keep the owed list sorted");
});

test("dialect modules, test code and module declarations are exempt; code is not", () => {
  assert.equal(isCoreFile("vize_l2/src/lang/js.rs"), false);
  assert.equal(isCoreFile("vize_l3/src/dialect/vue2.rs"), false);
  assert.equal(isCoreFile("vize_l4/src/targets/vue2/emit.rs"), false);
  assert.equal(isCoreFile("vize_l3/src/decision/tests.rs"), false);
  assert.equal(isCoreFile("vize_l2/src/file/region/native/for_tests.rs"), false);
  assert.equal(isCoreFile("vize_l2/src/file/tests/scope.rs"), false);
  assert.equal(isCoreFile("vize_l3/src/decision.rs"), true);
  assert.equal(isCoreFile("vize_l4/src/targets/ts/vue.rs"), true);
  assert.equal(referencesDialect("pub mod dialect;\nmod lang;\n"), false);
  assert.equal(referencesDialect('// vue2\nconst NAME: &str = "quirks";\n'), false);
  assert.equal(referencesDialect("#[cfg(test)]\nmod tests { use crate::dialect::vue2; }\n"), false);
  assert.equal(referencesDialect("use crate::dialect::vue3::VueDirectives;\n"), true);
  assert.equal(referencesDialect("if version != VueVersion::V3 {}\n"), true);
  assert.equal(referencesDialect("fn f(caps: LegacyCaps) {}\n"), true);
});

test("one L0 file descriptor owns every input axis and legacy types only re-export it", () => {
  const owner = fs.readFileSync(
    path.join(repoRoot, "davinci/vize_l0/src/dialect/file.rs"),
    "utf8",
  );
  for (const field of ["version", "framework", "syntax", "profile", "lang"])
    assert.match(owner, new RegExp(`\\b${field}:`, "u"), `missing FileDialect.${field}`);

  const relief = fs.readFileSync(path.join(repoRoot, "crates/vize_relief/src/options.rs"), "utf8");
  assert.match(relief, /pub use vize_l0::dialect::TemplateSyntaxMode;/u);
  assert.doesNotMatch(relief, /pub enum TemplateSyntaxMode/u);

  const descriptor = fs.readFileSync(
    path.join(repoRoot, "davinci/vize_l1/src/container/vue/descriptor.rs"),
    "utf8",
  );
  assert.match(descriptor, /dialect: FileDialect/u);
  assert.match(descriptor, /options\.resolve\(state\.lang\(\)\)/u);

  const document = fs.readFileSync(
    path.join(repoRoot, "davinci/vize_l1/src/markup/document.rs"),
    "utf8",
  );
  assert.match(document, /FileDialect::document\(configured, root\.source\(\), syntax, lang\)/u);

  const target = fs.readFileSync(
    path.join(repoRoot, "davinci/vize_l4/src/targets/ts/vue.rs"),
    "utf8",
  );
  assert.match(target, /\.dialect\(\)[\s\S]*\.is_standard_vue_component/u);
  assert.doesNotMatch(target, /options\.(?:version|dialect)/u);
});
