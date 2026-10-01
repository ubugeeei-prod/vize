import assert from "node:assert/strict";
import { mkdtempSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  boundedRuntimeEnvironment,
  runtimeBundleChunk,
  runtimeBundleOptions,
  runtimeModulesCovered,
} from "./support/runtime-bundle-context.ts";
import { immutableRuntimeBundle } from "./support/runtime-bundle-cache.ts";

void test("development and production retain the original self-contained build definitions", () => {
  const dev = runtimeBundleOptions("/entry.js", false),
    prod = runtimeBundleOptions("/entry.js", true);
  assert.deepEqual(dev.build, {
    write: false,
    minify: false,
    lib: { entry: "/entry.js", formats: ["es"] },
  });
  assert.deepEqual(prod.build, { ...dev.build, minify: true });
  assert.equal(dev.configFile, false);
  assert.equal(prod.configFile, false);
  assert.deepEqual(dev.define, {
    "process.env.NODE_ENV": '"development"',
    __VUE_OPTIONS_API__: "true",
    __VUE_PROD_DEVTOOLS__: "false",
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "true",
  });
  assert.deepEqual(prod.define, {
    ...dev.define,
    "process.env.NODE_ENV": '"production"',
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "false",
  });
});

void test("actual working-directory env files and custom loaders/native bindings bypass reuse", () => {
  const cwd = realpathSync(mkdtempSync(join(tmpdir(), "vize-runtime-context-")));
  try {
    const context = boundedRuntimeEnvironment(cwd, {
      VITE_MODE: "one",
      NODE_ENV: "development",
      IGNORED: "value",
    });
    assert.ok(context);
    assert.equal(context.cwd, cwd);
    assert.deepEqual(context.env, { NODE_ENV: "development", VITE_MODE: "one" });
    assert.ok(context.envFiles.every(({ sha256 }) => sha256 === null));
    assert.notDeepEqual(boundedRuntimeEnvironment(cwd, { VITE_MODE: "two" }), context);
    for (const env of [
      { VITE_USER_NODE_ENV: "development" },
      { NAPI_RS_NATIVE_LIBRARY_PATH: "/custom.node" },
      { NAPI_RS_FORCE_WASI: "1" },
      { NODE_OPTIONS: "--import=./hook.js" },
      { NODE_OPTIONS: "--require ./hook.cjs" },
      { NODE_OPTIONS: "--loader ./loader.js" },
      { NODE_OPTIONS: "--experimental-loader=./loader.js" },
      { NODE_OPTIONS: "-r./hook.cjs" },
      { NODE_OPTIONS: '"--import" "data:text/javascript,globalThis.changed=true"' },
      { NODE_OPTIONS: '"--require" "./hook.cjs"' },
    ])
      assert.equal(boundedRuntimeEnvironment(cwd, env), null);
    for (const args of [
      ["--import", "./hook.js"],
      ["--require=./hook.cjs"],
      ["--loader", "./loader.js"],
      ["-r./hook.cjs"],
    ])
      assert.equal(boundedRuntimeEnvironment(cwd, {}, args), null);
    for (const name of [".env", ".env.local", ".env.production", ".env.production.local"]) {
      const path = join(cwd, name);
      writeFileSync(path, "VITE_INPUT=${ARBITRARY_ENV}");
      assert.equal(boundedRuntimeEnvironment(cwd, {}), null);
      rmSync(path);
    }
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

void test("actual bundler module coverage requires the entry and every nonvirtual source", () => {
  const cwd = realpathSync(mkdtempSync(join(tmpdir(), "vize-runtime-modules-")));
  try {
    const entry = join(cwd, "entry.js"),
      dep = join(cwd, "dep.js"),
      extra = join(cwd, "extra.js");
    for (const file of [entry, dep, extra]) writeFileSync(file, "export const value = 1");
    assert.equal(
      runtimeModulesCovered([entry, `${dep}?query`, "\0generated"], [entry, dep], entry),
      true,
    );
    assert.equal(runtimeModulesCovered([dep, "\0generated"], [entry, dep], entry), false);
    assert.equal(runtimeModulesCovered([entry, extra], [entry, dep], entry), false);
    assert.equal(runtimeModulesCovered([entry, "unknown:source"], [entry, dep], entry), false);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

void test("extra assets or external imports preserve the original chunk while refusing byte reuse", async () => {
  const cwd = realpathSync(mkdtempSync(join(tmpdir(), "vize-runtime-output-")));
  try {
    const chunk = {
      type: "chunk" as const,
      code: "export const original = true",
      imports: [] as string[],
      dynamicImports: [] as string[],
    };
    for (const output of [
      [{ type: "asset" as const }, chunk],
      [{ ...chunk, imports: ["external"] }],
      [{ ...chunk, dynamicImports: ["dynamic"] }],
    ]) {
      const selected = runtimeBundleChunk([{ output }]);
      assert.equal(selected.code, chunk.code);
      assert.equal(selected.reusable, false);
      let builds = 0;
      for (let call = 0; call < 2; call++) {
        const result = await immutableRuntimeBundle({
          directory: join(cwd, "cache"),
          inputs: () => ({ same: "inputs" }),
          canStore: () => selected.reusable,
          build: async () => {
            builds++;
            return { code: selected.code, modules: ["entry.js"] };
          },
        });
        assert.equal(result.code, chunk.code);
        assert.equal(result.cache, "bypassed");
      }
      assert.equal(builds, 2);
    }
    assert.equal(runtimeBundleChunk([{ output: [chunk] }]).reusable, true);
    assert.throws(
      () => runtimeBundleChunk([{ output: [chunk, chunk] }]),
      /one self-contained module/,
    );
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});
