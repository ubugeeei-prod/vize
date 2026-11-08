import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  isExactVersion,
  resolveBinPath,
  resolvePackageRoot,
  scorePackageId,
} from "../../tools/commands/editors/vscode/run-package-bin.ts";

test("package binary resolution accepts only exact semantic versions", () => {
  assert.equal(isExactVersion("1.2.3"), true);
  assert.equal(isExactVersion("1.2.3-beta.1"), true);
  assert.equal(isExactVersion("^1.2.3"), false);
  assert.equal(isExactVersion("workspace:*"), false);
});

test("package map scoring prefers the declared version without peer suffixes", () => {
  assert.ok(
    scorePackageId("tool@2.4.0", "tool", "2.4.0") > scorePackageId("tool@2.3.0", "tool", "2.4.0"),
  );
  assert.ok(
    scorePackageId("tool@2.4.0", "tool", "2.4.0") >
      scorePackageId("tool@2.4.0(peer@1.0.0)", "tool", "2.4.0"),
  );
});

test("package binary resolution supports string and named manifests", () => {
  assert.equal(resolveBinPath({ name: "tool", bin: "cli.mjs" }, "tool"), "cli.mjs");
  assert.equal(resolveBinPath({ name: "tool", bin: { other: "other.mjs" } }, "other"), "other.mjs");
  assert.throws(() => resolveBinPath({ name: "tool", bin: {} }, "missing"), /does not expose/);
});

test("package root resolution prefers the exact package map candidate", () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-package-bin-"));
  const nodeModules = path.join(fixture, "node_modules");
  const oldPackage = path.join(nodeModules, ".store/tool-1/node_modules/tool");
  const expectedPackage = path.join(nodeModules, ".store/tool-2/node_modules/tool");
  try {
    fs.mkdirSync(oldPackage, { recursive: true });
    fs.mkdirSync(expectedPackage, { recursive: true });
    fs.writeFileSync(path.join(oldPackage, "package.json"), "{}\n");
    fs.writeFileSync(path.join(expectedPackage, "package.json"), "{}\n");
    fs.writeFileSync(
      path.join(fixture, "package.json"),
      `${JSON.stringify({ devDependencies: { tool: "2.0.0" } })}\n`,
    );
    fs.writeFileSync(
      path.join(nodeModules, ".package-map.json"),
      `${JSON.stringify({
        packages: {
          "tool@1.0.0": { url: "./.store/tool-1/node_modules/tool" },
          "tool@2.0.0": { url: "./.store/tool-2/node_modules/tool" },
        },
      })}\n`,
    );

    assert.equal(resolvePackageRoot(fixture, "tool"), expectedPackage);
  } finally {
    fs.rmSync(fixture, { force: true, recursive: true });
  }
});
