import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueUiLibraryPackages } from "../../legacy-tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * Vue UI libraries in Vize's `tests/package.json` still climb out of a
 * fixture that does not declare them (#4461). Unique isolation links the
 * fixture store copy when an ancestor is reachable. Swiper is included
 * because `swiper/vue` types load Vue from the answering copy.
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
  const libraries = [
    ["@ionic/vue", "@ionic+vue@8.8.9"],
    ["@nuxt/ui", "@nuxt+ui@4.8.2"],
    ["ant-design-vue", "ant-design-vue@4.2.6"],
    ["element-plus", "element-plus@2.14.1"],
    ["naive-ui", "naive-ui@2.44.1"],
    ["primevue", "primevue@4.5.5"],
    ["quasar", "quasar@2.19.3"],
    ["reka-ui", "reka-ui@2.9.10"],
    ["swiper", "swiper@12.2.0"],
    ["vant", "vant@4.9.24"],
    ["vue-select", "vue-select@4.0.0-beta.6"],
    ["vue-virtual-scroller", "vue-virtual-scroller@3.0.4"],
  ];
  const { fixtureRoot, outer } = scaffold(libraries.map(([name]) => name));
  try {
    for (const [name, id] of libraries) writeStoreCopy(fixtureRoot, id, name);
    assert.deepEqual(
      isolateUniqueUiLibraryPackages(fixtureRoot),
      libraries.map(([name, id]) => ({
        name,
        target: `node_modules/.pnpm/${id}/node_modules/${name}`,
      })),
    );
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
