import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { createVizeSymlinks, VIZE_LOCAL_PACKAGES } from "../_helpers/vize-local-packages.ts";

interface PackageManifest {
  dependencies?: Record<string, string>;
  name?: string;
  optionalDependencies?: Record<string, string>;
  peerDependencies?: Record<string, string>;
}

function readManifest(packageDir: string): PackageManifest {
  return JSON.parse(
    fs.readFileSync(path.join(packageDir, "package.json"), "utf8"),
  ) as PackageManifest;
}

test("app fixture links cover local packages' runtime workspace dependency graph", () => {
  const packagesByName = new Map(VIZE_LOCAL_PACKAGES.map((pkg) => [pkg.packageName, pkg]));
  const packageOrder = new Map(VIZE_LOCAL_PACKAGES.map((pkg, index) => [pkg.packageName, index]));
  const omissions: string[] = [];

  for (const [index, pkg] of VIZE_LOCAL_PACKAGES.entries()) {
    if (!pkg.linkIntoFixture) continue;
    const manifest = readManifest(pkg.dir);
    assert.equal(manifest.name, pkg.packageName, `${pkg.packageName} package directory`);

    const runtimeDependencies = {
      ...manifest.dependencies,
      ...manifest.optionalDependencies,
      ...manifest.peerDependencies,
    };
    for (const [dependency, version] of Object.entries(runtimeDependencies)) {
      if (!version.startsWith("workspace:")) continue;
      const target = packagesByName.get(dependency);
      if (!target) {
        omissions.push(`${pkg.packageName} -> ${dependency}: missing local package target`);
      } else if (!target.linkIntoFixture) {
        omissions.push(`${pkg.packageName} -> ${dependency}: not linked into fixtures`);
      } else if ((packageOrder.get(dependency) ?? Number.POSITIVE_INFINITY) >= index) {
        omissions.push(`${pkg.packageName} -> ${dependency}: dependency must be prepared first`);
      }
    }
  }

  assert.deepEqual(omissions, []);
});

test("app fixture links include scoped and unscoped package names", () => {
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-app-local-packages-"));
  const nodeModules = path.join(fixtureRoot, "node_modules");

  try {
    createVizeSymlinks(nodeModules);
    const linkedPackages = VIZE_LOCAL_PACKAGES.filter((candidate) => candidate.linkIntoFixture);
    assert.ok(
      linkedPackages.some((pkg) => pkg.packageName.startsWith("@")),
      "expected at least one scoped linked package",
    );
    assert.ok(
      linkedPackages.some((pkg) => !pkg.packageName.startsWith("@")),
      "expected at least one unscoped linked package",
    );

    for (const pkg of linkedPackages) {
      const link = path.join(nodeModules, ...pkg.packageName.split("/"));
      assert.equal(fs.realpathSync(link), fs.realpathSync(pkg.dir), pkg.packageName);
    }
  } finally {
    fs.rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test("app fixture links replace valid symlinks pointing elsewhere", () => {
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-app-local-packages-"));
  const nodeModules = path.join(fixtureRoot, "node_modules");
  const decoy = path.join(fixtureRoot, "decoy");

  try {
    fs.mkdirSync(decoy, { recursive: true });
    const [pkg] = VIZE_LOCAL_PACKAGES.filter((candidate) => candidate.linkIntoFixture);
    const link = path.join(nodeModules, ...pkg.packageName.split("/"));
    fs.mkdirSync(path.dirname(link), { recursive: true });
    fs.symlinkSync(decoy, link, "dir");

    createVizeSymlinks(nodeModules);

    assert.equal(fs.realpathSync(link), fs.realpathSync(pkg.dir), pkg.packageName);
  } finally {
    fs.rmSync(fixtureRoot, { recursive: true, force: true });
  }
});
