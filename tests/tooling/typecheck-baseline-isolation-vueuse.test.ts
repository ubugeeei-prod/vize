import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueVueUsePackages } from "../../legacy-tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * Elk loses @vueuse augmentations when TypeScript climbs into Vize's
 * `@vueuse/core`, which then loads Vize's Vue beside the fixture (#4461).
 * Unique isolation links the fixture copy when an ancestor copy is reachable.
 */

function scaffold(names: string[] = ["@vueuse/core"]) {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-vueuse-")));
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

test("an ancestor @vueuse/core with exactly one in-fixture copy is linked from that copy", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "@vueuse+core@13.1.0", "@vueuse/core");
    assert.deepEqual(isolateUniqueVueUsePackages(fixtureRoot), [
      {
        name: "@vueuse/core",
        target: "node_modules/.pnpm/@vueuse+core@13.1.0/node_modules/@vueuse/core",
      },
    ]);
    assert.equal(
      fs.realpathSync(path.join(fixtureRoot, "node_modules", "@vueuse", "core")),
      fs.realpathSync(
        path.join(
          fixtureRoot,
          "node_modules",
          ".pnpm",
          "@vueuse+core@13.1.0",
          "node_modules",
          "@vueuse",
          "core",
        ),
      ),
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a @vueuse/core no ancestor provides is left alone", () => {
  const { fixtureRoot, outer } = scaffold([]);
  try {
    writeStoreCopy(fixtureRoot, "@vueuse+core@13.1.0", "@vueuse/core");
    assert.deepEqual(isolateUniqueVueUsePackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "@vueuse", "core")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("several @vueuse/core copies whose Vue peers miss a unique match stay unlinked", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "@vueuse+core@13.1.0_vue@3.5.13", "@vueuse/core");
    writeStoreCopy(fixtureRoot, "@vueuse+core@13.1.0_vue@3.4.38", "@vueuse/core");
    assert.deepEqual(isolateUniqueVueUsePackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "@vueuse", "core")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("several @vueuse/core copies collapse to the one whose Vue peer matches the fixture", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeFixtureVue(fixtureRoot, "3.5.30");
    writeStoreCopy(fixtureRoot, "@vueuse+core@13.1.0_vue@3.5.30", "@vueuse/core");
    writeStoreCopy(fixtureRoot, "@vueuse+core@13.1.0_vue@3.5.13", "@vueuse/core");
    assert.deepEqual(isolateUniqueVueUsePackages(fixtureRoot), [
      {
        name: "@vueuse/core",
        target: "node_modules/.pnpm/@vueuse+core@13.1.0_vue@3.5.30/node_modules/@vueuse/core",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an already hoisted @vueuse/core is left for isolation; this repair does not relink it", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "@vueuse+core@13.1.0", "@vueuse/core");
    const hoisted = path.join(fixtureRoot, "node_modules", "@vueuse", "core");
    fs.mkdirSync(hoisted, { recursive: true });
    fs.writeFileSync(path.join(hoisted, "package.json"), `{"name":"@vueuse/core"}\n`);
    assert.deepEqual(isolateUniqueVueUsePackages(fixtureRoot), []);
    assert.equal(fs.lstatSync(hoisted).isSymbolicLink(), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
