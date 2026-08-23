import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueNuxtUiPackages } from "../../tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * Elk and Nuxt UI apps lose `@nuxt/ui` augmentations when TypeScript climbs
 * into Vize's copy, which then loads Vize's Vue beside the fixture (#4461).
 * Unique isolation links the fixture copy when an ancestor copy is reachable.
 */

function scaffold(names: string[] = ["@nuxt/ui"]) {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-nuxt-ui-")));
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

test("an ancestor @nuxt/ui with exactly one in-fixture copy is linked from that copy", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "@nuxt+ui@4.8.2", "@nuxt/ui");
    assert.deepEqual(isolateUniqueNuxtUiPackages(fixtureRoot), [
      {
        name: "@nuxt/ui",
        target: "node_modules/.pnpm/@nuxt+ui@4.8.2/node_modules/@nuxt/ui",
      },
    ]);
    assert.equal(
      fs.realpathSync(path.join(fixtureRoot, "node_modules", "@nuxt", "ui")),
      fs.realpathSync(
        path.join(
          fixtureRoot,
          "node_modules",
          ".pnpm",
          "@nuxt+ui@4.8.2",
          "node_modules",
          "@nuxt",
          "ui",
        ),
      ),
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a @nuxt/ui no ancestor provides is left alone", () => {
  const { fixtureRoot, outer } = scaffold([]);
  try {
    writeStoreCopy(fixtureRoot, "@nuxt+ui@4.8.2", "@nuxt/ui");
    assert.deepEqual(isolateUniqueNuxtUiPackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "@nuxt", "ui")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an already hoisted @nuxt/ui is left for isolation; this repair does not relink it", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "@nuxt+ui@4.8.2", "@nuxt/ui");
    const hoisted = path.join(fixtureRoot, "node_modules", "@nuxt", "ui");
    fs.mkdirSync(hoisted, { recursive: true });
    fs.writeFileSync(path.join(hoisted, "package.json"), `{"name":"@nuxt/ui"}\n`);
    assert.deepEqual(isolateUniqueNuxtUiPackages(fixtureRoot), []);
    assert.equal(fs.lstatSync(hoisted).isSymbolicLink(), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
