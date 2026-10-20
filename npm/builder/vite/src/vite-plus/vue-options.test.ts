import assert from "node:assert/strict";
import { test } from "node:test";
import { defineConfig as defineVpConfig, type ConfigEnv } from "vite-plus";
import { defineConfig, withVue } from "../vite-plus.ts";
import { resolveConfigExport, resolveViteConfigExport } from "../config.ts";
import { projectViteConfigSource } from "../../../../cli/src/config/vite-runtime.mjs";
import { normalizeConfig, resolveConfig } from "./config.ts";
import { runTools } from "./runner.ts";
import {
  sourceConfigKey,
  taskConfigKey,
  type ConfigWithVizeTasks,
  type VueConfigObject,
} from "./types.ts";

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
    const wrapped = defineConfig(original, integration).withVue({});
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
    fmt: { vue: { printWidth: 70, singleQuote: true } },
    typecheck: { strict: true },
    vue: {
      version: "3",
      formatter: { singleQuote: false },
      typeChecker: { tsconfig: "tsconfig.json", strict: false, enabled: false },
      linter: { enabled: false },
      languageServer: { hover: false },
      files: ["src/**/*.vue"],
      entries: [
        {
          name: "widgets",
          files: ["widgets/**/*.vue"],
          compiler: { whitespace: "condense" },
          linter: { rules: { "vue/no-v-html": "off" } },
        },
      ],
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
  assert.deepEqual(
    await resolveViteConfigExport(result, env, process.cwd()),
    direct,
    "an evaluated fluent export must project the same Vue settings as its raw source",
  );
  assert.equal(native.formatter?.singleQuote, false);
  assert.equal(native.formatter?.printWidth, 70);
  assert.equal(native.typeChecker?.strict, false);
  assert.equal(metadata.options.check, false);
  assert.equal(metadata.options.lint, false);
  assert.equal(metadata.options.fmt, true);
  assert.ok(!("vue" in result));
});

void test("action options route flat lint to Vite+ and native settings to the loader", async () => {
  const hostLint = {
    jsPlugins: ["oxlint-plugin-vize"],
    plugins: ["vue"],
    ignorePatterns: ["dist/**"],
    rules: {
      "no-console": "warn",
      "vize/vue/no-v-html": "warn",
    },
    settings: {
      vize: { preset: "incremental", locale: "ja", helpLevel: "short" },
    },
  };
  const { vp: result, metadata } = normalizeConfig(
    {
      lint: { rules: { "no-console": "off" } },
      vue: {
        compile: { whitespace: "preserve" },
        typeCheck: { strict: true },
        lint: hostLint,
        fmt: { singleQuote: true },
        entries: [
          {
            files: ["generated/**/*.vue"],
            linter: { rules: { "vue/no-v-html": "off" } },
          },
        ],
      },
    },
    integration,
  );
  assert.equal(result.lint, hostLint, "withVue lint takes ownership of the host Vite+ block");
  assert.equal(metadata.options.plugin, false);
  assert.equal(metadata.options.tasks, false);
  assert.equal(metadata.options.lint, false);
  assert.equal(metadata.options.fmt, true);
  assert.equal(metadata.options.conflicts, false);
  assert.equal(metadata.lintLocale, "ja");
  assert.equal(metadata.lintHelpLevel, "short");
  const native = await resolveConfigExport(metadata.config!, env);
  assert.deepEqual(
    {
      typeChecker: native.typeChecker,
      compiler: native.compiler,
      linter: native.linter,
      formatter: native.formatter,
    },
    {
      typeChecker: { jsxTypecheck: true, strict: true },
      compiler: { whitespace: "preserve" },
      linter: { preset: "incremental", rules: { "vue/no-v-html": "warn" } },
      formatter: { singleQuote: true },
    },
  );
  assert.deepEqual(native.entries?.at(-1), {
    basePath: process.cwd(),
    files: ["generated/**/*.vue"],
    linter: { rules: { "vue/no-v-html": "off" } },
  });
  const calls: string[][] = [];
  await runTools("lint", ["src"], metadata, "vp", "native", async (_command, args) => {
    calls.push(args);
    return 0;
  });
  assert.deepEqual(calls, [["vp", "lint", "src"]], "the host bridge owns the native lint pass");
});

