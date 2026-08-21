import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  assertProjectLocalToolchain,
  writeFiles,
} from "../../tools/npm/smoke-release-init-project.mjs";

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
