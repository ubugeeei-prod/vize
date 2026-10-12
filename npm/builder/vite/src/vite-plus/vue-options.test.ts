import assert from "node:assert/strict";
import { test } from "node:test";
import { defineConfig as defineVpConfig, type ConfigEnv } from "vite-plus";
import { defineConfig, withVue } from "../vite-plus.ts";
import { resolveConfigExport } from "../config.ts";
import { projectViteConfigSource } from "../../../../cli/src/config/vite-runtime.mjs";
import { normalizeConfig, resolveConfig } from "./config.ts";
import { resolveVitePlus } from "./runtime.ts";
import { taskConfigKey, type ConfigWithVizeTasks, type VueConfigObject } from "./types.ts";

const env: ConfigEnv = {
  command: "serve",
  mode: "audit",
  isSsrBuild: true,
  isPreview: true,
};
const integration = {
  plugin: false,
  tasks: false,
  lint: false,
  fmt: false,
  conflicts: false,
} as const;
const injected = [
  "vite-plus:vitest-resolver",
  "vite-plus:auto-inline-matcher-deps",
  "vite-plus:coverage-version-guard",
];

void test("withVue preserves the real Vite Plus injection and evaluates each supported export once", async () => {
  for (const shape of ["object", "promise", "function", "async"] as const) {
    let calls = 0;
    const hook = () => "original hook";
    const plugin = { name: "consumer", transform: hook };
    const source = {
      plugins: [plugin],
      futureOption: { enabled: true },
      worker: { plugins: () => [plugin] },
      test: { projects: [{ test: { name: "nested" }, plugins: [plugin] }] },
      run: { tasks: { original: "echo original" } },
    };
    const callback = (context: ConfigEnv) => {
      calls++;
      assert.equal(context, env);
      return source;
    };
    const original = defineVpConfig(
      shape === "object"
        ? source
        : shape === "promise"
          ? Promise.resolve(source)
          : shape === "function"
            ? callback
            : async (context) => callback(context),
    );
    const wrapped = withVue({}, original, integration);
    assert.equal(calls, 0, `${shape} must not be evaluated eagerly`);
    const result = await wrapped(env);
    assert.equal(calls, shape === "function" || shape === "async" ? 1 : 0);
    const plugins = result.plugins as { name: string; transform?: unknown }[];
    assert.deepEqual(
      plugins.map(({ name }) => name),
      [...injected, "consumer"],
    );
    assert.equal(plugins[3], plugin);
    assert.equal(plugins[3]?.transform, hook);
    assert.equal((result as typeof source).worker.plugins, source.worker.plugins);
    assert.deepEqual((result as typeof source).futureOption, source.futureOption);
    assert.deepEqual(result.run?.tasks, source.run.tasks);
    const nested = (result as unknown as { test: typeof source.test }).test.projects[0];
    assert.deepEqual(
      nested?.plugins.map(({ name }) => name),
      [...injected, "consumer"],
    );
    assert.equal(nested?.plugins[3], plugin);
    assert.deepEqual(source.plugins, [plugin]);
    assert.deepEqual(source.test.projects[0]?.plugins, [plugin]);
    assert.ok(!("vue" in result));
  }
});

void test("canonical sections reach the same native normalizer as the selected Vite source", async () => {
  const source: VueConfigObject = {
    root: "packages/app",
    vize: { formatter: { printWidth: 70, singleQuote: true }, typeChecker: { strict: true } },
    vue: {
      version: "3",
      formatter: { singleQuote: false },
      typeChecker: { tsconfig: "tsconfig.json", strict: false, enabled: false },
      linter: { enabled: false },
      languageServer: { hover: false },
      files: ["src/**/*.vue"],
      entries: [{ files: ["widgets/**/*.vue"], linter: { rules: { "vue/no-v-html": "off" } } }],
    },
  };
  const result = await defineConfig(source, integration)(env);
  const metadata = (result as ConfigWithVizeTasks)[taskConfigKey]!;
  const native = await resolveConfigExport(metadata.config!, env);
  const direct = await resolveConfigExport(
    (await projectViteConfigSource(source, env, process.cwd())) as Parameters<
      typeof resolveConfigExport
    >[0],
    env,
  );
  assert.deepEqual(native, direct);
  assert.equal(native.formatter?.singleQuote, false);
  assert.equal(native.formatter?.printWidth, 70);
  assert.equal(native.typeChecker?.strict, false);
  assert.equal(metadata.options.check, false);
  assert.equal(metadata.options.lint, false);
  assert.equal(metadata.options.fmt, true);
  assert.ok(!("vue" in result));
});

