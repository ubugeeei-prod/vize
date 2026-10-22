import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  assertAllowlistRatchet,
  assertProductAllowlistRatchet,
  assertDependencyReport,
  combineProductAllowlists,
  initialGlyphProductAllowlist,
  initialPatinaProductAllowlist,
  initialEntries,
  inspectLevelDependencies,
  policyPath,
  productPolicies,
  readBaseAllowlist,
  readMetadata,
  repoRoot,
  validateAllowlist,
  validateProductAllowlist,
} from "../../tools/support/compat/davinci/level-dependencies.ts";

import { fixture, rejectsPath } from "./support/level-dependencies.mjs";

void test("three actual entries produce five witnesses and include both alias owners", () => {
  const value = fixture(),
    report = value.report();
  assertDependencyReport(report);
  assert.equal(report.exceptions, 3);
  assert.equal(report.witnesses.length, 5);
  assert.deepEqual(
    report.roots.map(({ name }) => name),
    ["vize_carton", "vize_impeto", "vize_l1", "vize_l1_to_l2", "vize_l2", "vize_l2_to_l3"],
  );
  assert.equal(report.aliases.vize_l0, "vize_carton");
  assert.equal(report.aliases.vize_l3, "vize_impeto");
  assert.ok(report.unknown.some((entry) => entry.includes("physical L4")));
});

for (const owner of ["vize_carton", "vize_impeto"]) {
  void test("legacy normal dependency of alias owner " + owner + " is rejected", () => {
    rejectsPath(
      ({ add }) => add(owner, "vize_relief"),
      ({ root, edge }) => root === owner && edge.from === owner,
    );
  });
}

void test("canonical L3 package replaces its old alias owner without expanding debt", () => {
  const value = fixture(),
    old = value.pkg("vize_impeto");
  old.name = "vize_l3";
  old.manifest_path = "/fixtures/crates/vize_l3/Cargo.toml";
  const dep = value.pkg("vize_l2_to_l3").dependencies.find((entry) => entry.name === "vize_impeto");
  Object.assign(dep, { name: "vize_l3", rename: null, path: "/fixtures/crates/vize_l3" });
  const report = value.report();
  assertDependencyReport(report);
  assert.ok(report.roots.some(({ name }) => name === "vize_l3"));
  assert.ok(!report.roots.some(({ name }) => name === "vize_impeto"));
  assert.equal(report.exceptions, 3);
});

void test("renaming an existing legacy import does not reuse its exception", () => {
  rejectsPath(
    ({ pkg }) => {
      pkg("vize_l1").dependencies.find((entry) => entry.name === "vize_armature").rename = "lexer";
    },
    ({ edge }) => edge.to === "vize_armature" && edge.rename === "lexer",
  );
});

void test("renaming a legacy package to a level alias cannot hide its identity", () => {
  const value = fixture();
  value.add("helper", "vize_armature", { rename: "vize_l4" });
  assert.throws(value.report, /level alias hides a legacy/u);
});

void test("a registry declaration cannot reuse a measured workspace exception", () => {
  rejectsPath(
    ({ pkg }) => {
      delete pkg("vize_l1").dependencies.find((entry) => entry.name === "vize_armature").path;
    },
    ({ edge }) => edge.to === "vize_armature" && !edge.workspace,
  );
});

void test("invalid or missing comparison-base evidence fails", () => {
  assert.throws(() => readBaseAllowlist(repoRoot, "main"), /full comparison base/u);
  assert.throws(() => readBaseAllowlist(repoRoot, "0".repeat(40)));
});

void test("target-specific normal dependencies are checked even on another host", () => {
  rejectsPath(
    ({ add }) => add("vize_l2", "vize_relief", { target: "cfg(windows)" }),
    ({ edge }) => edge.target === "cfg(windows)",
  );
});

void test("inactive optional normal dependencies remain prohibited", () => {
  rejectsPath(
    ({ add }) => add("vize_l2", "vize_croquis", { optional: true }),
    ({ edge }) => edge.to === "vize_croquis" && edge.optional,
  );
});

