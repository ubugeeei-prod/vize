import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  toolArgs,
  typecheckSourceTsconfig,
  typecheckTsconfigPath,
} from "../../legacy-tools/fixtures/tool-matrix-command.mjs";
import { isolatedTsconfigOverlayPath } from "../../legacy-tools/fixtures/typecheck-baseline-outside-paths.mjs";

/**
 * Elk's root tsconfig is a Nuxt 4 solution (`files: []` + `references`). vue-tsc
 * already measures `.nuxt/tsconfig.app.json`. Vize must use that same program,
 * and the isolated overlay of that file, not the empty solution (#4461).
 */

const elk = {
  vueGlobs: ["app/**/*.vue"],
  tsconfig: "tsconfig.json",
  typecheckPerformance: { baseline: { tsconfig: ".nuxt/tsconfig.app.json" } },
};

test("typecheck uses the baseline tsconfig when it differs from the root config", () => {
  assert.equal(typecheckSourceTsconfig(elk), ".nuxt/tsconfig.app.json");
  assert.equal(typecheckTsconfigPath(elk), ".nuxt/tsconfig.app.json");
  assert.deepEqual(toolArgs(elk, "typechecker", "out"), [
    "check",
    "app/**/*.vue",
    "--format",
    "json",
    "--no-config",
    "--tsconfig",
    ".nuxt/tsconfig.app.json",
  ]);
});

test("typecheck falls back to the root tsconfig when no baseline is pinned", () => {
  const project = { vueGlobs: ["src/**/*.vue"], tsconfig: "tsconfig.json" };
  assert.equal(typecheckSourceTsconfig(project), "tsconfig.json");
  assert.equal(typecheckTsconfigPath(project), "tsconfig.json");
});

test("typecheck prefers the isolated overlay of the baseline tsconfig", () => {
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typecheck-tsconfig-"));
  try {
    const overlayRel = isolatedTsconfigOverlayPath(".nuxt/tsconfig.app.json");
    fs.mkdirSync(path.join(fixtureRoot, path.dirname(overlayRel)), { recursive: true });
    fs.writeFileSync(path.join(fixtureRoot, overlayRel), "{}\n");
    const project = { ...elk, fixturePath: fixtureRoot };
    assert.equal(typecheckTsconfigPath(project), overlayRel);
    assert.deepEqual(toolArgs(project, "typechecker", "out"), [
      "check",
      "app/**/*.vue",
      "--format",
      "json",
      "--no-config",
      "--tsconfig",
      overlayRel,
    ]);
  } finally {
    fs.rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test("typecheck does not use the root overlay when a baseline tsconfig is pinned", () => {
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typecheck-tsconfig-root-"));
  try {
    const rootOverlay = isolatedTsconfigOverlayPath("tsconfig.json");
    fs.writeFileSync(path.join(fixtureRoot, rootOverlay), "{}\n");
    assert.equal(
      typecheckTsconfigPath({ ...elk, fixturePath: fixtureRoot }),
      ".nuxt/tsconfig.app.json",
    );
  } finally {
    fs.rmSync(fixtureRoot, { recursive: true, force: true });
  }
});
