import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueLocalTypePackages } from "../../tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * Nuxt writes `paths` to a package it resolved above the fixture. Isolation
 * will not link that target in. If the fixture's pnpm store holds exactly one
 * copy of the name, this repair uses that copy. If it holds several, it uses
 * the fixture's own `vue` version to pick the matching peer suffix, and still
 * does not guess when that does not select exactly one copy (#4461).
 */

function scaffold() {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-unique-")));
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, ".nuxt"), { recursive: true });
  for (const name of ["vue-router", "@vue/runtime-core"]) {
    const packageRoot = path.join(outer, "node_modules", name);
    fs.mkdirSync(packageRoot, { recursive: true });
    fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
  }
  return { outer, fixtureRoot };
}

function writeStoreCopy(fixtureRoot: string, id: string, name: string) {
  const packageRoot = path.join(fixtureRoot, "node_modules", ".pnpm", id, "node_modules", name);
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
  return packageRoot;
}

function writeConfig(fixtureRoot: string, paths: Record<string, string[]>) {
  const configPath = path.join(fixtureRoot, ".nuxt", "tsconfig.app.json");
  fs.writeFileSync(
    configPath,
    `// generated\n${JSON.stringify({ compilerOptions: { paths } }, null, 2)}\n`,
  );
  return configPath;
}

function writeFixtureVue(fixtureRoot: string, version: string) {
  const packageRoot = path.join(fixtureRoot, "node_modules", "vue");
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(
    path.join(packageRoot, "package.json"),
    `{"name":"vue","version":"${version}"}\n`,
  );
}

test("an outside target with exactly one in-fixture copy is linked from that copy", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0", "vue-router");
    const configPath = writeConfig(fixtureRoot, {
      "vue-router": ["../../node_modules/vue-router"],
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      { name: "vue-router", target: "node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router" },
    ]);
    const linked = fs.realpathSync(path.join(fixtureRoot, "node_modules", "vue-router"));
    assert.equal(linked.startsWith(fs.realpathSync(fixtureRoot) + path.sep), true);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside target with several in-fixture copies is left unlinked", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.30", "vue-router");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.13", "vue-router");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.4.38", "vue-router");
    const configPath = writeConfig(fixtureRoot, {
      "vue-router": ["../../node_modules/vue-router"],
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-router")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an inside target is left for isolation; this repair does not relink it", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0", "vue-router");
    const configPath = writeConfig(fixtureRoot, {
      "vue-router": ["../node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router"],
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-router")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a name no ancestor provides is left alone", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "defu@6.1.4", "defu");
    const configPath = writeConfig(fixtureRoot, {
      defu: ["../../node_modules/defu"],
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "defu")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("several copies collapse to the one whose Vue peer matches the fixture", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeFixtureVue(fixtureRoot, "3.5.30");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.30", "vue-router");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.13", "vue-router");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.4.38", "vue-router");
    const configPath = writeConfig(fixtureRoot, {
      "vue-router": ["../../node_modules/vue-router"],
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      {
        name: "vue-router",
        target: "node_modules/.pnpm/vue-router@5.1.0_vue@3.5.30/node_modules/vue-router",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("several copies whose Vue peers miss the fixture stay unlinked", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeFixtureVue(fixtureRoot, "3.5.30");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.13", "vue-router");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.4.38", "vue-router");
    const configPath = writeConfig(fixtureRoot, {
      "vue-router": ["../../node_modules/vue-router"],
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-router")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("several copies that share the fixture Vue peer are still not guessed between", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeFixtureVue(fixtureRoot, "3.5.30");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.30", "vue-router");
    writeStoreCopy(fixtureRoot, "vue-router@4.5.1_vue@3.5.30", "vue-router");
    const configPath = writeConfig(fixtureRoot, {
      "vue-router": ["../../node_modules/vue-router"],
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-router")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
