import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  assertProjectLocalToolchain,
  writeFiles,
} from "../../legacy-tools/npm/smoke-release-init-project.mjs";
import { satisfiesVersionRange } from "../../legacy-tools/npm/smoke-release-semver.mjs";

test("fresh-project toolchain uses npm-compatible semver ranges", () => {
  assert.equal(satisfiesVersionRange("0.1.9", "^0.1.0"), true);
  assert.equal(satisfiesVersionRange("0.2.0", "^0.1.0"), false);
  assert.equal(satisfiesVersionRange("1.2.3-beta.1", "^1.2.3"), false);
  assert.equal(satisfiesVersionRange("1.2.3-beta.1", "^1.2.3-beta.1"), true);
  assert.equal(satisfiesVersionRange("7.0.1", "^7.0.0"), true);
});

test("fresh-project toolchain accepts package-local vize without manager shims", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-toolchain-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const projectRoot = path.join(root, "project");
  const corsaPackage = `@typescript/typescript-${process.platform}-${process.arch}`;
  const vizePackageRoot = path.join(projectRoot, "node_modules", "vize");
  writeFiles(vizePackageRoot, {
    "bin/vize": "",
    "package.json": `${JSON.stringify({ name: "vize", version: "0.0.0", optionalDependencies: { [corsaPackage]: "^7.0.0" } })}\n`,
  });
  writeFiles(path.join(vizePackageRoot, "node_modules", ...corsaPackage.split("/")), {
    "package.json": `${JSON.stringify({ name: corsaPackage, version: "7.0.1" })}\n`,
  });
  assertProjectLocalToolchain(
    {
      installDir: path.join(root, "install"),
      repoRoot: path.join(root, "repo"),
      versions: new Map([["vize", "0.0.0"]]),
    },
    projectRoot,
    { plannedDependencies: ["vize"] },
  );
});

test("fresh-project toolchain requires vize to declare the local Corsa runtime", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-toolchain-missing-corsa-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const projectRoot = path.join(root, "project");
  const corsaPackage = `@typescript/typescript-${process.platform}-${process.arch}`;
  const vizePackageRoot = path.join(projectRoot, "node_modules", "vize");
  writeFiles(vizePackageRoot, {
    "bin/vize": "",
    "package.json": `${JSON.stringify({ name: "vize", version: "0.0.0" })}\n`,
  });
  writeFiles(path.join(vizePackageRoot, "node_modules", ...corsaPackage.split("/")), {
    "package.json": `${JSON.stringify({ name: corsaPackage, version: "7.0.0" })}\n`,
  });

  assert.throws(
    () =>
      assertProjectLocalToolchain(
        {
          installDir: path.join(root, "install"),
          repoRoot: path.join(root, "repo"),
          versions: new Map([["vize", "0.0.0"]]),
        },
        projectRoot,
        { plannedDependencies: ["vize"] },
      ),
    new RegExp(`installed vize does not declare optional dependency ${corsaPackage}`),
  );
});
