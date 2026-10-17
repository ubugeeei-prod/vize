import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { defineConfig } from "../vite-plus.ts";
import { resolveConfigExport } from "../config.ts";
import { runNative } from "./runner.ts";
import { taskConfigKey } from "./types.ts";

void test("Vite+ tasks use one project projection with JSX defaults and explicit overrides", async () => {
  const cwd = process.cwd();
  const root = mkdtempSync(path.join(os.tmpdir(), "vize-project-defaults-"));
  try {
    process.chdir(root);
    const config = await defineConfig(
      { fmt: { printWidth: 90, vue: { singleQuote: true } } },
      { plugin: false, tasks: false, conflicts: false },
    )({ command: "build", mode: "production" });
    const metadata = config[taskConfigKey]!;
    const formatter = await resolveConfigExport(metadata.config!, {
      command: "fmt",
      mode: "production",
    });
    assert.equal(formatter.typeChecker?.jsxTypecheck, true);
    assert.deepEqual(formatter.formatter, { singleQuote: true });
    let checks = 0;
    assert.equal(
      await runNative("check", [], metadata, "native", async (_command, argv) => {
        const transported = JSON.parse(readFileSync(argv[3], "utf8"));
        assert.equal(transported.typeChecker.jsxTypecheck, true);
        assert.equal(transported.formatter.singleQuote, true);
        checks++;
        return 0;
      }),
      0,
    );
    assert.equal(checks, 1);

    const explicit = await defineConfig(
      { typecheck: { jsxTypecheck: false } },
      { plugin: false, tasks: false, conflicts: false },
    )({ command: "build", mode: "production" });
    assert.equal(
      (
        await resolveConfigExport(explicit[taskConfigKey]!.config!, {
          command: "check",
          mode: "production",
        })
      ).typeChecker?.jsxTypecheck,
      false,
    );

    const standard = await defineConfig(
      {},
      { plugin: false, tasks: false, conflicts: false },
    )({ command: "build", mode: "production" });
    const projected = await resolveConfigExport(standard[taskConfigKey]!.config!, {
      command: "check",
      mode: "production",
    });
    assert.equal(projected.typeChecker?.jsxTypecheck, true);
  } finally {
    process.chdir(cwd);
    rmSync(root, { recursive: true, force: true });
  }
});
