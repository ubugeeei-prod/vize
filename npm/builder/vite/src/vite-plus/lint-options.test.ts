import assert from "node:assert/strict";
import { test } from "node:test";
import { defineConfig } from "../vite-plus.ts";
import { resolveConfigExport } from "../config.ts";
import { runTools } from "./runner.ts";
import { taskConfigKey, type ConfigWithVizeTasks } from "./types.ts";

void test("lint.vue carries execution settings through native config without forwarding them to Oxlint", async () => {
  const linter = {
    preset: "opinionated" as const,
    crossFile: true,
    strictReactivity: true,
    crossFileTree: true,
    crossFileComplexity: true,
    maxWarnings: 0,
    reportUnusedDisableDirectives: true,
  };
  const resolved = await defineConfig(
    { lint: { vue: linter } },
    { plugin: false, tasks: false },
  )({ command: "build", mode: "production" });
  const metadata = (resolved as ConfigWithVizeTasks)[taskConfigKey]!;
  assert.deepEqual((await resolveConfigExport(metadata.config!)).linter, linter);
  const calls: string[][] = [];
  await runTools("lint", ["src"], metadata, "vp", "native", async (_command, args, stdin) => {
    calls.push(args);
    if (args[0] === "native") {
      assert.deepEqual(JSON.parse(stdin!).vue.linter, linter);
    }
    return 0;
  });
  assert.deepEqual(calls, [
    ["native", "lint", "--config-stdin", "src"],
    ["vp", "lint", "src"],
  ]);
  for (const key of Object.keys(linter)) assert.ok(!(key in (resolved.lint ?? {})));
});
