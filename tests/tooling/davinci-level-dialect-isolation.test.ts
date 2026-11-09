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
const registryCrates = [
  "vize_l0",
  "vize_l1",
  "vize_l1_to_l2",
  "vize_l2",
  "vize_l2_to_l3",
  "vize_l3",
  "vize_l4",
];
const dialectModules = new Set(["dialect", "markup", "lang", "framework", "registry"]);
// #6829: L4 keeps one target per framework runtime.
const frameworkTargets = new Set(["vize_l4/src/targets/vue2.rs", "vize_l4/src/targets/vue2"]);
const policy =
  /\b(?:dialect|vue[0-3]|petite|quirks|LegacyCaps|LegacyDialectCapabilities|LegacyVueVersion|VueVersion|VueDialect)\b/u;
const moduleDeclaration = /^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*;/gmu;
const concreteDialectVariantPath =
  /\b(?:VueVersion|VueDialect|TemplateSyntaxMode)\s*::\s*(?:[A-Za-z_]\w*|\*|\{[^{}]*\})/u;

// Core files that still reference dialect policy. Remove an entry when its file is fixed.
const owed: string[] = [];

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
  assert.equal(isCoreFile("vize_l2/src/registry.rs"), false);
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

test("dialect variant rematches include direct, glob and selected imports", () => {
  for (const rematch of [
    "VueVersion::V3",
    "use crate::VueDialect::*;",
    "use crate::TemplateSyntaxMode::Html as Markup;",
    "use crate::VueVersion::{V2, V3 as Current};",
  ])
    assert.match(rematch, concreteDialectVariantPath);
  assert.doesNotMatch(
    "use crate::registry::{capabilities as dialect_capabilities};",
    concreteDialectVariantPath,
  );
});

test("one L0 file descriptor owns every input axis and legacy types only re-export it", () => {
  const owner = fs.readFileSync(path.join(repoRoot, "davinci/vize_l0/src/dialect/file.rs"), "utf8");
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
    path.join(repoRoot, "davinci/vize_l4/src/framework/vue.rs"),
    "utf8",
  );
  assert.match(target, /registry::TargetOptions::for_file\(/u);
  assert.match(target, /\.descriptor\(\)\.dialect\(\)/u);
  assert.doesNotMatch(target, /options\.(?:version|dialect)/u);
});

test("each level has one registry and policy consumers do not rematch variants", () => {
  for (const crate of registryCrates) {
    const root = path.join(repoRoot, "davinci", crate, "src");
    assert.ok(fs.existsSync(path.join(root, "registry.rs")), `${crate} has no registry.rs`);
    assert.match(
      fs.readFileSync(path.join(root, "lib.rs"), "utf8"),
      /pub mod registry;/u,
      `${crate} does not expose its registry`,
    );
  }

  const consumers = [
    "vize_l1/src/container/vue/descriptor/policy.rs",
    "vize_l1/src/container/vue/descriptor/vue1/view.rs",
    "vize_l1/src/container/vue/descriptor/vue2/view.rs",
    "vize_l1_to_l2/src/dialect/vue3/native_file/selected_scoped.rs",
    "vize_l2/src/lang/js/file/native/scoped.rs",
    "vize_l4/src/framework/vue.rs",
  ];
  for (const relative of consumers) {
    const source = maskRustSource(
      fs.readFileSync(path.join(repoRoot, "davinci", relative), "utf8"),
    );
    assert.match(source, /registry::/u, `${relative} does not use its level registry`);
    assert.doesNotMatch(
      source,
      concreteDialectVariantPath,
      `${relative} rematches a concrete dialect variant`,
    );
  }

  const featureRoot = path.join(repoRoot, "davinci/vize_l1/src/dialect/vue/feature");
  const featureModule = fs.readFileSync(path.join(featureRoot, "mod.rs"), "utf8");
  for (const feature of [
    "attribute",
    "component",
    "computed",
    "directive",
    "event",
    "filters",
    "inheritance",
    "repeat",
    "slot_scope",
    "triple_mustache",
  ]) {
    assert.ok(
      fs.existsSync(path.join(featureRoot, `${feature}.rs`)),
      `missing Vue feature ${feature}`,
    );
    assert.ok(
      featureModule.includes(`pub mod ${feature};`),
      `Vue feature ${feature} is not exposed`,
    );
  }

  for (const version of ["vue0.rs", "vue1.rs", "vue2.rs", "vue3.rs"])
    assert.ok(
      fs.existsSync(path.join(repoRoot, "davinci/vize_l1/src/dialect", version)),
      `missing Vue version table ${version}`,
    );
});

// #6829: each proof-of-concept framework or input language is an `examples/` crate
// on public level APIs. No level crate names one or depends on one.
const proofOfConceptInputs =
  /\b(?:solid|svelte|astro|angular|coffee(?:script)?|moonbit|python|poc_support)\b/iu;
const publicLevelDependency = /^(?:vize_l[0-4](?:_to_l[0-4])?|oxc_\w+|poc-support)$/u;

function workspaceMembers(): string[] {
  const workspace = fs.readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8");
  const members = /^members = \[([\s\S]*?)\]/mu.exec(workspace)?.[1] ?? "";
  return [...members.matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
}

test("proof-of-concept inputs stay in examples on public level APIs", () => {
  const examples = workspaceMembers().filter((member) => member.startsWith("examples/"));
  assert.ok(examples.includes("examples/poc-support"), "the examples are workspace members");
  for (const crate of examples) {
    const manifest = fs.readFileSync(path.join(repoRoot, crate, "Cargo.toml"), "utf8");
    const section = manifest.split(/^\[dependencies\]$/mu)[1]?.split(/^\[/mu)[0] ?? "";
    const dependencies = [...section.matchAll(/^([\w-]+)\s*=/gmu)].map((match) => match[1]);
    assert.ok(dependencies.length > 0, `${crate} declares its dependencies`);
    assert.deepEqual(
      dependencies.filter((dependency) => !publicLevelDependency.test(dependency)),
      [],
      `${crate} reaches past the level crates`,
    );
  }
  const levels = workspaceMembers().filter((member) => /\/vize_l\d/u.test(member));
  assert.ok(levels.length >= 8, "the workspace lists the level crates");
  const named = levels
    .flatMap((crate) => rustFiles(path.join(repoRoot, crate, "src")))
    .filter((file) => proofOfConceptInputs.test(maskRustSource(fs.readFileSync(file, "utf8"))))
    .map((file) => path.relative(repoRoot, file));
  assert.deepEqual(named, [], "a level crate names a proof-of-concept input");
});
