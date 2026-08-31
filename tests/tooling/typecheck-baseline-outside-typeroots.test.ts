import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  rewriteOutsideTypeRoots,
  writeIsolatedTsconfigOverlay,
} from "../../legacy-tools/fixtures/typecheck-baseline-outside-paths.mjs";

/**
 * Unique isolation cannot retarget `compilerOptions.typeRoots`. An outside
 * `node_modules/@types` lets TypeScript load Vize's ambient packages beside
 * the fixture's Vue (#4461). Overlay rewrite is the repair.
 */

function scaffold() {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-typeroots-")));
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsideTypes = path.join(outer, "node_modules", "@types");
  fs.mkdirSync(outsideTypes, { recursive: true });
  return { fixtureRoot, outer, outsideTypes };
}

function writeLocalTypes(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "@types");
  fs.mkdirSync(local, { recursive: true });
  return local;
}

test("an outside node_modules/@types typeRoot is retargeted to the fixture copy", () => {
  const { fixtureRoot, outer, outsideTypes } = scaffold();
  try {
    writeLocalTypes(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { typeRoots: [path.relative(fixtureRoot, outsideTypes)] },
      })}\n`,
    );
    const configDir = path.join(fixtureRoot, ".vize-baseline");
    assert.deepEqual(rewriteOutsideTypeRoots(fixtureRoot, sourcePath, configDir), [
      "../node_modules/@types",
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("typeRoots reached only through relative extends are still retargeted", () => {
  const { fixtureRoot, outer, outsideTypes } = scaffold();
  try {
    writeLocalTypes(fixtureRoot);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.app.json"),
      `${JSON.stringify({
        compilerOptions: { typeRoots: [path.relative(fixtureRoot, outsideTypes)] },
      })}\n`,
    );
    const sourcePath = path.join(fixtureRoot, "tsconfig.check.json");
    fs.writeFileSync(sourcePath, `{ "extends": "./tsconfig.app.json" }\n`);
    const overlay = writeIsolatedTsconfigOverlay(fixtureRoot, sourcePath);
    assert.equal(overlay.path, path.join(fixtureRoot, ".vize-isolated-tsconfig.check.json"));
    assert.deepEqual(JSON.parse(fs.readFileSync(overlay.path, "utf8")), {
      extends: "./tsconfig.check.json",
      compilerOptions: { typeRoots: ["./node_modules/@types"] },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an inside typeRoot is left for inheritance", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    const typings = path.join(fixtureRoot, "typings");
    fs.mkdirSync(typings);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({ compilerOptions: { typeRoots: ["./typings"] } })}\n`,
    );
    assert.equal(rewriteOutsideTypeRoots(fixtureRoot, sourcePath, fixtureRoot), null);
    assert.equal(writeIsolatedTsconfigOverlay(fixtureRoot, sourcePath), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside typeRoot is left alone when the fixture has no local @types", () => {
  const { fixtureRoot, outer, outsideTypes } = scaffold();
  try {
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { typeRoots: [path.relative(fixtureRoot, outsideTypes)] },
      })}\n`,
    );
    assert.equal(rewriteOutsideTypeRoots(fixtureRoot, sourcePath, fixtureRoot), null);
    assert.equal(writeIsolatedTsconfigOverlay(fixtureRoot, sourcePath), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a custom outside typeRoot that is not node_modules/@types is not guessed", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocalTypes(fixtureRoot);
    const outsideTypings = path.join(outer, "typings");
    fs.mkdirSync(outsideTypings);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { typeRoots: [path.relative(fixtureRoot, outsideTypings)] },
      })}\n`,
    );
    assert.equal(rewriteOutsideTypeRoots(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
