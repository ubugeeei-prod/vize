import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/moonbit.ts";
import {
  getBlockedByManualPublishCrates,
  getManualPublishCrates,
  getMetadata,
  getPublishedCrates,
} from "./support/publish-crates-plan.ts";

test("publish_crates script keeps publishable workspace dependencies ordered", () => {
  const publishedCrates = getPublishedCrates();
  const publishOrder = new Map(publishedCrates.map((crateName, index) => [crateName, index]));
  const packages = new Map(getMetadata().packages.map((pkg) => [pkg.name, pkg]));

  for (const crateName of publishedCrates) {
    const pkg = packages.get(crateName);
    assert.ok(pkg, `Missing package metadata for ${crateName}`);

    for (const dependency of pkg.dependencies) {
      const dependencyOrder = publishOrder.get(dependency.name);
      const strippedOnPublish = dependency.kind === "dev";
      if (!packages.has(dependency.name) || strippedOnPublish) continue;
      assert.ok(
        dependencyOrder != null,
        `${crateName} cannot depend on deferred workspace crate ${dependency.name}`,
      );
      const crateOrder = publishOrder.get(crateName);
      assert.ok(crateOrder != null, `Missing publish order for ${crateName}`);
      assert.ok(
        dependencyOrder < crateOrder,
        `${crateName} must be published after ${dependency.name}`,
      );
    }
  }
});

test("publish_crates exactly partitions every publishable workspace crate", () => {
  const releaseCrates = [
    ...getPublishedCrates(),
    ...getManualPublishCrates(),
    ...getBlockedByManualPublishCrates(),
  ];
  const publishableCrates = getMetadata()
    .packages.filter((pkg) =>
      path.relative(repoRoot, pkg.manifest_path).startsWith(`crates${path.sep}`),
    )
    .filter((pkg) => pkg.publish === null || pkg.publish.length > 0)
    .map((pkg) => pkg.name);

  assert.equal(new Set(releaseCrates).size, releaseCrates.length, "release crate lists overlap");
  assert.deepEqual(releaseCrates.toSorted(), publishableCrates.toSorted());
});

test("publish_crates defers crates that still need manual crates.io handoff", () => {
  assert.deepEqual(getManualPublishCrates(), [
    "vize_croquis_cf",
    "vize_atelier_jsx",
    "vize_marquette",
    "vize_doctor",
  ]);
});

test("publish_crates only blocks crates that depend on manual-publish exclusions", () => {
  assert.deepEqual(getBlockedByManualPublishCrates(), ["vize_canon", "vize_patina"]);
});
