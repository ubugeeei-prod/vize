import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueVueRuntimePackages } from "../../tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * Nuxt's generated `nuxt.d.ts` still `/// <reference types="vue-router" />`
 * even when no tsconfig declared the package (#4461). Unique isolation links
 * the fixture copy when an ancestor `vue-router` is reachable.
 */

function scaffold(names: string[] = ["vue-router"]) {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-vue-router-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(fixtureRoot, { recursive: true });
  for (const name of names) {
    const packageRoot = path.join(outer, "node_modules", name);
    fs.mkdirSync(packageRoot, { recursive: true });
    fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
  }
  return { fixtureRoot, outer };
}

function writeStoreCopy(fixtureRoot: string, id: string, name: string) {
  const packageRoot = path.join(fixtureRoot, "node_modules", ".pnpm", id, "node_modules", name);
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
  return packageRoot;
}

function writeFixtureVue(fixtureRoot: string, version: string) {
  const packageRoot = path.join(fixtureRoot, "node_modules", "vue");
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(
    path.join(packageRoot, "package.json"),
    `{"name":"vue","version":"${version}"}\n`,
  );
}

test("an ancestor vue-router with exactly one in-fixture copy is linked from that copy", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0", "vue-router");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), [
      {
        name: "vue-router",
        target: "node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router",
      },
    ]);
    assert.equal(
      fs.realpathSync(path.join(fixtureRoot, "node_modules", "vue-router")),
      fs.realpathSync(
        path.join(
          fixtureRoot,
          "node_modules",
          ".pnpm",
          "vue-router@5.1.0",
          "node_modules",
          "vue-router",
        ),
      ),
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a vue-router no ancestor provides is left alone", () => {
  const { fixtureRoot, outer } = scaffold([]);
  try {
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0", "vue-router");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-router")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("several vue-router copies whose Vue peers miss a unique match stay unlinked", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.13", "vue-router");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.4.38", "vue-router");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-router")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("several vue-router copies collapse to the one whose Vue peer matches the fixture", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeFixtureVue(fixtureRoot, "3.5.30");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.30", "vue-router");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.13", "vue-router");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), [
      {
        name: "vue-router",
        target: "node_modules/.pnpm/vue-router@5.1.0_vue@3.5.30/node_modules/vue-router",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an already hoisted vue-router is left for isolation; this repair does not relink it", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0", "vue-router");
    const hoisted = path.join(fixtureRoot, "node_modules", "vue-router");
    fs.mkdirSync(hoisted, { recursive: true });
    fs.writeFileSync(path.join(hoisted, "package.json"), `{"name":"vue-router"}\n`);
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), []);
    assert.equal(fs.lstatSync(hoisted).isSymbolicLink(), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
