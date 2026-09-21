import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, writeFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { defineConfig, withVue, withVize } from "../vite-plus.ts";
import { taskConfigKey, type ConfigWithVizeTasks, type VueConfigObject } from "./types.ts";
import { resolveConfigExport } from "../config.ts";
import { createTasks } from "./tasks.ts";

assert.equal(withVize, defineConfig);
assert.equal(withVue, defineConfig);

const env = { command: "build", mode: "production" } as const;

void test("defineConfig composes async Vite+ and native config without mutating either input", async () => {
  const config = { linter: { preset: "essential" as const } };
  const plugin = { name: "consumer-plugin" };
  const rules = { "vue/valid-define-props": "warn" as const };
  const source = { plugins: [plugin], lint: { rules }, fmt: { ignorePatterns: ["generated/**"] } };
  const result = await defineConfig(
    async (context) => {
      assert.equal(context.mode, "production");
      return { ...source, vize: config };
    },
    { plugin: false, tasks: false },
  )(env);
  assert.ok(
    !("vize" in result),
    "native configuration is not forwarded as an unknown Vite+ option",
  );
  assert.deepEqual(result.plugins, [plugin]);
  assert.equal(result.lint?.rules?.["vue/valid-define-props"], "warn");
  assert.equal(result.lint?.rules?.["vue/no-export-in-script-setup"], "off");
  assert.equal(result.lint?.jsPlugins, undefined);
  assert.ok(result.lint?.plugins?.includes("typescript"));
  assert.deepEqual(result.fmt?.ignorePatterns, ["generated/**", "**/node_modules/**", "**/*.vue"]);
  assert.deepEqual(source.fmt.ignorePatterns, ["generated/**"]);
  assert.deepEqual(source.lint.rules, rules);
  assert.deepEqual(Object.keys(config), ["linter"]);
  const metadata = (result as ConfigWithVizeTasks)[taskConfigKey]!;
  assert.equal((await resolveConfigExport(metadata.config!)).linter?.preset, "essential");
});

void test("tool ownership and conflict defaults can be disabled independently", async () => {
  const vp = { lint: { plugins: ["eslint" as const] }, fmt: { printWidth: 90 } };
  for (const options of [{ conflicts: false }, { lint: false, fmt: false }]) {
    const config = await defineConfig(vp, { ...options, plugin: false, tasks: false })(env);
    assert.deepEqual(config.lint, vp.lint);
    assert.deepEqual(config.fmt, vp.fmt);
  }
});

void test("defineConfig installs its compiler and preserves caller plugins", async () => {
  const previousCompiler = { name: "vite:vue" };
  const config = await defineConfig(
    {
      plugins: [Promise.resolve([previousCompiler]), { name: "consumer" }],
    },
    { tasks: false },
  )(env);
  const plugins = ((config.plugins ?? []) as unknown[]).flat(Infinity) as { name: string }[];
  assert.ok(plugins.some((plugin) => plugin.name === "vite-plugin-vize"));
  assert.ok(!plugins.includes(previousCompiler));
  assert.equal(plugins.at(-1)?.name, "consumer");
});

void test("generated tasks preserve existing scripts and tasks, and reject ambiguous renames", () => {
  const cwd = process.cwd();
  const dir = mkdtempSync(path.join(os.tmpdir(), "vize tasks 'space-"));
  try {
    process.chdir(dir);
    writeFileSync("package.json", JSON.stringify({ scripts: { check: "existing-check" } }));
    const tasks = createTasks({ test: "custom-test" }, { build: "bundle", preview: false });
    assert.ok(tasks["vize:check"]);
    assert.equal(tasks.check, undefined);
    assert.equal(tasks.test, undefined);
    assert.equal(tasks.preview, undefined);
    assert.ok(tasks.bundle);
    assert.equal(
      typeof tasks.lint === "object" && "cache" in tasks.lint && tasks.lint.cache,
      false,
    );
    assert.throws(() => createTasks({}, { lint: "check" }), /already exists/);
    assert.throws(() => createTasks({}, { lint: "same", fmt: "same" }), /already exists/);
    assert.deepEqual(createTasks({}, false), {});
  } finally {
    process.chdir(cwd);
    rmSync(dir, { recursive: true, force: true });
  }
});

