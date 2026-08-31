import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueVueRuntimePackages } from "../../legacy-tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * vue-class-component and vue-property-decorator live in Vize's
 * `tests/package.json`. TypeScript can climb into those copies and load
 * Vize's Vue beside the fixture (#4461). Unique isolation links the fixture
 * store copy when an ancestor is reachable.
 */

function scaffold(names: string[]) {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-vue-class-")),
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

test("ancestor class-component packages with one in-fixture copy each are linked from those copies", () => {
  const { fixtureRoot, outer } = scaffold(["vue-class-component", "vue-property-decorator"]);
  try {
    writeStoreCopy(fixtureRoot, "vue-class-component@8.0.0-rc.1", "vue-class-component");
    writeStoreCopy(fixtureRoot, "vue-property-decorator@10.0.0-rc.3", "vue-property-decorator");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), [
      {
        name: "vue-class-component",
        target:
          "node_modules/.pnpm/vue-class-component@8.0.0-rc.1/node_modules/vue-class-component",
      },
      {
        name: "vue-property-decorator",
        target:
          "node_modules/.pnpm/vue-property-decorator@10.0.0-rc.3/node_modules/vue-property-decorator",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("class-component packages no ancestor provides are left alone", () => {
  const { fixtureRoot, outer } = scaffold([]);
  try {
    writeStoreCopy(fixtureRoot, "vue-class-component@8.0.0-rc.1", "vue-class-component");
    writeStoreCopy(fixtureRoot, "vue-property-decorator@10.0.0-rc.3", "vue-property-decorator");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), []);
    assert.equal(
      fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-class-component")),
      false,
    );
    assert.equal(
      fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-property-decorator")),
      false,
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an already hoisted vue-class-component is left for isolation; this repair does not relink it", () => {
  const { fixtureRoot, outer } = scaffold(["vue-class-component"]);
  try {
    writeStoreCopy(fixtureRoot, "vue-class-component@8.0.0-rc.1", "vue-class-component");
    const hoisted = path.join(fixtureRoot, "node_modules", "vue-class-component");
    fs.mkdirSync(hoisted, { recursive: true });
    fs.writeFileSync(path.join(hoisted, "package.json"), `{"name":"vue-class-component"}\n`);
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), []);
    assert.equal(fs.lstatSync(hoisted).isSymbolicLink(), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
