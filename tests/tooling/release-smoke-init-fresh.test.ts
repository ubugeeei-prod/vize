import assert from "node:assert/strict";
import { test } from "node:test";

import {
  FRESH_INIT_MATRIX,
  PACKAGE_MANAGERS,
  PROJECT_SHAPES,
} from "../../tools/npm/smoke-release-init-shapes.mjs";
import { readRepoFile } from "./support/github-workflows.ts";

const SHAPE_KEYS = [
  "addedScripts",
  "createdFiles",
  "detection",
  "expectedDevDependencies",
  "expectedFiles",
  "expectedScripts",
  "features",
  "initFlags",
  "plannedDependencies",
  "reconfiguredDetection",
  "reconfiguredFeatures",
  "requires",
  "updatedFiles",
];

/** Code-unit order, matching the driver's own comparison of installed names. */
const byCodeUnit = (left: string, right: string): number =>
  left < right ? -1 : left > right ? 1 : 0;

const MANAGER_KEYS = [
  "bootstrapArgs",
  "installArgs",
  "installFlags",
  "lockfile",
  "redirect",
  "runScriptArgs",
];

test("the fresh-project matrix is data, so new cells need no driver change", () => {
  assert.ok(FRESH_INIT_MATRIX.length > 0, "the fresh-project matrix must run at least one cell");
  for (const cell of FRESH_INIT_MATRIX) {
    const manager = PACKAGE_MANAGERS[cell.packageManager];
    const shape = PROJECT_SHAPES[cell.shape];
    assert.ok(manager, `unknown package manager ${cell.packageManager}`);
    assert.ok(shape, `unknown project shape ${cell.shape}`);
    for (const key of MANAGER_KEYS) {
      assert.ok(key in manager, `${cell.packageManager} is missing ${key}`);
    }
    for (const key of SHAPE_KEYS) {
      assert.ok(key in shape, `${cell.shape} is missing ${key}`);
    }
    // The plan the smoke asserts must be the plan it installs, and the install
    // must leave the project declaring nothing beyond it.
    for (const name of shape.plannedDependencies) {
      assert.ok(
        shape.expectedDevDependencies.includes(name),
        `${cell.shape} plans ${name} but does not expect it in devDependencies`,
      );
      assert.ok(shape.requires.includes(name), `${cell.shape} plans unpacked ${name}`);
    }
    assert.deepEqual(
      [...shape.expectedDevDependencies].sort(byCodeUnit),
      shape.expectedDevDependencies,
      `${cell.shape} devDependency expectation must be sorted for a stable comparison`,
    );
  }
});

test("every shape drives a clean, broken, and repaired check", () => {
  for (const shape of Object.values(PROJECT_SHAPES)) {
    const broken = Object.keys(shape.check.broken);
    assert.ok(broken.length > 0, `${shape.id} has no broken variant`);
    const authored = Object.keys(shape.files({ typescript: "0", vite: "0", vue: "0" }));
    for (const name of broken) {
      assert.ok(authored.includes(name), `${shape.id} breaks ${name}, which it never authored`);
    }
    const reported = shape.check.brokenDiagnostics.map((entry) => entry.file);
    assert.deepEqual(reported, broken, `${shape.id} must assert every broken file's diagnostics`);
    for (const entry of shape.check.brokenDiagnostics) {
      assert.ok(
        entry.diagnostics.length > 0,
        `${shape.id} expects no diagnostics for ${entry.file}`,
      );
      for (const diagnostic of entry.diagnostics) {
        // Full authored position and message, never a code-only assertion.
        assert.match(diagnostic, /^error:\d+:\d+ \[TS\d+\] .+\.$/u);
      }
    }
  }
});

test("the smoke only passes init flags the guide documents", () => {
  const guide = readRepoFile("docs", "content", "guide", "init.md");
  const documented = new Set([...guide.matchAll(/`(--?[a-z-]+)`/gu)].map((match) => match[1]));
  // Added by the driver itself around the shape's own flag set.
  for (const flag of ["--dry-run", "--no-install"]) {
    assert.ok(documented.has(flag), `docs/content/guide/init.md must document ${flag}`);
  }
  for (const shape of Object.values(PROJECT_SHAPES)) {
    for (const flag of shape.initFlags) {
      assert.ok(documented.has(flag), `${shape.id} passes undocumented init flag ${flag}`);
    }
  }
  // The idempotent run the smoke asserts is the one the guide prints verbatim.
  assert.ok(guide.includes("[vize init] nothing to do; the project is already configured"));
});

test("the release runtime smoke runs the fresh-project matrix", () => {
  const runtime = readRepoFile("tools", "npm", "smoke-release-runtime.mjs");
  assert.match(runtime, /runFreshProjectInitChecks\(\{[\s\S]*?tempDir,[\s\S]*?vizeBin,\n\s*\}\);/u);
  assert.match(runtime, /from "\.\/smoke-release-init-fresh\.mjs"/u);

  const project = readRepoFile("tools", "npm", "smoke-release-init-project.mjs");
  // The isolation contract: outside the install tree, outside the checkout, and
  // no ancestor that could resolve `vize` for the project.
  assert.match(project, /is inside the install tree/u);
  assert.match(project, /is inside the Vize checkout/u);
  assert.match(project, /would leak into the fresh project/u);
  assert.match(project, /installed vize did not bring @typescript\/native-preview/u);
  assert.match(project, /a missing Corsa runtime silently disabled type checking/u);
  // Host Corsa overrides must be stripped, or a packaging failure stays hidden.
  assert.match(project, /CORSA_PATH", "CORSA_EXECUTABLE", "TSGO_PATH", "TSGO_EXECUTABLE/u);

  for (const workflow of ["release.yml", "native-smoke.yml"]) {
    const source = readRepoFile(".github", "workflows", workflow);
    assert.match(
      source,
      /smoke-release-install\.mjs --prepare-manifests --runtime-checks[\s\S]*?npm\/cli/u,
      `${workflow} must run the runtime smoke over the packed CLI`,
    );
  }
});
