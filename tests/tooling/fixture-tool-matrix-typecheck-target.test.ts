import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { validateTypecheckPerformanceTarget } from "../../tools/fixtures/tool-matrix-typecheck-target.mjs";

test("fixture tool matrix requires an exact baseline tsconfig for performance targets", () => {
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typecheck-target-"));
  const project = {
    id: "performance-fixture",
    tsconfig: "configs/tsconfig.check.json",
    typecheckPerformance: {
      enabled: true,
      compareTo: "vue-tsc",
      packageManager: "pnpm",
      packageManagerVersion: "10.0.0",
      lockfile: "pnpm-lock.yaml",
    },
  };
  try {
    fs.mkdirSync(path.join(fixtureRoot, "configs"));
    fs.writeFileSync(path.join(fixtureRoot, project.tsconfig), "{}\n");
    fs.writeFileSync(path.join(fixtureRoot, "pnpm-lock.yaml"), "lockfileVersion: '9.0'\n");
    assert.doesNotThrow(() => validateTypecheckPerformanceTarget(project, fixtureRoot));
    fs.writeFileSync(path.join(fixtureRoot, "yarn.lock"), "# yarn lockfile v1\n");
    assert.doesNotThrow(() =>
      validateTypecheckPerformanceTarget(
        {
          ...project,
          typecheckPerformance: {
            ...project.typecheckPerformance,
            packageManager: "yarn",
            lockfile: "yarn.lock",
          },
        },
        fixtureRoot,
      ),
    );
    fs.writeFileSync(path.join(fixtureRoot, "package-lock.json"), "{}\n");
    assert.doesNotThrow(() =>
      validateTypecheckPerformanceTarget(
        {
          ...project,
          typecheckPerformance: {
            ...project.typecheckPerformance,
            packageManager: "npm",
            lockfile: "package-lock.json",
          },
        },
        fixtureRoot,
      ),
    );
    assert.doesNotThrow(() =>
      validateTypecheckPerformanceTarget(
        { ...project, tsconfig: undefined, typecheckPerformance: { enabled: false } },
        fixtureRoot,
      ),
    );

    for (const [candidate, message] of [
      [{ ...project, typecheckPerformance: { enabled: true, compareTo: "tsc" } }, /compareTo/],
      [{ ...project, tsconfig: undefined }, /normalized relative path/],
      [{ ...project, tsconfig: "../tsconfig.json" }, /normalized relative path/],
      [{ ...project, tsconfig: "./tsconfig.json" }, /normalized relative path/],
      [{ ...project, tsconfig: "/tmp/tsconfig.json" }, /normalized relative path/],
      [{ ...project, tsconfig: "configs/missing.json" }, /does not exist/],
      [{ ...project, tsconfig: "configs" }, /is not a file/],
      [
        {
          ...project,
          typecheckPerformance: { ...project.typecheckPerformance, packageManager: "bun" },
        },
        /packageManager/,
      ],
      [
        {
          ...project,
          typecheckPerformance: { ...project.typecheckPerformance, lockfile: "yarn.lock" },
        },
        /lockfile must be pnpm-lock.yaml/,
      ],
      [
        {
          ...project,
          typecheckPerformance: {
            ...project.typecheckPerformance,
            packageManagerVersion: "latest",
          },
        },
        /exact semantic version/,
      ],
      [
        {
          ...project,
          typecheckPerformance: { ...project.typecheckPerformance, lockfile: "../pnpm-lock.yaml" },
        },
        /lockfile must be pnpm-lock.yaml/,
      ],
    ] as const) {
      assert.throws(() => validateTypecheckPerformanceTarget(candidate, fixtureRoot), message);
    }
    fs.rmSync(path.join(fixtureRoot, "pnpm-lock.yaml"));
    assert.throws(() => validateTypecheckPerformanceTarget(project, fixtureRoot), /does not exist/);
    fs.mkdirSync(path.join(fixtureRoot, "pnpm-lock.yaml"));
    assert.throws(() => validateTypecheckPerformanceTarget(project, fixtureRoot), /is not a file/);
  } finally {
    fs.rmSync(fixtureRoot, { recursive: true, force: true });
  }
});
