import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { createTasks } from "./tasks.ts";

void test("task cache is opt-in and only checks cache without restoring outputs", () => {
  const previous = process.cwd();
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-task-cache-"));
  try {
    process.chdir(directory);
    const peer = path.join(directory, "node_modules/vite-plus");
    mkdirSync(peer, { recursive: true });
    for (const version of [
      "0.2.3",
      "0.3.3",
      "1.0.0-rc.0",
      "1.0.0-rc.1",
      "1.0.0",
      "1.4.0",
      "2.0.0",
    ]) {
      writeFileSync(
        path.join(peer, "package.json"),
        JSON.stringify({
          name: "vite-plus",
          version,
          exports: { "./package.json": "./package.json" },
        }),
      );
      writeFileSync(
        "package.json",
        JSON.stringify({ packageManager: "pnpm@12.1.0", scripts: { check: "existing" } }),
      );
      const existing = { lint: { command: "custom lint", cache: false } };
      const tasks = createTasks(existing, { typecheck: "types", preview: false }, true);
      assert.equal(tasks.lint, undefined, "existing tasks retain their cache policy");
      assert.equal(tasks.check, undefined, "script aliases keep their original behavior");
      assert.equal(tasks.preview, undefined);
      const cache = {
        input: [
          { auto: true },
          "!**/node_modules",
          "!**/node_modules/**",
          "!**/.vize/**",
          "pnpm-lock.yaml",
        ],
        output: [],
      };
      const nested = !version.startsWith("0.") && version !== "1.0.0-rc.0";
      for (const name of ["vize:check", "types", "fmt:check"]) {
        const task = tasks[name];
        assert.equal(typeof task, "object");
        assert.deepEqual(
          Object.fromEntries(Object.entries(task!).filter(([key]) => key !== "command")),
          nested ? { cache } : { cache: true, ...cache },
          `${version}: ${name}`,
        );
      }
      for (const name of ["editor:setup", "lint:fix", "fmt", "build", "pack", "dev", "test"]) {
        const task = tasks[name];
        assert.equal(typeof task === "object" && "cache" in task && task.cache, false, name);
      }
      for (const task of Object.values(createTasks())) {
        assert.equal(
          typeof task === "object" && "cache" in task && task.cache,
          false,
          "default stays uncached",
        );
      }
    }
    const lockfileInputs = (packageManager?: string, present: string[] = []): unknown => {
      for (const file of [
        "bun.lock",
        "bun.lockb",
        "npm-shrinkwrap.json",
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
      ]) {
        rmSync(file, { force: true });
      }
      for (const file of present) writeFileSync(file, "lockfile\n");
      writeFileSync(
        "package.json",
        JSON.stringify({ ...(packageManager ? { packageManager } : {}), scripts: {} }),
      );
      const task = createTasks({}, {}, true)["fmt:check"]!;
      if (typeof task !== "object") assert.fail("fmt:check is an object task");
      const cache = (typeof task.cache === "object" ? task.cache : task) as { input: unknown };
      return cache.input;
    };
    const common = [{ auto: true }, "!**/node_modules", "!**/node_modules/**", "!**/.vize/**"];
    assert.deepEqual(lockfileInputs("bun@1.3.14"), [...common, "bun.lock", "bun.lockb"]);
    assert.deepEqual(lockfileInputs("npm@11.11.0"), [
      ...common,
      "npm-shrinkwrap.json",
      "package-lock.json",
    ]);
    assert.deepEqual(lockfileInputs("pnpm@12.1.0"), [...common, "pnpm-lock.yaml"]);
    assert.deepEqual(lockfileInputs("yarn@4.12.0"), [...common, "yarn.lock"]);
    assert.deepEqual(lockfileInputs(undefined, ["bun.lock"]), [...common, "bun.lock"]);
    assert.deepEqual(lockfileInputs(undefined, ["package-lock.json", "yarn.lock"]), [
      ...common,
      "package-lock.json",
      "yarn.lock",
    ]);
    assert.deepEqual(lockfileInputs(), [
      ...common,
      "bun.lock",
      "bun.lockb",
      "npm-shrinkwrap.json",
      "package-lock.json",
      "pnpm-lock.yaml",
      "yarn.lock",
    ]);
    assert.deepEqual(createTasks({}, false, true), {});
  } finally {
    process.chdir(previous);
    rmSync(directory, { recursive: true, force: true });
  }
});
