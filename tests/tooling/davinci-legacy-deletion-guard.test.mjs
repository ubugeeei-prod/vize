import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import path from "node:path";
import {
  hasExactSuccessfulAudit,
  missingLegacyRouteWitnesses,
  removedLegacyManifests,
  retiredLegacyRouteWitnesses,
} from "../../tools/support/compat/davinci/legacy-deletion-guard.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sha = "a".repeat(40);
const run = {
  head_sha: sha,
  path: ".github/workflows/level-deletion-readiness.yml",
  event: "workflow_dispatch",
  conclusion: "success",
};

void test("crate manifest deletion or move triggers the guard", () => {
  assert.deepEqual(
    removedLegacyManifests(
      [
        "D\tcrates/vize_armature/Cargo.toml",
        "D\tcrates/vize_croquis_cf/Cargo.toml",
        "R100\tcrates/vize_atelier_core/Cargo.toml\tdavinci/vize_l2/Cargo.toml",
        "D\tcrates/vize_relief/src/lib.rs",
        "M\tcrates/vize_croquis/Cargo.toml",
        "D\tdavinci/vize_l1/Cargo.toml",
      ].join("\n"),
    ),
    [
      "crates/vize_armature/Cargo.toml",
      "crates/vize_croquis_cf/Cargo.toml",
      "crates/vize_atelier_core/Cargo.toml",
    ],
  );
});

void test("audit lookup rejects absent, stale, or unrelated runs", () => {
  assert.equal(hasExactSuccessfulAudit({ total_count: 1, workflow_runs: [run] }, sha), true);
  assert.equal(hasExactSuccessfulAudit({ total_count: 0, workflow_runs: [] }, sha), false);
  assert.equal(hasExactSuccessfulAudit({ workflow_runs: [run] }, sha), false);
  assert.equal(
    hasExactSuccessfulAudit({ total_count: 1, workflow_runs: [run] }, "b".repeat(40)),
    false,
  );
  for (const changed of [
    { conclusion: "failure" },
    { event: "push" },
    { path: ".github/workflows/check.yml" },
  ]) {
    assert.equal(
      hasExactSuccessfulAudit({ total_count: 1, workflow_runs: [{ ...run, ...changed }] }, sha),
      false,
    );
  }
});

void test("retiring a retained compiler route witness triggers the guard", () => {
  const dom = "crates/vize_atelier_dom/src/compile/inner.rs";
  const vapor = "crates/vize_atelier_vapor/src/compile/inner.rs";
  const base = {
    [dom]: "generate_with_sections_and_experimental_options_slot_scope_and_environment(",
    [vapor]: [
      "transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id(",
      "vapor_lower::transform_to_ir_with_spans(",
    ].join("\n"),
  };

  assert.deepEqual(retiredLegacyRouteWitnesses(base, base), []);
  assert.deepEqual(retiredLegacyRouteWitnesses(base, { ...base, [dom]: "" }), [
    `DOM compatibility emitter (${dom})`,
  ]);
  assert.deepEqual(
    retiredLegacyRouteWitnesses(base, {
      ...base,
      [vapor]: "vapor_lower::transform_to_ir_with_spans(",
    }),
    [`Vapor compatibility transform (${vapor})`],
  );
  assert.deepEqual(retiredLegacyRouteWitnesses(base, { ...base, [vapor]: undefined }), [
    `Vapor compatibility transform (${vapor})`,
    `Vapor compatibility emitter (${vapor})`,
  ]);
});

void test("the checked-in compiler routes retain every guarded witness", () => {
  const files = Object.fromEntries(
    [
      "crates/vize_atelier_dom/src/compile/inner.rs",
      "crates/vize_atelier_dom/src/compile/sfc.rs",
      "crates/vize_atelier_ssr/src/compile.rs",
      "crates/vize_atelier_vapor/src/compile/inner.rs",
    ].map((file) => [file, fs.readFileSync(path.join(root, file), "utf8")]),
  );
  assert.deepEqual(missingLegacyRouteWitnesses(files), []);
});

void test("the existing PR and merge-group check runs the conditional deletion guard", () => {
  const workflow = fs.readFileSync(path.join(root, ".github/workflows/check.yml"), "utf8");
  assert.match(workflow, /- name: Guard legacy crate deletion\n\s+if: .*pull_request.*merge_group/);
  assert.match(workflow, /BASE_SHA: .*pull_request\.base\.sha.*merge_group\.base_sha/);
  assert.match(workflow, /CANDIDATE_SHA: .*pull_request\.head\.sha.*merge_group\.head_sha/);
  assert.match(workflow, /vp node tools\/support\/compat\/davinci\/legacy-deletion-guard\.ts/);
});
