import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateFixtureTypePackages } from "../../tools/fixtures/typecheck-baseline-isolation.mjs";

/**
 * Nuxt writes `baseUrl: ".."` on `.nuxt/tsconfig.json` and `paths` relative to
 * that directory. Isolation used to resolve those targets from `.nuxt`, so the
 * fixture store copy looked missing and `vue-router` escaped to Vize (#4461).
 */

function scaffold() {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-baseurl-")));
  const fixtureRoot = path.join(outer, "fixture");
  const storeId = "vue-router@5.1.0";
  const storeCopy = path.join(
    fixtureRoot,
    "node_modules",
    ".pnpm",
    storeId,
    "node_modules",
    "vue-router",
  );
  fs.mkdirSync(storeCopy, { recursive: true });
  fs.writeFileSync(path.join(storeCopy, "package.json"), '{"name":"vue-router"}\n');
  const ancestor = path.join(outer, "node_modules", "vue-router");
  fs.mkdirSync(ancestor, { recursive: true });
  fs.writeFileSync(path.join(ancestor, "package.json"), '{"name":"vue-router"}\n');
  return { fixtureRoot, outer, storeId };
}

test("a Nuxt baseUrl resolves declared package paths from the fixture root", () => {
  const { outer, fixtureRoot, storeId } = scaffold();
  try {
    const nuxtDir = path.join(fixtureRoot, ".nuxt");
    fs.mkdirSync(nuxtDir, { recursive: true });
    const configPath = path.join(nuxtDir, "tsconfig.json");
    fs.writeFileSync(
      configPath,
      `${JSON.stringify({
        compilerOptions: {
          baseUrl: "..",
          paths: {
            "vue-router": [`./node_modules/.pnpm/${storeId}/node_modules/vue-router`],
          },
        },
      })}\n`,
    );
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, configPath), [
      {
        name: "vue-router",
        target: `node_modules/.pnpm/${storeId}/node_modules/vue-router`,
      },
    ]);
    assert.equal(
      fs.realpathSync(path.join(fixtureRoot, "node_modules", "vue-router")),
      fs.realpathSync(
        path.join(fixtureRoot, "node_modules", ".pnpm", storeId, "node_modules", "vue-router"),
      ),
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("paths without a baseUrl still resolve from the declaring tsconfig directory", () => {
  const { outer, fixtureRoot, storeId } = scaffold();
  try {
    const nuxtDir = path.join(fixtureRoot, ".nuxt");
    fs.mkdirSync(nuxtDir, { recursive: true });
    const configPath = path.join(nuxtDir, "tsconfig.json");
    fs.writeFileSync(
      configPath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "vue-router": [`../node_modules/.pnpm/${storeId}/node_modules/vue-router`],
          },
        },
      })}\n`,
    );
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, configPath), [
      {
        name: "vue-router",
        target: `node_modules/.pnpm/${storeId}/node_modules/vue-router`,
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