void test("fluent Vue settings merge native leaves and replace canonical arrays immutably", async () => {
  const plugin = { name: "consumer" };
  const source: VueConfigObject = {
    plugins: [plugin],
    server: { port: 4321 },
    vue: {
      files: ["old.vue"],
      formatter: { printWidth: 80, attributeGroups: [["old"]], singleQuote: true },
      entries: [{ files: ["old/**/*.vue"] }],
    },
  };
  const base = defineConfig(source, integration);
  const fluent = base.withVue({
    files: [],
    formatter: { attributeGroups: [], singleQuote: false },
    entries: [],
  });
  const result = await resolveConfig(fluent, env);
  assert.deepEqual(result.plugins, [plugin]);
  assert.equal(result.plugins?.[0], plugin);
  assert.equal(result.server, source.server);
  assert.deepEqual(result.vue, {
    files: [],
    formatter: { printWidth: 80, attributeGroups: [], singleQuote: false },
    entries: [],
  });
  assert.deepEqual((await resolveConfig(base, env)).vue, source.vue);
  assert.deepEqual(source.vue?.files, ["old.vue"]);
  assert.deepEqual(source.vue?.entries, [{ files: ["old/**/*.vue"] }]);
  const chained = fluent.withVue({ formatter: { tabWidth: 4 } });
  assert.deepEqual((await resolveConfig(chained, env)).vue?.formatter, {
    printWidth: 80,
    attributeGroups: [],
    singleQuote: false,
    tabWidth: 4,
  });
  assert.equal((await resolveConfig(fluent, env)).vue?.formatter?.tabWidth, undefined);
});

void test("the standalone helper delegates to fluent Vue composition", async () => {
  const source = { server: { host: true } };
  assert.deepEqual(
    await resolveConfig(withVue({ formatter: { tabWidth: 4 } }, source, integration), env),
    await resolveConfig(
      defineConfig(source, integration).withVue({ formatter: { tabWidth: 4 } }),
      env,
    ),
  );
});

void test("canonical native leaves retain disabled flags and explicit compiler/formatter presence", () => {
  const legacy: VueConfigObject = {
    compiler: false,
    typecheck: false,
    lint: { vue: false },
    fmt: { vue: false },
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
  assert.equal(selected.compiler, false);
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

void test("rejections remain errors without re-evaluating a source", async () => {
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
  await assert.rejects(
    withVue({ formatter: { enabled: false } } as never, {}, integration)(env),
    /unknown option "enabled"/,
  );
});

void test("fluent composition preserves caller tasks and explicit tool conflict ownership", async () => {
  const tasks = { check: "custom-check", lint: "custom-lint" };
  const lint = { rules: { "vue/valid-define-props": "warn" as const } };
  const result = await defineConfig(
    { run: { tasks }, lint },
    { plugin: false, conflicts: false },
  ).withVue({ typeChecker: { enabled: false }, linter: { enabled: false } })(env);
  assert.equal(result.run?.tasks?.check, "custom-check");
  assert.equal(result.run?.tasks?.lint, "custom-lint");
  assert.deepEqual(result.lint, lint);
  assert.deepEqual(tasks, { check: "custom-check", lint: "custom-lint" });
  assert.equal((result as ConfigWithVizeTasks)[taskConfigKey]?.options.check, false);
  assert.equal((result as ConfigWithVizeTasks)[taskConfigKey]?.options.lint, false);
});

void test("pack.vue is consumed without forwarding native settings to Vite+", async () => {
  const source = { pack: [{ outDir: "lib", vue: false as const }] };
  const result = await defineConfig(source, integration).withVue({})(env);
  assert.deepEqual(result.pack, [{ outDir: "lib" }]);
  assert.deepEqual(source.pack, [{ outDir: "lib", vue: false }]);
});

void test("evaluated fluent output retains its raw Vue source without another callback", async () => {
  let calls = 0;
  const plugin = { name: "consumer" };
  const output = await defineConfig(() => {
    calls++;
    return { root: "gallery", plugins: [plugin] };
  }, integration).withVue({ formatter: { singleQuote: true } })(env);
  const source = Reflect.get(output, sourceConfigKey) as VueConfigObject;
  assert.equal(typeof source, "object");
  assert.equal(source.root, "gallery");
  assert.deepEqual(source.vue, { formatter: { singleQuote: true } });
  assert.equal(source.plugins?.[0], plugin);
  assert.ok(!("vue" in output));
  assert.equal(calls, 1);
});
