import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

type CargoMetadata = {
  packages: CargoPackage[];
};

type CargoPackage = {
  name: string;
  dependencies: CargoDependency[];
};

type CargoDependency = {
  kind: "dev" | "build" | null;
  name: string;
};

const repoRoot = path.resolve(import.meta.dirname, "..");
const publishScriptPath = path.join(repoRoot, "scripts", "publish-crates.sh");

const getPublishedCrates = () =>
  Array.from(
    readFileSync(publishScriptPath, "utf8").matchAll(/^publish_crate (vize_[a-z_]+)$/gm),
    ([, crateName]) => crateName,
  );

const getMetadata = (): CargoMetadata =>
  JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: repoRoot,
      encoding: "utf8",
    }),
  ) as CargoMetadata;

test("publish-crates script keeps publishable workspace dependencies ordered", () => {
  const publishedCrates = getPublishedCrates();
  const publishOrder = new Map(publishedCrates.map((crateName, index) => [crateName, index]));
  const metadata = getMetadata();
  const packages = new Map(metadata.packages.map((pkg) => [pkg.name, pkg]));

  for (const crateName of publishedCrates) {
    const pkg = packages.get(crateName);
    assert.ok(pkg, `Missing package metadata for ${crateName}`);

    for (const dependency of pkg.dependencies) {
      if (dependency.kind === "dev") {
        continue;
      }

      const dependencyOrder = publishOrder.get(dependency.name);
      if (dependencyOrder == null) {
        continue;
      }

      const crateOrder = publishOrder.get(crateName);
      assert.ok(crateOrder != null, `Missing publish order for ${crateName}`);
      assert.ok(
        dependencyOrder < crateOrder,
        `${crateName} must be published after ${dependency.name}`,
      );
    }
  }
});