void test("a newly introduced level root cannot inherit a known legacy exception", () => {
  const value = fixture();
  value.metadata.packages.push({
    id: "vize_l4@1",
    name: "vize_l4",
    manifest_path: "/fixtures/crates/vize_l4/Cargo.toml",
    dependencies: [],
  });
  value.metadata.workspace_members.push("vize_l4@1");
  value.add("vize_l4", "vize_l1");
  const report = value.report();
  assert.deepEqual(
    report.unlisted.map(({ root, edge }) => [root, edge.to]),
    [
      ["vize_l4", "vize_armature"],
      ["vize_l4", "vize_relief"],
    ],
  );
  assert.throws(() => assertDependencyReport(report), /new protected-root-to-legacy/u);
});

void test("Glyph is a protected product root with one exact shrinking migration permission", () => {
  const value = fixture(),
    product = structuredClone(initialGlyphProductAllowlist);
  value.add("vize_glyph", "vize_atelier_sfc");
  const combined = validateAllowlist({
    ...value.policy,
    entries: [...value.policy.entries, ...product.entries],
  });
  const report = inspectLevelDependencies(value.metadata, combined, [product.root]);
  assertDependencyReport(report);
  assert.deepEqual(report.protectedProducts, ["vize_glyph"]);
  assert.ok(
    report.witnesses.some(
      ({ root, edge, path: route }) =>
        root === "vize_glyph" &&
        edge.from === "vize_glyph" &&
        edge.to === "vize_atelier_sfc" &&
        route.join("/") === "vize_glyph/vize_atelier_sfc",
    ),
  );

  value.add("vize_glyph", "helper");
  value.add("helper", "vize_croquis");
  const expanded = inspectLevelDependencies(value.metadata, combined, [product.root]);
  assert.ok(
    expanded.unlisted.some(
      ({ root, edge, path: route }) =>
        root === "vize_glyph" &&
        edge.from === "helper" &&
        edge.to === "vize_croquis" &&
        route.join("/") === "vize_glyph/helper/vize_croquis",
    ),
  );
  assert.throws(() => assertDependencyReport(expanded), /new protected-root-to-legacy/u);
});

void test("Patina is protected across its direct and inherited Canon legacy paths", () => {
  const value = fixture();
  for (const entry of initialGlyphProductAllowlist.entries) value.add(entry.from, entry.to);
  value.add("vize_patina", "vize_canon", { target: 'cfg(not(target_arch = "wasm32"))' });
  for (const entry of initialPatinaProductAllowlist.entries) value.add(entry.from, entry.to);
  const products = combineProductAllowlists([
    initialGlyphProductAllowlist,
    initialPatinaProductAllowlist,
  ]);
  const combined = validateAllowlist({
    ...value.policy,
    entries: [...value.policy.entries, ...products.entries],
  });
  const report = inspectLevelDependencies(value.metadata, combined, products.roots);
  assertDependencyReport(report);
  assert.deepEqual(report.protectedProducts, ["vize_glyph", "vize_patina"]);
  assert.equal(report.witnesses.filter(({ root }) => root === "vize_patina").length, 11);
  assert.ok(
    report.witnesses.some(
      ({ root, edge, path: route }) =>
        root === "vize_patina" &&
        edge.from === "vize_canon" &&
        edge.to === "vize_atelier_core" &&
        route.join("/") === "vize_patina/vize_canon/vize_atelier_core",
    ),
  );

  value.add("vize_patina", "helper");
  value.add("helper", "vize_atelier_core");
  const expanded = inspectLevelDependencies(value.metadata, combined, products.roots);
  assert.ok(
    expanded.unlisted.some(
      ({ root, edge, path: route }) =>
        root === "vize_patina" &&
        edge.from === "helper" &&
        edge.to === "vize_atelier_core" &&
        route.join("/") === "vize_patina/helper/vize_atelier_core",
    ),
  );
  assert.throws(() => assertDependencyReport(expanded), /new protected-root-to-legacy/u);
});

