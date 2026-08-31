/**
 * Typechecker-performance half of the Vue ecosystem fixture registry contract.
 *
 * Kept separate from `vue-ecosystem-fixtures.test.ts` so neither file grows past
 * the repository source-length limit as fixtures are added.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registryPath = path.join(root, "tests", "_fixtures", "vue-ecosystem-fixtures.json");

interface TypecheckPerformance {
  enabled: boolean;
  compareTo: string;
  packageManager: "npm" | "pnpm" | "yarn";
  packageManagerVersion: string;
  lockfile: "pnpm-lock.yaml" | "yarn.lock";
  baseline?: { tsconfig: string; prepare?: string[] };
  hangTimeoutMs: number;
  corpusGlobs?: string[];
  maxFalsePositiveRatio: number;
  maxFalseNegativeRatio: number;
  largeProjectRegressionTarget?: boolean;
}

interface FixtureProject {
  id: string;
  fixturePath?: string;
  tsconfig?: string;
  typecheckPerformance?: TypecheckPerformance;
}

const requiredTypecheckProjects = ["voicevox", "elk", "misskey"] as const;

function readRegistry(): { projects: FixtureProject[] } {
  return JSON.parse(fs.readFileSync(registryPath, "utf8")) as { projects: FixtureProject[] };
}

test("typecheck baselines have complete budgets and bounded release coverage", () => {
  const registry = readRegistry();
  const targets = registry.projects.filter((project) => {
    if (project.typecheckPerformance?.enabled !== true) return false;
    return true;
  });

  assert.equal(targets.length, 13);
  for (const project of targets) {
    const performance = project.typecheckPerformance!;
    assert.equal(performance.compareTo, "vue-tsc", `${project.id} baseline`);
    assert.equal(
      performance.lockfile,
      { npm: "package-lock.json", pnpm: "pnpm-lock.yaml", yarn: "yarn.lock" }[
        performance.packageManager
      ],
    );
    assert.match(performance.packageManagerVersion, /^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/);
    assert.ok(Number.isSafeInteger(performance.hangTimeoutMs));
    assert.ok(performance.hangTimeoutMs > 0 && performance.hangTimeoutMs <= 300_000);
    assert.ok(performance.maxFalsePositiveRatio >= 0 && performance.maxFalsePositiveRatio <= 1);
    assert.equal(performance.maxFalseNegativeRatio, performance.maxFalsePositiveRatio);
  }
});

test("large typechecker fixtures have performance safeguards and bench wiring", () => {
  const registry = readRegistry();
  const benchCheck = fs.readFileSync(
    path.join(root, "tools", "benchmarks", "scripts", "check.ts"),
    "utf8",
  );

  for (const id of requiredTypecheckProjects) {
    const project = registry.projects.find((candidate) => candidate.id === id);
    assert.ok(project, `${id} should be registered`);
    assert.equal(project?.typecheckPerformance?.enabled, true);
    assert.equal(project?.typecheckPerformance?.largeProjectRegressionTarget, true);
    assert.ok((project?.typecheckPerformance?.hangTimeoutMs ?? Infinity) <= 300_000);
    assert.ok((project?.typecheckPerformance?.maxFalsePositiveRatio ?? Infinity) <= 0.02);
    assert.ok((project?.typecheckPerformance?.maxFalseNegativeRatio ?? Infinity) <= 0.02);
    assert.match(
      benchCheck,
      new RegExp(`name:\\s*"${id}"`),
      `${id} should be in tools/benchmarks/scripts/check.ts`,
    );
  }
  const baseline = registry.projects.find((project) => project.id === "elk")?.typecheckPerformance
    ?.baseline;
  assert.equal(baseline?.tsconfig, ".nuxt/tsconfig.app.json");
  assert.deepEqual(baseline?.prepare, ["pnpm", "exec", "nuxt", "prepare"]);
});

test("PrimeVue Volt is measured under the app tsconfig, sharing the PrimeVue fixture", () => {
  const registry = readRegistry();
  const library = registry.projects.find((project) => project.id === "primevue");
  const volt = registry.projects.find((project) => project.id === "primevue-volt");
  assert.equal(volt?.fixturePath, library?.fixturePath);
  assert.equal(volt?.tsconfig, "apps/volt/tsconfig.json");
  assert.deepEqual(volt?.typecheckPerformance?.corpusGlobs, ["apps/volt/**/*.vue"]);
  assert.equal(volt?.typecheckPerformance?.baseline?.tsconfig, "apps/volt/.nuxt/tsconfig.json");
  assert.deepEqual(volt?.typecheckPerformance?.baseline?.prepare, [
    "pnpm",
    "--filter",
    "volt",
    "exec",
    "nuxt",
    "prepare",
  ]);
});

test("PrimeVue Showcase is measured under the app tsconfig, sharing the PrimeVue fixture", () => {
  const registry = readRegistry();
  const library = registry.projects.find((project) => project.id === "primevue");
  const showcase = registry.projects.find((project) => project.id === "primevue-showcase");
  assert.equal(showcase?.fixturePath, library?.fixturePath);
  assert.equal(showcase?.tsconfig, "apps/showcase/tsconfig.json");
  assert.deepEqual(showcase?.typecheckPerformance?.corpusGlobs, ["apps/showcase/**/*.vue"]);
  assert.equal(
    showcase?.typecheckPerformance?.baseline?.tsconfig,
    "apps/showcase/.nuxt/tsconfig.json",
  );
  assert.deepEqual(showcase?.typecheckPerformance?.baseline?.prepare, [
    "pnpm",
    "--filter",
    "showcase",
    "exec",
    "nuxt",
    "prepare",
  ]);
});

test("Lew UI typechecks under its fixture tsconfig", () => {
  const registry = readRegistry();
  const project = registry.projects.find((candidate) => candidate.id === "lew-ui");

  assert.equal(project?.tsconfig, "tsconfig.json");
});