void test("Vite fields keep consumer merge semantics while canonical arrays replace", async () => {
  const a = { name: "a" },
    b = { name: "b" };
  const first: VueConfigObject = {
    plugins: [a],
    server: { port: 4321 },
    vue: {
      files: ["old.vue"],
      formatter: { attributeGroups: [["old"]], singleQuote: true },
      entries: [{ files: ["old/**/*.vue"] }],
    },
  };
  const own: VueConfigObject = {
    extends: [first],
    plugins: [b],
    server: { host: true },
    vue: { files: [], formatter: { attributeGroups: [], singleQuote: false }, entries: [] },
  };
  const result = await resolveConfig(own, env);
  assert.deepEqual(result.plugins, [a, b]);
  const { mergeConfig } = resolveVitePlus().require("vite-plus") as typeof import("vite-plus");
  const consumer = mergeConfig(mergeConfig({}, { plugins: first.plugins, server: first.server }), {
    plugins: own.plugins,
    server: own.server,
  });
  assert.deepEqual(result.server, consumer.server);
  assert.equal(result.server?.port, 4321);
  assert.equal(result.server?.host, true);
  assert.deepEqual(first.server, { port: 4321 });
  assert.deepEqual(own.server, { host: true });
  assert.deepEqual(result.vue, {
    files: [],
    formatter: { attributeGroups: [], singleQuote: false },
    entries: [],
  });
  assert.deepEqual(first.vue?.files, ["old.vue"]);
  assert.deepEqual(first.vue?.entries, [{ files: ["old/**/*.vue"] }]);
});

void test("canonical native leaves retain disabled flags and explicit compiler/formatter presence", () => {
  const legacy: VueConfigObject = {
    compiler: false,
    typecheck: false,
    lint: { vize: false },
    fmt: { vize: false },
  };
  const inherited = normalizeConfig(
    { ...legacy, vue: { version: "3", typeChecker: {}, linter: {} } },
    {},
  );
  assert.equal(inherited.compiler, false);
  assert.deepEqual(inherited.metadata.options, { check: false, lint: false, fmt: false });
  const selected = normalizeConfig(
    {
      ...legacy,
      vue: {
        compiler: {},
        formatter: {},
        typeChecker: { enabled: true },
        linter: { enabled: true },
      },
    },
    {},
  );
  assert.notEqual(selected.compiler, false);
  assert.deepEqual(selected.metadata.options, { check: true, lint: true, fmt: true });
});

void test("canonical version chooses the exact existing native dialect vocabulary", () => {
  for (const [version, expected] of [
    ["0.10", 0.1],
    ["0.11", 0.11],
    ["1", 1],
    ["2", 2],
    ["2.7", "2.7"],
    ["3", 3],
  ] as const) {
    const result = normalizeConfig(
      { compiler: { vueVersion: 3 }, vue: { version, compiler: { sourceMap: false } } },
      {},
    );
    assert.notEqual(result.compiler, false);
    if (result.compiler !== false) {
      assert.equal(result.compiler.vueVersion, expected);
      assert.equal(result.compiler.compatibility?.vueVersion, expected);
      assert.equal(result.compiler.sourceMap, false);
    }
  }
});

void test("rejections and cycles remain errors without re-evaluating a source", async () => {
  let calls = 0;
  const rejected = withVue(
    {},
    defineVpConfig(async () => {
      calls++;
      throw new Error("original rejection");
    }),
    integration,
  );
  await assert.rejects(rejected(env), /original rejection/);
  assert.equal(calls, 1);
  const cyclic: VueConfigObject = { vue: {} };
  cyclic.extends = cyclic;
  await assert.rejects(defineConfig(cyclic, integration)(env), /Circular extends/);
  await assert.rejects(
    withVue({ formatter: { enabled: false } } as never, {}, integration)(env),
    /unknown option enabled/,
  );
});
