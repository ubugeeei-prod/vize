import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueVueRuntimePackages } from "../../tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * `vue-router` (and other Vue packages) still climb `node_modules` for `vue`
 * even when no tsconfig declared it (#4461). Unique isolation links the
 * fixture copy of the Vue runtime packages when an ancestor copy is reachable.
 */

function scaffold(names: string[] = ["vue", "@vue/runtime-core", "@vue/runtime-dom"]) {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-vue-rt-")));
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(fixtureRoot, { recursive: true });
  for (const name of names) {
    const packageRoot = path.join(outer, "node_modules", name);
    fs.mkdirSync(packageRoot, { recursive: true });
    fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
  }
  return { fixtureRoot, outer };
}

function writeStoreCopy(fixtureRoot: string, id: string, name: string, version = "3.5.30") {
  const packageRoot = path.join(fixtureRoot, "node_modules", ".pnpm", id, "node_modules", name);
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(
    path.join(packageRoot, "package.json"),
    `{"name":"${name}","version":"${version}"}\n`,
  );
  return packageRoot;
}

test("an ancestor vue with exactly one in-fixture copy is linked from that copy", () => {
  const { fixtureRoot, outer } = scaffold(["vue"]);
  try {
    writeStoreCopy(fixtureRoot, "vue@3.5.30", "vue");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), [
      { name: "vue", target: "node_modules/.pnpm/vue@3.5.30/node_modules/vue" },
    ]);
    assert.equal(
      fs.realpathSync(path.join(fixtureRoot, "node_modules", "vue")),
      fs.realpathSync(
        path.join(fixtureRoot, "node_modules", ".pnpm", "vue@3.5.30", "node_modules", "vue"),
      ),
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a vue runtime name no ancestor provides is left alone", () => {
  const { fixtureRoot, outer } = scaffold([]);
  try {
    writeStoreCopy(fixtureRoot, "vue@3.5.30", "vue");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("several vue copies whose Vue peers miss a unique match stay unlinked", () => {
  const { fixtureRoot, outer } = scaffold(["vue"]);
  try {
    writeStoreCopy(fixtureRoot, "vue@3.5.13", "vue", "3.5.13");
    writeStoreCopy(fixtureRoot, "vue@3.4.38", "vue", "3.4.38");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an already hoisted vue is left for isolation; this repair does not relink it", () => {
  const { fixtureRoot, outer } = scaffold(["vue"]);
  try {
    writeStoreCopy(fixtureRoot, "vue@3.5.30", "vue");
    const hoisted = path.join(fixtureRoot, "node_modules", "vue");
    fs.mkdirSync(hoisted, { recursive: true });
    fs.writeFileSync(path.join(hoisted, "package.json"), `{"name":"vue","version":"3.5.30"}\n`);
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), []);
    assert.equal(fs.lstatSync(hoisted).isSymbolicLink(), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an ancestor @vue/compiler-sfc with exactly one in-fixture copy is linked", () => {
  const { fixtureRoot, outer } = scaffold(["@vue/compiler-sfc"]);
  try {
    writeStoreCopy(fixtureRoot, "@vue+compiler-sfc@3.5.30", "@vue/compiler-sfc");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), [
      {
        name: "@vue/compiler-sfc",
        target: "node_modules/.pnpm/@vue+compiler-sfc@3.5.30/node_modules/@vue/compiler-sfc",
      },
    ]);
    assert.equal(
      fs.realpathSync(path.join(fixtureRoot, "node_modules", "@vue", "compiler-sfc")),
      fs.realpathSync(
        path.join(
          fixtureRoot,
          "node_modules",
          ".pnpm",
          "@vue+compiler-sfc@3.5.30",
          "node_modules",
          "@vue",
          "compiler-sfc",
        ),
      ),
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an ancestor @vue/compiler-dom with exactly one in-fixture copy is linked", () => {
  const { fixtureRoot, outer } = scaffold(["@vue/compiler-dom"]);
  try {
    writeStoreCopy(fixtureRoot, "@vue+compiler-dom@3.5.30", "@vue/compiler-dom");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), [
      {
        name: "@vue/compiler-dom",
        target: "node_modules/.pnpm/@vue+compiler-dom@3.5.30/node_modules/@vue/compiler-dom",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an ancestor @vue/runtime-core with exactly one in-fixture copy is linked", () => {
  const { fixtureRoot, outer } = scaffold(["@vue/runtime-core"]);
  try {
    writeStoreCopy(fixtureRoot, "@vue+runtime-core@3.5.30", "@vue/runtime-core");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), [
      {
        name: "@vue/runtime-core",
        target: "node_modules/.pnpm/@vue+runtime-core@3.5.30/node_modules/@vue/runtime-core",
      },
    ]);
    assert.equal(
      fs.realpathSync(path.join(fixtureRoot, "node_modules", "@vue", "runtime-core")),
      fs.realpathSync(
        path.join(
          fixtureRoot,
          "node_modules",
          ".pnpm",
          "@vue+runtime-core@3.5.30",
          "node_modules",
          "@vue",
          "runtime-core",
        ),
      ),
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a nested runtime-dom store copy is hoisted when no ancestor is hoisted", () => {
  const { fixtureRoot, outer } = scaffold([]);
  try {
    writeStoreCopy(fixtureRoot, "@vue+runtime-dom@3.5.30", "@vue/runtime-dom");
    assert.deepEqual(isolateUniqueVueRuntimePackages(fixtureRoot), [
      {
        name: "@vue/runtime-dom",
        target: "node_modules/.pnpm/@vue+runtime-dom@3.5.30/node_modules/@vue/runtime-dom",
      },
    ]);
    assert.equal(
      fs.realpathSync(path.join(fixtureRoot, "node_modules", "@vue", "runtime-dom")),
      fs.realpathSync(
        path.join(
          fixtureRoot,
          "node_modules",
          ".pnpm",
          "@vue+runtime-dom@3.5.30",
          "node_modules",
          "@vue",
          "runtime-dom",
        ),
      ),
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
