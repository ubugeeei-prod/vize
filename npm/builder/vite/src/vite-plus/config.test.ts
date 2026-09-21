import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { withVize } from "../vite-plus.ts";
import { taskConfigKey } from "./types.ts";
import { createTasks } from "./tasks.ts";

const env = { command: "build", mode: "production" } as const;

void test("withVize composes async Vite+ config without mutating either input", async () => {
  const config = { linter: { preset: "essential" as const } };
  const plugin = { name: "consumer-plugin" };
  const rules = { "vue/valid-define-props": "warn" as const };
  const source = { plugins: [plugin], lint: { rules }, fmt: { ignorePatterns: ["generated/**"] } };
  const result = await withVize(config, { plugin: false, tasks: false }).vp(async (context) => {
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
    const config = await withVize({}, { ...options, plugin: false, tasks: false }).vp(vp)(env);
    assert.equal(config.lint, vp.lint);
    assert.equal(config.fmt, vp.fmt);
  }
});

void test("bare withVize installs its compiler and preserves caller plugins", async () => {
  const config = await withVize({}, { tasks: false }).vp({ plugins: [{ name: "consumer" }] })(env);
  const plugins = (config.plugins ?? []).flat(Infinity) as { name: string }[];
  assert.ok(plugins.some((plugin) => plugin.name === "vite-plugin-vize"));
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
