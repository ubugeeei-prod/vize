import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueVueI18nPackages } from "../../tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * Elk's components lose vue-i18n augmentations when TypeScript climbs into
 * Vize's `vue-i18n`, which then loads Vize's Vue beside the fixture (#4461).
 * Unique isolation links the fixture copy when an ancestor copy is reachable.
 */

function scaffold(names: string[] = ["vue-i18n"]) {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-vue-i18n-")));
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

test("an ancestor vue-i18n with exactly one in-fixture copy is linked from that copy", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue-i18n@11.1.11", "vue-i18n");
    assert.deepEqual(isolateUniqueVueI18nPackages(fixtureRoot), [
      { name: "vue-i18n", target: "node_modules/.pnpm/vue-i18n@11.1.11/node_modules/vue-i18n" },
    ]);
    assert.equal(
      fs.realpathSync(path.join(fixtureRoot, "node_modules", "vue-i18n")),
      fs.realpathSync(
        path.join(
          fixtureRoot,
          "node_modules",
          ".pnpm",
          "vue-i18n@11.1.11",
          "node_modules",
          "vue-i18n",
        ),
      ),
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a vue-i18n no ancestor provides is left alone", () => {
  const { fixtureRoot, outer } = scaffold([]);
  try {
    writeStoreCopy(fixtureRoot, "vue-i18n@11.1.11", "vue-i18n");
    assert.deepEqual(isolateUniqueVueI18nPackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-i18n")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("several vue-i18n copies whose Vue peers miss a unique match stay unlinked", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue-i18n@11.1.11_vue@3.5.13", "vue-i18n");
    writeStoreCopy(fixtureRoot, "vue-i18n@11.1.11_vue@3.4.38", "vue-i18n");
    assert.deepEqual(isolateUniqueVueI18nPackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-i18n")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("several vue-i18n copies collapse to the one whose Vue peer matches the fixture", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeFixtureVue(fixtureRoot, "3.5.30");
    writeStoreCopy(fixtureRoot, "vue-i18n@11.1.11_vue@3.5.30", "vue-i18n");
    writeStoreCopy(fixtureRoot, "vue-i18n@11.1.11_vue@3.5.13", "vue-i18n");
    assert.deepEqual(isolateUniqueVueI18nPackages(fixtureRoot), [
      {
        name: "vue-i18n",
        target: "node_modules/.pnpm/vue-i18n@11.1.11_vue@3.5.30/node_modules/vue-i18n",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an already hoisted vue-i18n is left for isolation; this repair does not relink it", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue-i18n@11.1.11", "vue-i18n");
    const hoisted = path.join(fixtureRoot, "node_modules", "vue-i18n");
    fs.mkdirSync(hoisted, { recursive: true });
    fs.writeFileSync(path.join(hoisted, "package.json"), `{"name":"vue-i18n"}\n`);
    assert.deepEqual(isolateUniqueVueI18nPackages(fixtureRoot), []);
    assert.equal(fs.lstatSync(hoisted).isSymbolicLink(), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
