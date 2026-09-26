import assert from "node:assert/strict";
import {
  assertDependencyReport,
  initialEntries,
  inspectLevelDependencies,
} from "../../../tools/support/compat/davinci/level-dependencies.mjs";

const revision = "f59e69c38ecbead394ba30f0fdacb5f9c1b9fd04";
export function fixture() {
  const names = [
    "vize_carton",
    "vize_impeto",
    "vize_davinci",
    "vize_l1",
    "vize_l2",
    "vize_l1_to_l2",
    "vize_l2_to_l3",
    "vize_armature",
    "vize_relief",
    "vize_croquis",
    "helper",
  ];
  const packages = names.map((name) => ({
    id: name + "@1",
    name,
    manifest_path: "/fixtures/crates/" + name + "/Cargo.toml",
    dependencies: [],
  }));
  const metadata = {
    workspace_root: "/fixtures",
    workspace_members: packages.map((pkg) => pkg.id),
    packages,
  };
  const pkg = (name) => packages.find((entry) => entry.name === name);
  const add = (from, to, options = {}) => {
    pkg(from).dependencies.push({
      name: to,
      kind: null,
      rename: null,
      target: null,
      optional: false,
      path: "/fixtures/crates/" + to,
      ...options,
    });
  };
  add("vize_l1", "vize_carton", { rename: "vize_l0" });
  add("vize_l1", "vize_armature");
  add("vize_l1", "vize_relief");
  add("vize_armature", "vize_relief");
  add("vize_l2", "vize_davinci");
  add("vize_impeto", "vize_carton", { rename: "vize_l0" });
  add("vize_l1_to_l2", "vize_l1");
  add("vize_l1_to_l2", "vize_relief");
  add("vize_l2_to_l3", "vize_l2");
  add("vize_l2_to_l3", "vize_impeto", { rename: "vize_l3" });
  const policy = {
    schema: "vize.level-dependency-allowlist",
    version: 1,
    baselineRevision: revision,
    entries: structuredClone(initialEntries).map((entry) => ({
      ...entry,
      issue: 6831,
      reason: "Test existing debt removal.",
    })),
  };
  return { metadata, policy, pkg, add, report: () => inspectLevelDependencies(metadata, policy) };
}

export function rejectsPath(change, expected) {
  const value = fixture();
  change(value);
  const report = value.report();
  assert.ok(report.unlisted.some(expected), JSON.stringify(report.unlisted));
  assert.throws(() => assertDependencyReport(report), /new level-to-legacy/u);
}
