import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueVueVisualizationPackages } from "../../legacy-tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * TresJS and vue-chartjs live in Vize's `tests/package.json`. TypeScript can
 * climb into those copies and load Vize's Vue beside the fixture (#4461).
 * Unique isolation links the fixture store copy when an ancestor is reachable.
 */

function scaffold(names: string[]) {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-vue-viz-")));
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

test("ancestor visualization packages with one in-fixture copy each are linked from those copies", () => {
  const { fixtureRoot, outer } = scaffold(["@tresjs/core", "vue-chartjs"]);
  try {
    writeStoreCopy(fixtureRoot, "@tresjs+core@5.8.1", "@tresjs/core");
    writeStoreCopy(fixtureRoot, "vue-chartjs@5.3.3", "vue-chartjs");
    assert.deepEqual(isolateUniqueVueVisualizationPackages(fixtureRoot), [
      {
        name: "@tresjs/core",
        target: "node_modules/.pnpm/@tresjs+core@5.8.1/node_modules/@tresjs/core",
      },
      {
        name: "vue-chartjs",
        target: "node_modules/.pnpm/vue-chartjs@5.3.3/node_modules/vue-chartjs",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("visualization packages no ancestor provides are left alone", () => {
  const { fixtureRoot, outer } = scaffold([]);
  try {
    writeStoreCopy(fixtureRoot, "@tresjs+core@5.8.1", "@tresjs/core");
    writeStoreCopy(fixtureRoot, "vue-chartjs@5.3.3", "vue-chartjs");
    assert.deepEqual(isolateUniqueVueVisualizationPackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "@tresjs", "core")), false);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-chartjs")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an already hoisted vue-chartjs is left for isolation; this repair does not relink it", () => {
  const { fixtureRoot, outer } = scaffold(["vue-chartjs"]);
  try {
    writeStoreCopy(fixtureRoot, "vue-chartjs@5.3.3", "vue-chartjs");
    const hoisted = path.join(fixtureRoot, "node_modules", "vue-chartjs");
    fs.mkdirSync(hoisted, { recursive: true });
    fs.writeFileSync(path.join(hoisted, "package.json"), `{"name":"vue-chartjs"}\n`);
    assert.deepEqual(isolateUniqueVueVisualizationPackages(fixtureRoot), []);
    assert.equal(fs.lstatSync(hoisted).isSymbolicLink(), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
