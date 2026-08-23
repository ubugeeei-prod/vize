import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { rewriteOutsidePackagePaths } from "../../tools/fixtures/typecheck-baseline-outside-paths.mjs";

/**
 * A `paths` mapping named `*` whose target is an outside `node_modules`
 * directory still loads Vize's Vue (#4461). Overlay retargets that catch-all
 * to the fixture copy. Other outside directories are not guessed.
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-outside-catchall-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsideModules = path.join(outer, "node_modules");
  fs.mkdirSync(outsideModules, { recursive: true });
  return { fixtureRoot, outer, outsideModules };
}

test("an outside node_modules catch-all is retargeted to the fixture copy", () => {
  const { fixtureRoot, outer, outsideModules } = scaffold();
  try {
    fs.mkdirSync(path.join(fixtureRoot, "node_modules"), { recursive: true });
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: { "*": [`${path.relative(fixtureRoot, outsideModules)}/*`] },
        },
      })}\n`,
    );
    assert.deepEqual(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { "*": ["../node_modules/*"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside catch-all with no fixture node_modules is left inherited", () => {
  const { fixtureRoot, outer, outsideModules } = scaffold();
  try {
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: { "*": [`${path.relative(fixtureRoot, outsideModules)}/*`] },
        },
      })}\n`,
    );
    assert.equal(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      null,
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside catch-all that is not node_modules is not guessed", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    fs.mkdirSync(path.join(fixtureRoot, "node_modules"), { recursive: true });
    const outsideSrc = path.join(outer, "src");
    fs.mkdirSync(outsideSrc, { recursive: true });
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: { "*": [`${path.relative(fixtureRoot, outsideSrc)}/*`] },
        },
      })}\n`,
    );
    assert.equal(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      null,
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an inside node_modules catch-all is left inherited", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    fs.mkdirSync(path.join(fixtureRoot, "node_modules"), { recursive: true });
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: { "*": ["./node_modules/*"] },
        },
      })}\n`,
    );
    assert.equal(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      null,
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
