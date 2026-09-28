import assert from "node:assert/strict";
import { test } from "node:test";

import { planFastRust } from "../../tools/support/compat/github/plan-fast-rust.mjs";

const root = "/repo";
const pkg = (name, kinds = [["lib"]]) => ({
  id: `package:${name}`,
  name,
  manifest_path: `${root}/crates/${name}/Cargo.toml`,
  dependencies: [],
  targets: kinds.map((kind) => ({ kind })),
});
const packages = [
  pkg("vize_l1"),
  pkg("vize_atelier_sfc"),
  pkg("vize_l2"),
  pkg("vize_bin", [["bin"]]),
];
const metadata = {
  workspace_root: root,
  packages,
  workspace_members: packages.map((entry) => entry.id),
};
const entry = (path, status = "M") => ({ path, status });

void test("audited SFC and L1 source edits select only their changed owners", () => {
  const plan = planFastRust(metadata, [
    entry("crates/vize_l1/src/dump.rs"),
    entry("crates/vize_atelier_sfc/src/compile.rs"),
    entry("docs/davinci/decisions/source-tier.md"),
  ]);
  assert.equal(plan.mode, "fast");
  assert.deepEqual(plan.packages, ["vize_atelier_sfc", "vize_l1"]);
  assert.deepEqual(plan.cargoArgs, ["--package", "vize_atelier_sfc", "--package", "vize_l1"]);
});

void test("global, manifest, fixture, deleted, and unknown inputs retain broad PR tests", () => {
  const additions = [
    entry("Cargo.lock"),
    entry("crates/vize_l1/Cargo.toml"),
    entry("crates/vize_l1/tests/fixture.rs"),
    entry("crates/vize_l1/src/deleted.rs", "D"),
    entry("crates/vize_l2/src/dump.rs"),
    entry("crates/vize_bin/src/main.rs"),
    entry("docs/davinci/plan/storage-inventory.tsv"),
    entry("tests/tooling/plan-fast-rust.test.mjs"),
    entry("new-dir/source.rs"),
    entry("../crates/vize_l1/src/lib.rs"),
  ];
  for (const unsafe of additions) {
    const plan = planFastRust(metadata, [entry("crates/vize_l1/src/lib.rs"), unsafe]);
    assert.equal(plan.mode, "broad", unsafe.path);
    assert.deepEqual(plan.packages, [], unsafe.path);
    assert.ok(plan.reasons.length, unsafe.path);
  }
  assert.equal(planFastRust(metadata, []).mode, "broad");
  assert.equal(
    planFastRust(metadata, [entry("crates/vize_l1/src/lib.rs")], "merge_group").mode,
    "broad",
  );
});
