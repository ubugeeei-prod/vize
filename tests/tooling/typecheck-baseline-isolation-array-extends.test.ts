import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateFixtureTypePackages } from "../../tools/fixtures/typecheck-baseline-isolation.mjs";

/**
 * TypeScript array `extends` walks every entry; later files win. Isolation used
 * to follow only the first specifier, so a Nuxt-style
 * `extends: ["./empty.json", "./tsconfig.app.json"]` never saw the app `paths`
 * and `vue-router` escaped to Vize (#4461).
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-array-extends-")),
  );
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

test("a later array extends entry still isolates declared package paths", () => {
  const { outer, fixtureRoot, storeId } = scaffold();
  try {
    const nuxtDir = path.join(fixtureRoot, ".nuxt");
    fs.mkdirSync(nuxtDir, { recursive: true });
    fs.writeFileSync(path.join(nuxtDir, "empty.json"), "{}\n");
    fs.writeFileSync(
      path.join(nuxtDir, "tsconfig.app.json"),
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "vue-router": [`../node_modules/.pnpm/${storeId}/node_modules/vue-router`],
          },
        },
      })}\n`,
    );
    const configPath = path.join(nuxtDir, "tsconfig.json");
    fs.writeFileSync(
      configPath,
      `${JSON.stringify({ extends: ["./empty.json", "./tsconfig.app.json"] })}\n`,
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

test("an earlier array extends entry does not hide a later paths mapping", () => {
  const { outer, fixtureRoot, storeId } = scaffold();
  try {
    const nuxtDir = path.join(fixtureRoot, ".nuxt");
    fs.mkdirSync(nuxtDir, { recursive: true });
    fs.writeFileSync(
      path.join(nuxtDir, "stale.json"),
      `${JSON.stringify({
        compilerOptions: { paths: { "vue-router": ["./missing-vue-router"] } },
      })}\n`,
    );
    fs.writeFileSync(
      path.join(nuxtDir, "tsconfig.app.json"),
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "vue-router": [`../node_modules/.pnpm/${storeId}/node_modules/vue-router`],
          },
        },
      })}\n`,
    );
    const configPath = path.join(nuxtDir, "tsconfig.json");
    fs.writeFileSync(
      configPath,
      `${JSON.stringify({ extends: ["./stale.json", "./tsconfig.app.json"] })}\n`,
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
