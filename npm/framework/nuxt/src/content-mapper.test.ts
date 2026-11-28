import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  injectVizeContentMapper,
  registerNuxtContentMapper,
  resolveVizeMapperPackage,
} from "./content-mapper.ts";

function installPackage(nodeModules: string, name: string): string {
  const dir = join(nodeModules, name);
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, "package.json"), JSON.stringify({ name }));
  return dir;
}

/** pnpm's strict layout: only direct dependencies are linked into the project's node_modules. */
function strictLayout(directVize: boolean) {
  const root = mkdtempSync(join(tmpdir(), "vize-nuxt-mapper-"));
  const buildDir = join(root, ".nuxt");
  mkdirSync(buildDir);
  const store = join(root, "node_modules", ".pnpm", "@vizejs+nuxt@0.0.0", "node_modules");
  const moduleDir = join(installPackage(store, "@vizejs/nuxt"), "dist");
  mkdirSync(moduleDir);
  const transitiveVize = installPackage(store, "vize");
  if (directVize) installPackage(join(root, "node_modules"), "vize");
  return { root, buildDir, moduleDir, transitiveVize };
}

const expectedMapper = { package: "vize", extensions: [".vue"] };

void test("injects the Vize mapper once without changing existing entries", () => {
  const existing = { package: "vize", extensions: [".md"], options: { strict: true } };
  const tsConfig: Record<string, unknown> = {
    compilerOptions: { strict: true },
    contentMappers: [existing],
  };

  assert.equal(injectVizeContentMapper(tsConfig), true);
  assert.deepEqual(tsConfig.contentMappers, [existing, expectedMapper]);
  assert.equal((tsConfig.contentMappers as unknown[])[0], existing);

  assert.equal(injectVizeContentMapper(tsConfig), false);
  assert.deepEqual(tsConfig.contentMappers, [existing, expectedMapper]);
});

void test("keeps malformed array entries while adding the valid default", () => {
  const malformed = { package: "vize", extensions: "not-an-array", options: { strict: false } };
  const tsConfig = { contentMappers: [malformed] };

  assert.equal(injectVizeContentMapper(tsConfig), true);
  assert.deepEqual(tsConfig.contentMappers, [malformed, expectedMapper]);
  assert.equal(tsConfig.contentMappers[0], malformed);
});

void test("preserves an explicitly configured Vue mapper", () => {
  const configured = [
    { package: "custom-vue-mapper", extensions: [".vue"], options: { mode: "strict" } },
    { package: "vize", extensions: [".md"] },
  ];
  const tsConfig = { contentMappers: configured };

  assert.equal(injectVizeContentMapper(tsConfig), false);
  assert.equal(tsConfig.contentMappers, configured);
  assert.deepEqual(tsConfig.contentMappers, configured);
});

void test("does not overwrite a malformed contentMappers value", () => {
  for (const contentMappers of [null, "inherited", { package: "vize" }]) {
    const tsConfig = { contentMappers };
    assert.equal(injectVizeContentMapper(tsConfig), false);
    assert.equal(tsConfig.contentMappers, contentMappers);
  }
});

void test("registers prepare:types for Nuxt 3 and 4 only", () => {
  for (const nuxtMajor of [3, 4]) {
    let hookName: string | undefined;
    let hook: ((payload: unknown) => unknown) | undefined;
    registerNuxtContentMapper(
      {
        hook(name, callback) {
          hookName = name;
          hook = callback;
        },
      },
      nuxtMajor,
    );

    assert.equal(hookName, "prepare:types");
    const tsConfig = { compilerOptions: { strict: true } };
    hook?.({ tsConfig });
    hook?.({ tsConfig });
    assert.deepEqual(tsConfig, {
      compilerOptions: { strict: true },
      contentMappers: [expectedMapper],
    });
  }

  let registered = false;
  registerNuxtContentMapper(
    {
      hook() {
        registered = true;
      },
    },
    2,
  );
  assert.equal(registered, false);
});

void test("names a transitive vize by path when the tsconfig directory cannot resolve it", (t) => {
  const layout = strictLayout(false);
  t.after(() => rmSync(layout.root, { recursive: true, force: true }));

  assert.equal(resolveVizeMapperPackage(layout.buildDir, layout.moduleDir), layout.transitiveVize);

  let hook: ((payload: unknown) => unknown) | undefined;
  registerNuxtContentMapper(
    {
      options: { buildDir: layout.buildDir },
      hook(_name, callback) {
        hook = callback;
      },
    },
    4,
    layout.moduleDir,
  );
  const tsConfig = {};
  hook?.({ tsConfig });
  assert.deepEqual(tsConfig, {
    contentMappers: [{ package: layout.transitiveVize, extensions: [".vue"] }],
  });
});

void test("keeps the package name when vize resolves from the tsconfig directory", (t) => {
  const layout = strictLayout(true);
  t.after(() => rmSync(layout.root, { recursive: true, force: true }));

  assert.equal(resolveVizeMapperPackage(layout.buildDir, layout.moduleDir), "vize");
  const typesDir = join(layout.root, "node_modules", ".cache", "nuxt", "types");
  mkdirSync(typesDir, { recursive: true });
  assert.equal(resolveVizeMapperPackage(typesDir, layout.moduleDir), "vize");
});

void test("keeps the package name when neither location installs vize", (t) => {
  const root = mkdtempSync(join(tmpdir(), "vize-nuxt-mapper-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));

  assert.equal(resolveVizeMapperPackage(root, root), "vize");
});
