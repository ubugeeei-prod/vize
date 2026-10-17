import assert from "node:assert/strict";
import { test } from "node:test";
import { nativeProjectSource } from "../../npm/builder/vite/src/plugin/host-config-source.ts";
import { projectViteConfigSource } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

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
      fmt: { vue: { printWidth: 70 } },
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
    assert.equal(view.fmt, source.fmt);
    assert.deepEqual(await projectViteConfigSource(view, env), {
      typeChecker: { jsxTypecheck: true },
      formatter: { printWidth: 70 },
    });
    assert.equal(callbackCalls, 0);
    assert.equal(hookReads, 0);
    assert.deepEqual(Object.getOwnPropertyDescriptors(source), originalDescriptors);
    const legacy = Object.defineProperty(
      Object.create(Object.getPrototypeOf(source), originalDescriptors),
      "vize",
      {
        enumerable: true,
        value: (actual: unknown) => {
          callbackCalls++;
          assert.equal(actual, env);
          return { formatter: { printWidth: 70 } };
        },
      },
    );
    await assert.rejects(
      projectViteConfigSource(nativeProjectSource(legacy, "nuxt"), env),
      /vize: configuration is no longer supported/,
    );
    assert.equal(callbackCalls, 0, "retired callbacks must never execute");
    assert.equal(hookReads, 0);
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
  const source = Object.defineProperty({ lint: { vue: { enabled: false } } }, "vue", {
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