for (const initial of [initialGlyphProductAllowlist, initialPatinaProductAllowlist]) {
  void test(`${initial.root} product policy fixes its root and only permits debt removal`, () => {
    const current = structuredClone(initial);
    assertProductAllowlistRatchet(current, initial);
    current.entries = [];
    assertProductAllowlistRatchet(current, initial);

    for (const change of [
      (policy) => {
        policy.root = policy.root === "vize_glyph" ? "vize_patina" : "vize_glyph";
      },
      (policy) => {
        policy.baselineRevision = "0".repeat(40);
      },
      (policy) => {
        policy.entries.push({ ...policy.entries[0], to: "vize_relief" });
      },
      (policy) => {
        policy.entries[0].roots = [policy.root === "vize_glyph" ? "vize_patina" : "vize_glyph"];
      },
    ]) {
      const changed = structuredClone(initial);
      change(changed);
      assert.throws(() => assertProductAllowlistRatchet(changed, initial));
    }
  });
}

void test("product policy composition merges the same legacy edge by protected root", () => {
  const canon = structuredClone(initialPatinaProductAllowlist);
  canon.root = "vize_canon";
  canon.entries = [{ ...canon.entries[0], roots: ["vize_canon"] }];
  const combined = combineProductAllowlists([initialPatinaProductAllowlist, canon]);
  assert.deepEqual(combined.roots, ["vize_canon", "vize_patina"]);
  assert.deepEqual(combined.entries[0].roots, ["vize_canon", "vize_patina"]);
});

void test("a non-level helper cannot launder a new legacy entry", () => {
  rejectsPath(
    ({ add }) => {
      add("vize_l2", "helper");
      add("helper", "vize_croquis");
    },
    ({ edge, path: route }) =>
      edge.from === "helper" && route.join("/") === "vize_l2/helper/vize_croquis",
  );
});

void test("a new root reaching an existing L1 entry is rejected", () => {
  rejectsPath(
    ({ add }) => add("vize_l2", "vize_l1"),
    ({ root, edge }) => root === "vize_l2" && edge.from === "vize_l1",
  );
});

void test("dev oracle edges are excluded at every hop; build kind remains separate", () => {
  const value = fixture();
  value.add("vize_l2", "helper");
  value.add("helper", "vize_relief", { kind: "dev" });
  value.add("vize_l1_to_l2", "vize_croquis", { kind: "dev" });
  value.add("vize_l2", "vize_armature", { kind: "build" });
  const report = value.report();
  assertDependencyReport(report);
  assert.equal(report.devOracleEdges, 1);
  assert.deepEqual(
    report.buildEdges.map(({ from, to }) => [from, to]),
    [["vize_l2", "vize_armature"]],
  );
});

void test("a normal edge is not excused by an additional dev edge", () => {
  rejectsPath(
    ({ add }) => {
      add("vize_l2", "vize_relief", { kind: "dev" });
      add("vize_l2", "vize_relief");
    },
    ({ edge }) => edge.from === "vize_l2" && edge.to === "vize_relief",
  );
});

void test("deleting an allowlist entry cannot hide a remaining dependency", () => {
  rejectsPath(
    ({ policy }) => {
      policy.entries = policy.entries.filter((entry) => entry.to !== "vize_armature");
    },
    ({ edge }) => edge.to === "vize_armature",
  );
});

void test("unused exception entries must be removed with their dependencies", () => {
  const value = fixture();
  value.pkg("vize_l1").dependencies = value
    .pkg("vize_l1")
    .dependencies.filter((dep) => dep.name !== "vize_armature");
  assert.throws(() => assertDependencyReport(value.report()), /remove stale/u);
  const old = structuredClone(value.policy);
  value.policy.entries = value.policy.entries.filter((entry) => entry.to !== "vize_armature");
  assertAllowlistRatchet(value.policy, old);
  const report = value.report();
  assertDependencyReport(report);
  assert.equal(report.exceptions, 2);
  assert.equal(report.witnesses.length, 3);
});

