import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { nativeProjectSource } from "../../npm/builder/vite/src/plugin/host-config-source.ts";
import { projectViteConfigSource } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";
import { elkSetupSfcViteConfig } from "../_fixtures/differential/config/nuxt-elk-host-options/resolved-vite-config.ts";

void test("Elk setupSFC host options project in development and both build environments", async () => {
  const expected = JSON.parse(
    await readFile(
      new URL(
        "../_fixtures/differential/config/nuxt-elk-host-options/projected.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  for (const env of [
    { command: "serve", mode: "development", isSsrBuild: false },
    { command: "build", mode: "production", isSsrBuild: false },
    { command: "build", mode: "production", isSsrBuild: true },
  ]) {
    const source = elkSetupSfcViteConfig(env.command === "build");
    const host = source.vue;
    const descriptors = Object.getOwnPropertyDescriptors(host);
    Object.freeze(host.include);
    Object.freeze(host.script);
    Object.freeze(source);

    // Without the private constructor context, these remain invalid native options.
    assert.equal(nativeProjectSource(source), source);
    await assert.rejects(projectViteConfigSource(source, env), /unknown option script/);
    const projected = await projectViteConfigSource(nativeProjectSource(source, "nuxt"), env);
    assert.deepEqual(projected, expected);
    assert.equal(source.vue, host);
    assert.deepEqual(Object.getOwnPropertyDescriptors(source.vue), descriptors);
    assert.equal(source.vue.include, descriptors.include.value);
    assert.equal(source.vue.script, descriptors.script.value);
    assert.equal(source.vue.template, descriptors.template.value);
  }
});

void test("host filter forms retain their exact identities and are never interpreted as native settings", async () => {
  for (const filters of [
    { include: /\.vue$/, exclude: /\/vendor\// },
    { include: "**/*.vue", exclude: "**/ignored/**" },
    { include: [/\.vue$/, /\.setup\.[cm]?[jt]sx?$/], exclude: [/\.ce\.vue$/, "**/ignored/**"] },
    { exclude: [] },
  ]) {
    const host = Object.freeze({ script: { hoistStatic: true }, ...filters });
    const source = Object.freeze({ vue: host });
    const descriptors = Object.getOwnPropertyDescriptors(host);
    assert.deepEqual(await projectViteConfigSource(nativeProjectSource(source, "nuxt")), {
      typeChecker: { jsxTypecheck: true },
    });
    assert.equal(source.vue, host);
    assert.deepEqual(Object.getOwnPropertyDescriptors(host), descriptors);
    assert.equal(source.vue.exclude, filters.exclude);
    assert.equal(source.vue.script, descriptors.script.value);
  }
});

void test("only the explicit Nuxt constructor context excludes the complete host compiler object", async () => {
  const env = { command: "build", mode: "production", isSsrBuild: true };
  for (const host of [
    { script: { hoistStatic: true } },
    { isProduction: true },
    { template: { compilerOptions: { customElement: () => true }, transformAssetUrls: true } },
    { features: { propsDestructure: true } },
    { isProduction: false, template: {}, script: {}, features: {} },
  ]) {
    let callbackCalls = 0;
    let hookReads = 0;
    const source = {
      vue: host,
      vize: (actual: unknown) => {
        callbackCalls++;
        assert.equal(actual, env);
        return { formatter: { printWidth: 70 } };
      },
      get unrelatedHostHook() {
        hookReads++;
        return host;
      },
    };
    const originalDescriptors = Object.getOwnPropertyDescriptors(source);
    const view = nativeProjectSource(source, "nuxt") as typeof source;
    assert.notEqual(view, source);
    assert.equal(source.vue, host);
    assert.equal("vue" in view, false);
    assert.equal(callbackCalls, 0);
    assert.equal(hookReads, 0);
    assert.deepEqual(
      Object.getOwnPropertyDescriptor(view, "unrelatedHostHook"),
      originalDescriptors.unrelatedHostHook,
    );
    assert.equal(view.vize, source.vize);
    assert.deepEqual(await projectViteConfigSource(view, env), {
      typeChecker: { jsxTypecheck: true },
      formatter: { printWidth: 70 },
    });
    assert.equal(callbackCalls, 1);
    assert.equal(hookReads, 0);
    assert.deepEqual(Object.getOwnPropertyDescriptors(source), originalDescriptors);
  }
});

void test("spoofable host flags and all canonical, mixed, unknown or nonplain objects stay strict", async () => {
  const host = { script: { hoistStatic: true } };
  const spoofed = {
    vue: host,
    appType: "custom",
    configFile: false,
    define: { __NUXT_VERSION__: '"4.5.2"' },
    nuxtPageMeta: true,
  };
  assert.equal(nativeProjectSource(spoofed), spoofed);
  await assert.rejects(projectViteConfigSource(spoofed), /unknown option script/);
  for (const vue of [
    {},
    { version: "3" },
    { linter: { enabled: false } },
    { unknown: true },
    { script: {}, version: "3" },
    { script: {}, entries: [{ formatter: {} }] },
    { script: {}, future: true },
    { script: {}, include: [/\.vue$/], compiler: { whitespace: "preserve" } },
    { script: {}, exclude: [], future: true },
    Object.assign(Object.create({ version: "3" }), { script: {} }),
    Object.defineProperty({ script: {} }, "compiler", { value: {} }),
    Object.assign({ script: {} }, { [Symbol("native")]: {} }),
  ]) {
    const source = { vue };
    assert.equal(nativeProjectSource(source, "nuxt"), source);
    if (Reflect.ownKeys(vue).includes("script") || Reflect.ownKeys(vue).includes("unknown")) {
      await assert.rejects(projectViteConfigSource(source), /unknown option|must be object/);
    }
  }
  const array = Object.assign([], { vue: { script: {} } });
  assert.equal(nativeProjectSource(array, "nuxt"), array);
  await assert.rejects(projectViteConfigSource(array), /source must be an object/);
  for (const source of [null, undefined, 0, "config", [], {}, { vue: null }]) {
    assert.equal(nativeProjectSource(source, "nuxt"), source);
  }
});

void test("host root getters are evaluated once and their original descriptors remain intact", async () => {
  let reads = 0;
  const host = { script: {} };
  const source = Object.defineProperty({ vize: { linter: { enabled: false } } }, "vue", {
    enumerable: true,
    get() {
      reads++;
      return host;
    },
  });
  const before = Object.getOwnPropertyDescriptor(source, "vue");
  const view = nativeProjectSource(source, "nuxt");
  assert.equal(reads, 1);
  assert.deepEqual(await projectViteConfigSource(view), {
    typeChecker: { jsxTypecheck: true },
    linter: { enabled: false },
  });
  assert.equal(reads, 1);
  assert.deepEqual(Object.getOwnPropertyDescriptor(source, "vue"), before);
});

void test("changing canonical accessor values cannot bypass the one-read strict admission", async () => {
  for (const first of [
    { compiler: { enabled: false } },
    { script: {}, version: "3" },
    { unknown: true },
  ]) {
    let reads = 0;
    const source = Object.defineProperty({}, "vue", {
      enumerable: true,
      get() {
        return ++reads === 1 ? first : {};
      },
    });
    const descriptor = Object.getOwnPropertyDescriptor(source, "vue");
    await assert.rejects(
      projectViteConfigSource(nativeProjectSource(source, "nuxt")),
      /unknown option/,
    );
    assert.equal(reads, 1);
    assert.deepEqual(Object.getOwnPropertyDescriptor(source, "vue"), descriptor);
  }
  let reads = 0;
  const first = { linter: { enabled: false } };
  const source = Object.defineProperty({}, "vue", {
    enumerable: true,
    get() {
      return ++reads === 1 ? first : { unknown: true };
    },
  });
  assert.deepEqual(await projectViteConfigSource(nativeProjectSource(source, "nuxt")), {
    typeChecker: { jsxTypecheck: true },
    linter: { enabled: false },
  });
  assert.equal(reads, 1);
});
