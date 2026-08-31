import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueLocalTypePackages } from "../../legacy-tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * Isolation already follows `extends` / `references`. Unique repair used to
 * read only the source config's own `paths`, so a check tsconfig that merely
 * extends the generated app config never saw an outside mapping (#4461).
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-unique-extends-")),
  );
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

function writeAppPaths(fixtureRoot: string) {
  const configPath = path.join(fixtureRoot, ".nuxt", "tsconfig.app.json");
  fs.writeFileSync(
    configPath,
    `// generated\n${JSON.stringify(
      { compilerOptions: { paths: { "vue-router": ["../../node_modules/vue-router"] } } },
      null,
      2,
    )}\n`,
  );
  return configPath;
}

test("an outside mapping reached only through extends still links the unique copy", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0", "vue-router");
    writeAppPaths(fixtureRoot);
    const configPath = path.join(fixtureRoot, ".nuxt", "tsconfig.check.json");
    fs.writeFileSync(configPath, `// check-only\n{ "extends": "./tsconfig.app.json", }\n`);
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      { name: "vue-router", target: "node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router" },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside mapping reached only through references still links the unique copy", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0", "vue-router");
    writeAppPaths(fixtureRoot);
    const configPath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      configPath,
      `${JSON.stringify({
        files: [],
        references: [{ path: "@vue/tsconfig" }, { path: "./.nuxt/tsconfig.app.json" }],
      })}\n`,
    );
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      { name: "vue-router", target: "node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router" },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

function writeFixtureVue(fixtureRoot: string, version: string) {
  const packageRoot = path.join(fixtureRoot, "node_modules", "vue");
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(
    path.join(packageRoot, "package.json"),
    `{"name":"vue","version":"${version}"}\n`,
  );
}

test("several copies reached through extends collapse to the matching Vue peer", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeFixtureVue(fixtureRoot, "3.5.30");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.30", "vue-router");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.13", "vue-router");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.4.38", "vue-router");
    writeAppPaths(fixtureRoot);
    const configPath = path.join(fixtureRoot, ".nuxt", "tsconfig.check.json");
    fs.writeFileSync(configPath, `{ "extends": "./tsconfig.app.json" }\n`);
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

test("several copies reached through extends are still not guessed between", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.30", "vue-router");
    writeStoreCopy(fixtureRoot, "vue-router@5.1.0_vue@3.5.13", "vue-router");
    writeAppPaths(fixtureRoot);
    const configPath = path.join(fixtureRoot, ".nuxt", "tsconfig.check.json");
    fs.writeFileSync(configPath, `{ "extends": "./tsconfig.app.json" }\n`);
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-router")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
