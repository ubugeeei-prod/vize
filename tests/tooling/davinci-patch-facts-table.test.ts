// Davinci patch-facts table contract (plan/phase-3.md P3-7).
//
// P3-7 moves VDOM patch-flag decisions out of final call assembly. This keeps
// the owner-keyed side table visible while the broad DOM parity suites pin the
// emitted bytes.

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const emitRoot = path.join(repoRoot, "crates", "vize_s1_to_s2", "src", "emit.rs");
const runPath = path.join(repoRoot, "crates", "vize_s1_to_s2", "src", "emit", "run.rs");
const patchPath = path.join(repoRoot, "crates", "vize_s1_to_s2", "src", "emit", "patch.rs");
const vnodePath = path.join(repoRoot, "crates", "vize_s1_to_s2", "src", "emit", "vnode.rs");
const componentPath = path.join(repoRoot, "crates", "vize_s1_to_s2", "src", "emit", "component.rs");
const dispatchPath = path.join(repoRoot, "crates", "vize_s1_to_s2", "src", "emit", "dispatch.rs");

function read(filePath: string): string {
  return fs.readFileSync(filePath, "utf8");
}

test("EmitCx carries an owner-keyed PatchFacts inline table", () => {
  const emitSource = read(emitRoot);
  const runSource = read(runPath);
  const patchSource = read(patchPath);

  assert.match(patchSource, /struct PatchFactsTable/);
  assert.match(patchSource, /SmallVec<\[\(NodeId, PatchFacts\); 16\]>/);
  assert.match(emitSource, /patch_facts: patch::PatchFactsTable,/);
  assert.match(runSource, /patch_facts: super::patch::PatchFactsTable::new\(\),/);
});

test("VNode and component writers read materialized patch facts", () => {
  const patchSource = read(patchPath);
  const vnodeSource = read(vnodePath);
  const componentSource = read(componentPath);

  assert.match(patchSource, /fn materialize_patch_facts/);
  assert.match(vnodeSource, /cx\.materialize_patch_facts\(id, &element\.bindings, false/);
  assert.match(componentSource, /cx\.materialize_patch_facts\(id, &component\.bindings, true/);
  assert.doesNotMatch(vnodeSource, /binding_patch_facts/);
  assert.doesNotMatch(componentSource, /binding_patch_facts/);
});

test("Patch facts consume reactivity-lattice binding metadata", () => {
  const patchSource = read(patchPath);

  assert.match(patchSource, /vize_impeto::lattice::\{/);
  assert.match(patchSource, /fn binding_kind_lattice_input/);
  assert.match(patchSource, /evaluate_binding\(binding_kind_lattice_input\(kind\)\)/);
  assert.match(patchSource, /reads_lattice_static_patch_binding_name/);
  assert.match(patchSource, /handler_static_patch_binding/);
});

test("if-branch native roots keep their owner id through dispatch", () => {
  const dispatchSource = read(dispatchPath);
  const vnodeSource = read(vnodePath);

  assert.match(dispatchSource, /key: &str,\n    id: Option<NodeId>,/);
  assert.match(dispatchSource, /vnode::emit_if_branch_element\(cx, element, key, id\)/);
  assert.match(vnodeSource, /\(true, id, PropHoistPosition::Nested\)/);
});
