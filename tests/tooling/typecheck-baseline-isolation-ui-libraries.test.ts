import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueUiLibraryPackages } from "../../tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * PrimeVue, Reka UI, and Nuxt UI live in Vize's `tests/package.json`.
 * TypeScript can climb into those copies and load Vize's Vue beside the
 * fixture (#4461). Unique isolation links the fixture store copy when an
 * ancestor is reachable.
 */

function scaffold(names: string[]) {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-ui-libraries-")),
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

test("ancestor UI libraries with one in-fixture copy each are linked from those copies", () => {
  const { fixtureRoot, outer } = scaffold(["@nuxt/ui", "primevue", "reka-ui"]);
  try {
    writeStoreCopy(fixtureRoot, "@nuxt+ui@4.8.2", "@nuxt/ui");
    writeStoreCopy(fixtureRoot, "primevue@4.5.5", "primevue");
    writeStoreCopy(fixtureRoot, "reka-ui@2.9.10", "reka-ui");
    assert.deepEqual(isolateUniqueUiLibraryPackages(fixtureRoot), [
      {
        name: "@nuxt/ui",
        target: "node_modules/.pnpm/@nuxt+ui@4.8.2/node_modules/@nuxt/ui",
      },
      { name: "primevue", target: "node_modules/.pnpm/primevue@4.5.5/node_modules/primevue" },
      { name: "reka-ui", target: "node_modules/.pnpm/reka-ui@2.9.10/node_modules/reka-ui" },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("UI libraries no ancestor provides are left alone", () => {
  const { fixtureRoot, outer } = scaffold([]);
  try {
    writeStoreCopy(fixtureRoot, "primevue@4.5.5", "primevue");
    writeStoreCopy(fixtureRoot, "reka-ui@2.9.10", "reka-ui");
    assert.deepEqual(isolateUniqueUiLibraryPackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "primevue")), false);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "reka-ui")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an already hoisted primevue is left for isolation; this repair does not relink it", () => {
  const { fixtureRoot, outer } = scaffold(["primevue"]);
  try {
    writeStoreCopy(fixtureRoot, "primevue@4.5.5", "primevue");
    const hoisted = path.join(fixtureRoot, "node_modules", "primevue");
    fs.mkdirSync(hoisted, { recursive: true });
    fs.writeFileSync(path.join(hoisted, "package.json"), `{"name":"primevue"}\n`);
    assert.deepEqual(isolateUniqueUiLibraryPackages(fixtureRoot), []);
    assert.equal(fs.lstatSync(hoisted).isSymbolicLink(), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
