import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { validateTypecheckPerformanceTarget } from "../../tools/support/compat/fixtures/tool-matrix-typecheck-target.ts";
import {
  typecheckUnits,
  typecheckUnitTsconfigs,
} from "../../tools/support/compat/fixtures/typecheck-targets.ts";
import { isolationTsconfigPaths } from "../../tools/support/compat/fixtures/typecheck-dependency-prepare.mjs";

const performance = {
  enabled: true,
  compareTo: "vue-tsc",
  packageManager: "pnpm",
  packageManagerVersion: "10.0.0",
  lockfile: "pnpm-lock.yaml",
  hangTimeoutMs: 1000,
  maxFalsePositiveRatio: 0.05,
  maxFalseNegativeRatio: 0.05,
};

function targeted() {
  return {
    id: "suite",
    vueGlobs: ["packages/lib/**/*.vue", "packages/docs/**/*.vue"],
    tsconfig: "packages/lib/tsconfig.json",
    typecheckPerformance: {
      ...performance,
      targets: [
        {
          id: "lib",
          tsconfig: "packages/lib/tsconfig.json",
          corpusGlobs: ["packages/lib/**/*.vue"],
        },
        {
          id: "docs",
          tsconfig: "packages/docs/tsconfig.json",
          corpusGlobs: ["packages/docs/**/*.vue"],
          configurationOnly: { reason: "generated docs data", issue: 1 },
        },
      ],
    },
  };
}

test("typecheck targets become units with their own config and corpus", () => {
  const units = typecheckUnits(targeted());
  // Units carry the per-target performance shape, not the registry shape.
  const unitPerformance = (index: number) =>
    units[index].typecheckPerformance as unknown as Record<string, unknown>;
  assert.deepEqual(
    units.map((unit, index) => [unit.id, unit.tsconfig, unitPerformance(index).corpusGlobs]),
    [
      ["suite", "packages/lib/tsconfig.json", ["packages/lib/**/*.vue"]],
      ["suite@docs", "packages/docs/tsconfig.json", ["packages/docs/**/*.vue"]],
    ],
  );
  // Compiler, lint and LSP keep the project's own globs.
  assert.deepEqual(units[1].vueGlobs, targeted().vueGlobs);
  assert.equal("targets" in unitPerformance(1), false);
  assert.equal("configurationOnly" in unitPerformance(0), false);
  assert.deepEqual(unitPerformance(1).configurationOnly, {
    reason: "generated docs data",
    issue: 1,
  });
  assert.deepEqual(typecheckUnitTsconfigs(targeted()), [
    "packages/lib/tsconfig.json",
    "packages/docs/tsconfig.json",
  ]);
  assert.deepEqual(isolationTsconfigPaths(targeted()), typecheckUnitTsconfigs(targeted()));

  const plain = { id: "plain", tsconfig: "tsconfig.json", typecheckPerformance: performance };
  assert.deepEqual(typecheckUnits(plain), [plain]);
});

test("a shared baseline prepare gives every unit its own baseline config", () => {
  const project = targeted();
  const withBaseline = {
    ...project,
    typecheckPerformance: {
      ...project.typecheckPerformance,
      baseline: { prepare: ["pnpm", "exec", "nuxt", "prepare"] },
      targets: [
        { ...project.typecheckPerformance.targets[0], baselineTsconfig: ".nuxt/tsconfig.json" },
        project.typecheckPerformance.targets[1],
      ],
    },
  };
  assert.deepEqual(
    typecheckUnits(withBaseline).map((unit) => unit.typecheckPerformance.baseline),
    [
      { prepare: ["pnpm", "exec", "nuxt", "prepare"], tsconfig: ".nuxt/tsconfig.json" },
      { prepare: ["pnpm", "exec", "nuxt", "prepare"], tsconfig: "packages/docs/tsconfig.json" },
    ],
  );
});

test("typecheck targets are validated before their units", () => {
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typecheck-targets-"));
  try {
    for (const config of ["packages/lib/tsconfig.json", "packages/docs/tsconfig.json"]) {
      fs.mkdirSync(path.dirname(path.join(fixtureRoot, config)), { recursive: true });
      fs.writeFileSync(path.join(fixtureRoot, config), "{}\n");
    }
    fs.writeFileSync(path.join(fixtureRoot, "pnpm-lock.yaml"), "lockfileVersion: '9.0'\n");
    assert.doesNotThrow(() => validateTypecheckPerformanceTarget(targeted(), fixtureRoot));

    const mutate = (change: (project: ReturnType<typeof targeted>) => void) => {
      const project = targeted();
      change(project);
      return project;
    };
    for (const [project, message] of [
      [mutate((p) => (p.tsconfig = "packages/docs/tsconfig.json")), /first target's tsconfig/],
      [
        mutate((p) => Object.assign(p.typecheckPerformance, { corpusGlobs: ["**/*.vue"] })),
        /corpusGlobs belongs to each target/,
      ],
      [mutate((p) => (p.typecheckPerformance.targets[1].id = "lib")), /duplicate target id lib/],
      [mutate((p) => (p.typecheckPerformance.targets[1].id = "Docs")), /lowercase slug/],
      [
        mutate((p) => Object.assign(p.typecheckPerformance.targets[1], { extra: true })),
        /unknown fields/,
      ],
      [
        mutate(
          (p) => (p.typecheckPerformance.targets[1].configurationOnly = { reason: "", issue: 1 }),
        ),
        /needs a reason and a tracking issue/,
      ],
      [
        mutate((p) => (p.typecheckPerformance.targets[1].tsconfig = "packages/missing.json")),
        /tsconfig does not exist/,
      ],
      [mutate((p) => (p.typecheckPerformance.targets = [])), /non-empty array/],
    ] as const) {
      assert.throws(() => validateTypecheckPerformanceTarget(project, fixtureRoot), message);
    }
  } finally {
    fs.rmSync(fixtureRoot, { recursive: true, force: true });
  }
});
