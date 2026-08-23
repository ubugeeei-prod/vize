import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { rewriteOutsidePackagePaths } from "../../tools/fixtures/typecheck-baseline-outside-paths.mjs";

/**
 * Trailing `/*` on a package `paths` mapping still loads that outside tree
 * (#4461). Overlay retargets it to the fixture copy. Interior `*` and
 * `#alias/*` keys are not guessed.
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-outside-paths-star-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsideRouter = path.join(outer, "node_modules", "vue-router");
  fs.mkdirSync(outsideRouter, { recursive: true });
  fs.writeFileSync(path.join(outsideRouter, "package.json"), `{"name":"vue-router"}\n`);
  return { outer, fixtureRoot, outsideRouter };
}

function writeLocalRouter(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "vue-router");
  fs.mkdirSync(local, { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"vue-router"}\n`);
  return local;
}

test("an outside package path with a trailing star is retargeted", () => {
  const { outer, fixtureRoot, outsideRouter } = scaffold();
  try {
    writeLocalRouter(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "vue-router": [`${path.relative(fixtureRoot, outsideRouter)}/*`],
          },
        },
      })}\n`,
    );
    assert.deepEqual(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { "vue-router": ["../node_modules/vue-router/*"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside package/* mapping is retargeted", () => {
  const { outer, fixtureRoot, outsideRouter } = scaffold();
  try {
    writeLocalRouter(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "vue-router/*": [`${path.relative(fixtureRoot, outsideRouter)}/*`],
          },
        },
      })}\n`,
    );
    assert.deepEqual(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { "vue-router/*": ["../node_modules/vue-router/*"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside hash alias with a trailing star is left inherited", () => {
  const { outer, fixtureRoot, outsideRouter } = scaffold();
  try {
    writeLocalRouter(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "#app/*": [`${path.relative(fixtureRoot, outsideRouter)}/*`],
          },
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

test("an interior star in a package path is not guessed", () => {
  const { outer, fixtureRoot, outsideRouter } = scaffold();
  try {
    writeLocalRouter(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "vue-router": [
              `${path.relative(fixtureRoot, path.dirname(outsideRouter))}/*/vue-router`,
            ],
          },
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

test("a trailing-star package with no fixture-local copy is left inherited", () => {
  const { outer, fixtureRoot, outsideRouter } = scaffold();
  try {
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "vue-router": [`${path.relative(fixtureRoot, outsideRouter)}/*`],
          },
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
