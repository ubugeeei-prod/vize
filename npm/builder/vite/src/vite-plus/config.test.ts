import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, writeFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { withVue, withVize } from "../vite-plus.ts";
import { taskConfigKey } from "./types.ts";
import { createTasks } from "./tasks.ts";

assert.equal(withVize, withVue, "withVize must remain the same configurable helper");

const env = { command: "build", mode: "production" } as const;

void test("withVue composes async Vite+ config without mutating either input", async () => {
  const config = { linter: { preset: "essential" as const } };
  const plugin = { name: "consumer-plugin" };
  const rules = { "vue/valid-define-props": "warn" as const };
  const source = { plugins: [plugin], lint: { rules }, fmt: { ignorePatterns: ["generated/**"] } };
  const result = await withVue(config, { plugin: false, tasks: false }).vp(async (context) => {
    assert.equal(context.mode, "production");
    return source;
  })(env);
  assert.deepEqual(result.plugins, [plugin]);
  assert.equal(result.lint?.rules?.["vue/valid-define-props"], "warn");
  assert.equal(result.lint?.rules?.["vue/no-export-in-script-setup"], "off");
  assert.equal(result.lint?.jsPlugins, undefined);
  assert.ok(result.lint?.plugins?.includes("typescript"));
  assert.deepEqual(result.fmt?.ignorePatterns, ["generated/**", "**/node_modules/**", "**/*.vue"]);
  assert.deepEqual(source.fmt.ignorePatterns, ["generated/**"]);
  assert.deepEqual(source.lint.rules, rules);
  assert.deepEqual(Object.keys(config), ["linter"]);
  assert.equal(
    (result as { [taskConfigKey]?: { config: unknown } })[taskConfigKey]?.config,
    config,
  );
});

void test("tool ownership and conflict defaults can be disabled independently", async () => {
  const vp = { lint: { plugins: ["eslint" as const] }, fmt: { printWidth: 90 } };
  for (const options of [{ conflicts: false }, { lint: false, fmt: false }]) {
    const config = await withVue({}, { ...options, plugin: false, tasks: false }).vp(vp)(env);
    assert.equal(config.lint, vp.lint);
    assert.equal(config.fmt, vp.fmt);
  }
});

void test("bare withVue installs its compiler and preserves caller plugins", async () => {
  const previousCompiler = { name: "vite:vue" };
  const config = await withVue({}, { tasks: false }).vp({
    plugins: [Promise.resolve([previousCompiler]), { name: "consumer" }],
  })(env);
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
      const config = await withVue({}, { plugin: false, tasks: false })(env);
      assert.deepEqual(config.lint?.rules, { [`vue/${rule}`]: "off" });
    } finally {
      process.chdir(cwd);
      rmSync(directory, { recursive: true, force: true });
    }
  }
});
