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
    await assert.rejects(projectViteConfigSource(source, env), /unknown option "script"/);
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

void test("Nuxt 4.7's plugin-vue compiler module stays with the host", async () => {
  // @nuxt/vite-builder 4.7 passes plugin-vue the app's `vue/compiler-sfc` as
  // `vite.vue.compiler`, next to the schema defaults. The module namespace is
  // an exotic object whose exports must never be read by the projection.
  const compilerSfc =
    "data:text/javascript,export const parse = () => {}; export const compileScript = () => {};";
  const compiler: object = await import(compilerSfc);
  const env = { command: "build", mode: "production", isSsrBuild: false };
  const resolved = elkSetupSfcViteConfig(true);
  const host = { ...resolved.vue, compiler };
  const source = Object.freeze({ ...resolved, vue: host });
  const descriptors = Object.getOwnPropertyDescriptors(host);

  assert.equal(nativeProjectSource(source), source);
  await assert.rejects(projectViteConfigSource(source, env), /unknown option "script"/);
  const view = nativeProjectSource(source, "nuxt");
  assert.equal("vue" in (view as object), false);
  assert.deepEqual(await projectViteConfigSource(view, env), {
    typeChecker: { jsxTypecheck: true },
  });
  assert.equal(source.vue, host);
  assert.equal(source.vue.compiler, compiler);
  assert.deepEqual(Object.getOwnPropertyDescriptors(host), descriptors);
});

void test("spoofable host flags stay strict outside the explicit Nuxt constructor context", async () => {
  const host = { script: { hoistStatic: true } };
  const spoofed = {
    vue: host,
    appType: "custom",
    configFile: false,
    define: { __NUXT_VERSION__: '"4.5.2"' },
    nuxtPageMeta: true,
  };
  assert.equal(nativeProjectSource(spoofed), spoofed);
  await assert.rejects(projectViteConfigSource(spoofed), /unknown option "script"/);
  for (const vue of [{ unknown: true }, { script: {}, compiler: { whitespace: "preserve" } }]) {
    const source = { vue };
    assert.equal(nativeProjectSource(source), source);
    await assert.rejects(projectViteConfigSource(source), /unknown option/);
  }
});

void test("in the Nuxt host the whole vue key is host-owned, whatever it holds", async () => {
  // Nuxt's schema always fills `vite.vue` with plugin-vue options, so it never
  // carries Vize settings; those come from the project config files.
  for (const vue of [
    {},
    { version: "3" },
    { linter: { enabled: false } },
    { compiler: { whitespace: "preserve" } },
    { unknown: true },
    { script: {}, version: "3" },
    { script: {}, entries: [{ formatter: {} }] },
    { script: {}, future: true },
    { script: {}, include: [/\.vue$/], compiler: { whitespace: "preserve" } },
    { script: {}, exclude: [], future: true },
    { script: {}, style: { trim: true }, customElement: /\.ce\.vue$/ },
    Object.assign(Object.create({ version: "3" }), { script: {} }),
    Object.defineProperty({ script: {} }, "compiler", { value: {} }),
    Object.assign({ script: {} }, { [Symbol("native")]: {} }),
    null,
    "vue",
    0,
  ]) {
    const source = Object.freeze({ vue, fmt: { vue: { printWidth: 70 } } });
    const view = nativeProjectSource(source, "nuxt") as { fmt?: unknown };
    assert.notEqual(view, source);
    assert.equal("vue" in view, false);
    assert.equal(view.fmt, source.fmt);
    assert.deepEqual(await projectViteConfigSource(view), {
      typeChecker: { jsxTypecheck: true },
      formatter: { printWidth: 70 },
    });
    assert.equal(source.vue, vue);
  }
  const array = Object.assign([], { vue: { script: {} } });
  assert.equal(nativeProjectSource(array, "nuxt"), array);
  await assert.rejects(projectViteConfigSource(array), /source must be an object/);
  for (const source of [null, undefined, 0, "config", [], {}, Object.create({ vue: {} })]) {
    assert.equal(nativeProjectSource(source, "nuxt"), source);
  }
});

void test("host vue getters are never evaluated and the original descriptors remain intact", async () => {
  let reads = 0;
  let hookReads = 0;
  const source = Object.defineProperties(
    { lint: { vue: { enabled: false } } },
    {
      vue: {
        enumerable: true,
        get() {
          reads++;
          return { unknown: true };
        },
      },
      unrelatedHostHook: {
        enumerable: true,
        get() {
          hookReads++;
          return {};
        },
      },
    },
  );
  const before = Object.getOwnPropertyDescriptors(source);
  const view = nativeProjectSource(source, "nuxt");
  assert.equal(reads, 0);
  assert.equal(hookReads, 0);
  assert.deepEqual(
    Object.getOwnPropertyDescriptor(view, "unrelatedHostHook"),
    before.unrelatedHostHook,
  );
  assert.deepEqual(await projectViteConfigSource(view), {
    typeChecker: { jsxTypecheck: true },
    linter: { enabled: false },
  });
  assert.equal(reads, 0);
  assert.deepEqual(Object.getOwnPropertyDescriptors(source), before);
});
