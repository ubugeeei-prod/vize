import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  dependency,
  metadata,
  readRepoFile,
  repoRoot,
  workspacePackage,
} from "./support/davinci-stage-dependencies.ts";

test("Davinci SSR compile path imports the L4 string-plan bridge", () => {
  for (const [dependencyName, rename] of [
    ["vize_davinci", null],
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

test("the SSR plan emitter never reads the legacy template AST", () => {
  const emitRoot = path.join(repoRoot, "crates", "vize_atelier_ssr", "src", "l4");
  const files = [path.join(emitRoot, "emit.rs")];
  for (const entry of fs.readdirSync(path.join(emitRoot, "emit"))) {
    files.push(path.join(emitRoot, "emit", entry));
  }
  const legacyAst =
    /\b(?:TemplateChildNode|ElementNode|RootNode|PropNode|DirectiveNode|ExpressionNode|vize_relief|vize_armature)\b/u;
  for (const file of files) {
    const source = fs.readFileSync(file, "utf8");
    assert.doesNotMatch(
      source,
      legacyAst,
      `${path.relative(repoRoot, file)} must emit from the L4 plan, not the legacy AST`,
    );
  }
});