void test("unused root reachability must shrink without adding derived exceptions", () => {
  const value = fixture();
  value.pkg("vize_l1_to_l2").dependencies = value
    .pkg("vize_l1_to_l2")
    .dependencies.filter((dep) => dep.name !== "vize_l1");
  assert.throws(() => assertDependencyReport(value.report()), /remove stale/u);
  const old = structuredClone(value.policy);
  for (const entry of value.policy.entries.filter((entry) => entry.from === "vize_l1"))
    entry.roots = ["vize_l1"];
  assertAllowlistRatchet(value.policy, old);
  assertDependencyReport(value.report());
});

for (const [name, change] of [
  ["entry", (policy) => policy.entries.push({ ...policy.entries[0], from: "vize_l2" })],
  ["root", (policy) => policy.entries[0].roots.push("vize_l2")],
  [
    "target",
    (policy) => {
      policy.entries[0].target = "cfg(windows)";
    },
  ],
  [
    "optional",
    (policy) => {
      policy.entries[0].optional = true;
    },
  ],
  [
    "rename",
    (policy) => {
      policy.entries[0].rename = "lexer";
    },
  ],
]) {
  void test("ratchet rejects an added or changed " + name, () => {
    const value = fixture(),
      old = structuredClone(value.policy);
    change(value.policy);
    assert.throws(() => assertAllowlistRatchet(value.policy, old), /may only shrink/u);
  });
}

void test("malformed, duplicate and untracked allowlist entries fail", () => {
  for (const change of [
    (policy) => {
      delete policy.entries;
    },
    (policy) => policy.entries.push(structuredClone(policy.entries[0])),
    (policy) => {
      policy.entries[0].roots.push(policy.entries[0].roots[0]);
    },
    (policy) => {
      policy.entries[0].reason = "";
    },
    (policy) => {
      delete policy.entries[0].optional;
    },
    (policy) => {
      policy.entries[0].hiddenPermission = true;
    },
  ]) {
    const value = fixture();
    change(value.policy);
    assert.throws(() => validateAllowlist(value.policy));
  }
});

void test("missing and ambiguous metadata identity fail instead of passing", () => {
  for (const change of [
    ({ metadata }) => {
      delete metadata.workspace_members;
    },
    ({ metadata }) => {
      metadata.workspace_members.push("missing@1");
    },
    ({ metadata }) => {
      metadata.packages[1].id = metadata.packages[0].id;
    },
    ({ pkg }) => {
      delete pkg("vize_l1").dependencies[0].path;
    },
    ({ pkg }) => {
      delete pkg("vize_l1").dependencies[0].target;
    },
    ({ pkg }) => {
      pkg("vize_l1").dependencies[0].path = "/fixtures/crates/vize_relief";
    },
    ({ metadata }) => {
      metadata.packages[1].manifest_path = metadata.packages[0].manifest_path;
    },
  ]) {
    const value = fixture();
    change(value);
    assert.throws(value.report);
  }
});

void test("the real workspace metadata has no new or stale legacy permissions", () => {
  const policy = validateAllowlist(
    JSON.parse(readFileSync(path.join(repoRoot, policyPath), "utf8")),
  );
  const products = productPolicies.map(({ path: productPath, initial }) => {
    const product = validateProductAllowlist(
      JSON.parse(readFileSync(path.join(repoRoot, productPath), "utf8")),
    );
    assertProductAllowlistRatchet(product, initial);
    return product;
  });
  const product = combineProductAllowlists(products);
  assertAllowlistRatchet(policy, { entries: initialEntries });
  const report = inspectLevelDependencies(
    readMetadata(),
    validateAllowlist({ ...policy, entries: [...policy.entries, ...product.entries] }),
    product.roots,
  );
  assertDependencyReport(report);
  assert.ok(report.roots.length > 0);
  assert.deepEqual(report.protectedProducts, ["vize_glyph", "vize_patina"]);
});
