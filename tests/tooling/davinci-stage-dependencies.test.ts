import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type Dependency = {
  name: string;
  rename: string | null;
  kind: "dev" | "build" | null;
};

type Package = {
  name: string;
  dependencies: Dependency[];
};

type Metadata = { packages: Package[] };

function readMetadata(): Metadata {
  const result = spawnSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--locked"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout) as Metadata;
}

function workspacePackage(metadata: Metadata, name: string): Package {
  const found = metadata.packages.find((pkg) => pkg.name === name);
  assert.ok(found, `workspace package ${name} must exist`);
  return found;
}

const aliases = new Map<string, ReadonlyArray<readonly [string, string]>>([
  ["vize_davinci", [["vize_carton", "vize_s0"]]],
  ["vize_sinopia", [["vize_carton", "vize_s0"]]],
  ["vize_disegno", [["vize_carton", "vize_s0"]]],
  [
    "vize_ricalco",
    [
      ["vize_carton", "vize_s0"],
      ["vize_sinopia", "vize_s1"],
      ["vize_disegno", "vize_s2"],
    ],
  ],
]);

const metadata = readMetadata();

test("Davinci crates import retained packages through stage aliases", () => {
  for (const [packageName, expectedAliases] of aliases) {
    const dependencies = workspacePackage(metadata, packageName).dependencies;
    for (const [dependencyName, rename] of expectedAliases) {
      assert.ok(
        dependencies.some(
          (dependency) =>
            dependency.kind === null &&
            dependency.name === dependencyName &&
            dependency.rename === rename,
        ),
        `${packageName} must import ${dependencyName} as ${rename}`,
      );
    }
  }
});

test("Davinci stage dependencies are one-way and acyclic", () => {
  const tiers = new Map<string, number>([
    ["vize_carton", 0],
    ["vize_davinci", 1],
    ["vize_sinopia", 1],
    ["vize_disegno", 2],
    ["vize_ricalco", 3],
  ]);
  const expectedEdges = new Map<string, string[]>([
    ["vize_carton", []],
    ["vize_davinci", ["vize_carton"]],
    ["vize_sinopia", ["vize_carton"]],
    ["vize_disegno", ["vize_carton", "vize_davinci"]],
    ["vize_ricalco", ["vize_carton", "vize_davinci", "vize_disegno", "vize_sinopia"]],
  ]);

  for (const [packageName, packageTier] of tiers) {
    const dependencies = workspacePackage(metadata, packageName).dependencies;
    const stageEdges = dependencies
      .filter((dependency) => dependency.kind === null && tiers.has(dependency.name))
      .map((dependency) => dependency.name)
      .sort();
    assert.deepEqual(stageEdges, expectedEdges.get(packageName));

    for (const dependency of dependencies) {
      if (dependency.kind !== null) continue;
      const dependencyTier = tiers.get(dependency.name);
      if (dependencyTier === undefined) continue;
      assert.ok(
        dependencyTier < packageTier,
        `${packageName} (tier ${packageTier}) reverses the edge to ${dependency.name} ` +
          `(tier ${dependencyTier})`,
      );
    }
  }
});
