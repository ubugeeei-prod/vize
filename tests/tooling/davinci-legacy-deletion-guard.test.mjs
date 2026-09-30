import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import path from "node:path";
import {
  hasExactSuccessfulAudit,
  removedLegacyManifests,
} from "../../tools/support/compat/davinci/legacy-deletion-guard.mjs";

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

void test("the existing PR and merge-group check runs the conditional deletion guard", () => {
  const workflow = fs.readFileSync(path.join(root, ".github/workflows/check.yml"), "utf8");
  assert.match(workflow, /- name: Guard legacy crate deletion\n\s+if: .*pull_request.*merge_group/);
  assert.match(workflow, /BASE_SHA: .*pull_request\.base\.sha.*merge_group\.base_sha/);
  assert.match(workflow, /CANDIDATE_SHA: .*pull_request\.head\.sha.*merge_group\.head_sha/);
  assert.match(workflow, /vp node tools\/support\/compat\/davinci\/legacy-deletion-guard\.mjs/);
});
