import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  dependency,
  metadata,
  readRepoFile,
  repoRoot,
  workspacePackage,
} from "./support/davinci-stage-dependencies.ts";
import { productionRustModuleSources } from "./support/rust-production-modules.ts";

test("Davinci SSR compile path imports the L4 string-plan bridge", () => {
  for (const [dependencyName, rename] of [
    ["vize_l0", null],
    ["vize_l1", null],
    ["vize_l1_to_l2", null],
    ["vize_l2", null],
    ["vize_l2_to_l3", null],
    ["vize_l3", null],
  ] as const) {
    const dep = dependency(metadata, "vize_atelier_ssr", dependencyName, null);
    assert.equal(dep.rename, rename);
    assert.equal(dep.req, `=${workspacePackage(metadata, dependencyName).version}`);
  }

  const compile = readRepoFile("crates", "vize_atelier_ssr", "src", "compile.rs");
  const compileInner = readRepoFile("crates", "vize_atelier_ssr", "src", "compile", "inner.rs");
  const bridge = readRepoFile("crates", "vize_atelier_ssr", "src", "l4.rs");
  const source = readRepoFile("crates", "vize_atelier_ssr", "src", "l4", "source.rs");
  const select = readRepoFile("crates", "vize_atelier_ssr", "src", "l4", "select.rs");
  const plan = readRepoFile("crates", "vize_atelier_ssr", "src", "l4", "string_plan.rs");
  assert.match(compile, /mod inner;/u);
  assert.match(compileInner, /l4::select_ssr_lane_captured\(/u);
  assert.match(compileInner, /SsrL4Selection::Emitted/u);
  assert.match(bridge, /mod source;/u);
  assert.match(bridge, /pub\(crate\) use source::select_ssr_lane_captured/u);
  assert.match(source, /vize_l1::parse_with_options/u);
  assert.match(source, /vize_l1_to_l2::lower/u);
  assert.match(source, /select::select_from_l2/u);
  assert.match(select, /vize_l2_to_l3::lower/u);
  assert.match(select, /vize_l3::verify::verify/u);
  assert.match(plan, /PartitionFacts/u);
  // JSX SSR builds L2 itself and enters the same plan lane first.
  const jsx = readRepoFile("crates", "vize_atelier_jsx", "src", "ssr.rs");
  assert.match(jsx, /compile_l2_to_ssr\(/u);
});

test("SSR L4 selection counters name every legacy reason", () => {
  const bridge = readRepoFile("crates", "vize_atelier_ssr", "src", "l4.rs");
  const reasons = /pub\(crate\) enum LegacyReason \{(?<body>[^}]*)\}/u.exec(bridge)?.groups?.body;
  assert.ok(reasons, "l4.rs must declare the LegacyReason enum");
  const variants = [...reasons.matchAll(/^\s+(?<name>[A-Z][A-Za-z]+),$/gmu)].map(
    (match) => match.groups!.name,
  );
  assert.deepEqual(variants, [
    "Options",
    "Croquis",
    "SurfaceSemantics",
    "Operation",
    "Element",
    "Binding",
    "ExpressionOrEncoding",
    "Structure",
  ]);
  for (const counter of [
    "davinci.s4_ssr.accepted",
    "davinci.s4_ssr.rejected",
    "davinci.s4_ssr.legacy.options",
    "davinci.s4_ssr.legacy.croquis",
    "davinci.s4_ssr.legacy.surface_semantics",
    "davinci.s4_ssr.legacy.operation",
    "davinci.s4_ssr.legacy.element",
    "davinci.s4_ssr.legacy.binding",
    "davinci.s4_ssr.legacy.expression_or_encoding",
    "davinci.s4_ssr.legacy.structure",
  ]) {
    assert.ok(bridge.includes(`"${counter}"`), `l4.rs must record ${counter}`);
  }
});

function assertEmitterDoesNotReadLegacyAst(entry: string): void {
  const legacyAst =
    /\b(?:TemplateChildNode|ElementNode|RootNode|PropNode|DirectiveNode|ExpressionNode|vize_relief|vize_armature)\b/u;
  for (const [file, source] of productionRustModuleSources(entry)) {
    assert.doesNotMatch(
      source,
      legacyAst,
      `${path.relative(repoRoot, file)} must emit from the L4 plan, not the legacy AST`,
    );
  }
}

test("the SSR plan emitter never reads the legacy template AST", () => {
  assertEmitterDoesNotReadLegacyAst(
    path.join(repoRoot, "crates", "vize_atelier_ssr", "src", "l4", "emit.rs"),
  );
});

test("emitter policy follows actual test-only edges and rejects nested production legacy reads", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "ssr-emitter-module-policy-"));
  const write = (file: string, source: string) => {
    const absolute = path.join(root, file);
    fs.mkdirSync(path.dirname(absolute), { recursive: true });
    fs.writeFileSync(absolute, source);
  };
  const entry = path.join(root, "emit.rs");
  const inspect = () => assertEmitterDoesNotReadLegacyAst(entry);
  try {
    write("emit.rs", "mod control;\n");
    write("emit/control.rs", "#[cfg(test)]\nmod fixture;\nmod nested;\n");
    write("emit/control/fixture.rs", "mod child;\nuse vize_armature::TemplateChildNode;\n");
    write("emit/control/fixture/child.rs", "use vize_relief::RootNode;\n");
    write("emit/control/nested/mod.rs", "mod leaf;\n");
    write("emit/control/nested/leaf.rs", "pub fn render() {}\n");
    assert.doesNotThrow(inspect, "real cfg(test) edge covers its fixture subtree only");

    write("emit/control/nested/leaf.rs", "use vize_armature::ElementNode;\n");
    assert.throws(inspect, /nested[/\\]leaf\.rs must emit from the L4 plan/);
    write("emit/control/nested/leaf.rs", "pub fn render() {}\n");

    for (const source of [
      "mod fixture;\nmod nested;\n",
      "// #[cfg(test)]\nmod fixture;\nmod nested;\n",
      'const NOTE: &str = "#[cfg(test)] mod fixture;";\nmod fixture;\nmod nested;\n',
      "#[cfg(test)] fn witness() {}\nmod fixture;\nmod nested;\n",
      '#[cfg(any(test, feature = "production"))]\nmod fixture;\nmod nested;\n',
      "#[cfg(not(test))] mod fixture;\n#[cfg(test)] mod fixture;\nmod nested;\n",
    ]) {
      write("emit/control.rs", source);
      assert.throws(inspect, /fixture\.rs must emit from the L4 plan/);
    }
    write("emit/control.rs", "#[cfg(test)]\nmod fixture;\nmod nested;\n");
    write("emit/control/unregistered_tests.rs", "use vize_armature::PropNode;\n");
    assert.throws(inspect, /unregistered_tests\.rs must emit from the L4 plan/);
    fs.unlinkSync(path.join(root, "emit/control/unregistered_tests.rs"));

    for (const source of [
      '#[path = "fixture.rs"] mod alias;\n',
      '#[cfg_attr(feature = "production", path = "../outside.rs")] mod nested;\n',
      'include!("fixture.rs");\n',
    ]) {
      write("emit/control.rs", source);
      assert.throws(inspect, /unsupported module routing/);
    }
    write("emit/control.rs", "#[cfg(test)]\nmod fixture;\nmod nested;\n");
    fs.cpSync(path.join(root, "emit"), path.join(root, "entry_link"), { recursive: true });
    const linkedEntry = path.join(root, "entry_link.rs");
    fs.symlinkSync(entry, linkedEntry);
    assert.throws(
      () => assertEmitterDoesNotReadLegacyAst(linkedEntry),
      /unsupported symbolic module routing/,
    );
    fs.symlinkSync(path.join(root, "emit/control/fixture.rs"), path.join(root, "emit/linked.rs"));
    assert.throws(inspect, /unsupported symbolic module routing/);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
