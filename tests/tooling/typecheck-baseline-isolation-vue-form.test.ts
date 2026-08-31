import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueVueFormPackages } from "../../legacy-tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * FormKit and VeeValidate live in Vize's `tests/package.json`. TypeScript can
 * climb into those copies and load Vize's Vue beside the fixture (#4461).
 * Unique isolation links the fixture store copy when an ancestor is reachable.
 */

function scaffold(names: string[]) {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-vue-form-")));
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

test("ancestor form packages with one in-fixture copy each are linked from those copies", () => {
  const { fixtureRoot, outer } = scaffold(["@formkit/vue", "vee-validate"]);
  try {
    writeStoreCopy(fixtureRoot, "@formkit+vue@2.1.0", "@formkit/vue");
    writeStoreCopy(fixtureRoot, "vee-validate@4.15.1", "vee-validate");
    assert.deepEqual(isolateUniqueVueFormPackages(fixtureRoot), [
      {
        name: "@formkit/vue",
        target: "node_modules/.pnpm/@formkit+vue@2.1.0/node_modules/@formkit/vue",
      },
      {
        name: "vee-validate",
        target: "node_modules/.pnpm/vee-validate@4.15.1/node_modules/vee-validate",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("form packages no ancestor provides are left alone", () => {
  const { fixtureRoot, outer } = scaffold([]);
  try {
    writeStoreCopy(fixtureRoot, "@formkit+vue@2.1.0", "@formkit/vue");
    writeStoreCopy(fixtureRoot, "vee-validate@4.15.1", "vee-validate");
    assert.deepEqual(isolateUniqueVueFormPackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "@formkit", "vue")), false);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vee-validate")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an already hoisted vee-validate is left for isolation; this repair does not relink it", () => {
  const { fixtureRoot, outer } = scaffold(["vee-validate"]);
  try {
    writeStoreCopy(fixtureRoot, "vee-validate@4.15.1", "vee-validate");
    const hoisted = path.join(fixtureRoot, "node_modules", "vee-validate");
    fs.mkdirSync(hoisted, { recursive: true });
    fs.writeFileSync(path.join(hoisted, "package.json"), `{"name":"vee-validate"}\n`);
    assert.deepEqual(isolateUniqueVueFormPackages(fixtureRoot), []);
    assert.equal(fs.lstatSync(hoisted).isSymbolicLink(), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
