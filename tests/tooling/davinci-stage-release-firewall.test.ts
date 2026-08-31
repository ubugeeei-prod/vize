import assert from "node:assert/strict";
import { test } from "node:test";

import { metadata, workspacePackage, type Package } from "./support/davinci-stage-dependencies.ts";

const unpublishedDavinciStages = new Set(["vize_davinci", "vize_s1", "vize_s2", "vize_s1_to_s2"]);

function isPublishable(pkg: Package): boolean {
  return pkg.publish === null || pkg.publish.length > 0;
}

type FeatureOwner = Pick<Package, "dependencies">;

function featureDependencyKey(featureValue: string): string {
  return featureValue.replace(/^dep:/u, "").split(/[?/]/u, 1)[0];
}

function referencedWorkspaceFeaturePackage(pkg: FeatureOwner, featureValue: string): string | null {
  const dependencyKey = featureDependencyKey(featureValue);
  const dependency = pkg.dependencies.find(
    (dependency) => (dependency.rename ?? dependency.name) === dependencyKey,
  );
  return dependency?.name ?? null;
}

test("Feature values resolve unpublished stages through dependency keys", () => {
  const pkg: FeatureOwner = {
    dependencies: [
      {
        name: "vize_s1_to_s2",
        features: [],
        rename: "stage_alias",
        kind: null,
        optional: true,
        req: "*",
      },
    ],
  };

  assert.equal(referencedWorkspaceFeaturePackage(pkg, "dep:stage_alias"), "vize_s1_to_s2");
  assert.equal(referencedWorkspaceFeaturePackage(pkg, "stage_alias?/legacy"), "vize_s1_to_s2");
  assert.equal(referencedWorkspaceFeaturePackage(pkg, "vize_s1"), null);
  assert.equal(referencedWorkspaceFeaturePackage(pkg, "local_feature"), null);
});

test("Published crates expose no feature that enables unpublished Davinci stages", () => {
  const offenders: string[] = [];

  for (const pkg of metadata.packages.filter(isPublishable)) {
    for (const [featureName, featureValues] of Object.entries(pkg.features)) {
      for (const featureValue of featureValues) {
        const referencedPackage = referencedWorkspaceFeaturePackage(pkg, featureValue);
        if (!unpublishedDavinciStages.has(referencedPackage)) continue;
        offenders.push(`${pkg.name} feature ${featureName} includes ${featureValue}`);
      }
    }
  }

  assert.deepEqual(offenders, []);
});

test("DOM S2 witnesses keep their unpublished stage edges test-space only", () => {
  const dom = workspacePackage(metadata, "vize_atelier_dom");
  assert.deepEqual(dom.features, {
    legacy: ["vize_atelier_core/legacy"],
  });

  const stageEdges = dom.dependencies
    .filter((dependency) => unpublishedDavinciStages.has(dependency.name))
    .map((dependency) => ({
      name: dependency.name,
      kind: dependency.kind,
      req: dependency.req,
      rename: dependency.rename,
      optional: dependency.optional,
      features: dependency.features,
    }))
    .sort((left, right) => left.name.localeCompare(right.name));

  assert.deepEqual(stageEdges, [
    {
      name: "vize_davinci",
      kind: "dev",
      req: "*",
      rename: null,
      optional: false,
      features: [],
    },
    {
      name: "vize_s1_to_s2",
      kind: "dev",
      req: "*",
      rename: null,
      optional: false,
      features: [],
    },
  ]);
});
