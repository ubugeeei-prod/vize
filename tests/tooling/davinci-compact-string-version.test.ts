import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type Dependency = {
  name: string;
  req: string;
};

type Package = {
  name: string;
  version: string;
  dependencies: Dependency[];
};

type Metadata = { packages: Package[] };

function readMetadata(): Metadata {
  const result = spawnSync("cargo", ["metadata", "--format-version", "1", "--locked"], {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout) as Metadata;
}

function workspacePackage(metadata: Metadata, name: string): Package {
  const found = metadata.packages.find((pkg) => pkg.name === name);
  assert.ok(found, `workspace package ${name} must exist`);
  return found;
}

test("Davinci and OXC share one exact CompactString implementation", () => {
  const metadata = readMetadata();
  const versions = metadata.packages
    .filter((pkg) => pkg.name === "compact_str")
    .map((pkg) => pkg.version)
    .sort();

  assert.deepEqual(versions, ["0.10.0"]);

  const carton = workspacePackage(metadata, "vize_carton");
  const cartonDependency = carton.dependencies.find(
    (dependency) => dependency.name === "compact_str",
  );
  assert.ok(cartonDependency, "the S0 package must depend on compact_str");
  assert.equal(cartonDependency.req, "=0.10.0");

  const oxcSpan = workspacePackage(metadata, "oxc_span");
  assert.ok(
    oxcSpan.dependencies.some((dependency) => dependency.name === "compact_str"),
    "the pinned OXC span package must share the single compact_str package",
  );
});