void test("each consumer's Vite+ selects the available overlap rules", async () => {
  const cwd = process.cwd();
  for (const rule of ["no-export-in-script-setup", "valid-define-props"]) {
    const directory = mkdtempSync(path.join(os.tmpdir(), "vize-consumer-peer-"));
    try {
      const peer = path.join(directory, "node_modules/vite-plus");
      mkdirSync(peer, { recursive: true });
      writeFileSync(
        path.join(peer, "package.json"),
        JSON.stringify({
          name: "vite-plus",
          exports: { "./package.json": "./package.json" },
          bin: { vp: "cli.cjs" },
        }),
      );
      writeFileSync(
        path.join(peer, "cli.cjs"),
        `console.log(${JSON.stringify(JSON.stringify([{ scope: "vue", value: rule }]))});`,
      );
      process.chdir(directory);
      const config = await defineConfig({}, { plugin: false, tasks: false })(env);
      assert.deepEqual(config.lint?.rules, { [`vue/${rule}`]: "off" });
    } finally {
      process.chdir(cwd);
      rmSync(directory, { recursive: true, force: true });
    }
  }
});

void test("extends merges native sections and Vite+ options with deterministic alias precedence", async () => {
  const base = defineConfig({
    compiler: false,
    server: { port: 4321 },
    vize: { lint: { preset: "essential", typecheck: true } },
    fmt: { vize: { singleQuote: true }, printWidth: 90 },
  });
  const own: VueConfigObject = {
    extends: [base, Promise.resolve({ server: { host: true } })],
    lint: { vize: { typecheck: false, rules: { "vue/no-v-html": "error" } } },
    typecheck: { strict: true },
    fmt: { vize: { tabWidth: 4 } },
  };
  const result = await defineConfig(own, { tasks: false })(env);
  assert.equal(result.server?.port, 4321);
  assert.equal(result.server?.host, true);
  assert.equal(result.fmt?.printWidth, 90);
  for (const key of ["vize", "compiler", "typecheck", "extends"]) assert.ok(!(key in result));
  assert.ok(!("vize" in result.lint!));
  assert.ok(!("vize" in result.fmt!));
  const metadata = (result as ConfigWithVizeTasks)[taskConfigKey]!;
  assert.equal(metadata.lintTypecheck, false);
  const native = await resolveConfigExport(metadata.config!);
  assert.equal(native.linter?.preset, "essential");
  assert.equal(native.linter?.rules?.["vue/no-v-html"], "error");
  assert.equal(native.formatter?.singleQuote, true);
  assert.equal(native.formatter?.tabWidth, 4);
  assert.equal(native.typeChecker?.strict, true);
  assert.equal(
    own.lint?.vize && typeof own.lint.vize === "object" && own.lint.vize.typecheck,
    false,
  );
  const cyclic: VueConfigObject = {};
  cyclic.extends = cyclic;
  await assert.rejects(defineConfig(cyclic, { plugin: false })(env), /Circular extends/);
});

void test("either lint typecheck spelling enables one native checker, and false opts out", async () => {
  for (const source of [
    { lint: { vize: { typecheck: true } } },
    { vize: { lint: { typecheck: true } } },
  ]) {
    const result = await defineConfig(source, { plugin: false, tasks: false })(env);
    assert.equal((result as ConfigWithVizeTasks)[taskConfigKey]?.lintTypecheck, true);
  }
  const result = await defineConfig(
    {
      compiler: false,
      typecheck: false,
      lint: { vize: false },
      fmt: { vize: false },
    },
    { tasks: false },
  )(env);
  assert.deepEqual((result as ConfigWithVizeTasks)[taskConfigKey]?.options, {
    tasks: false,
    check: false,
    lint: false,
    fmt: false,
  });
  assert.deepEqual(result.plugins, []);
  assert.equal(result.fmt?.ignorePatterns, undefined);
});
