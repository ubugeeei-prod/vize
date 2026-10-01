import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { defineConfig } from "../vite-plus.ts";
import { resolveConfigExport } from "../config.ts";
import { runTools } from "./runner.ts";
import { taskConfigKey, type ConfigWithVizeTasks } from "./types.ts";

void test("lint.vize carries execution settings through native config without forwarding them to Oxlint", async () => {
  const linter = {
    preset: "opinionated" as const,
    crossFile: true,
    strictReactivity: true,
    crossFileTree: true,
    crossFileComplexity: true,
    maxWarnings: 0,
  };
  const resolved = await defineConfig(
    { lint: { vize: linter } },
    { plugin: false, tasks: false },
  )({ command: "build", mode: "production" });
  const metadata = (resolved as ConfigWithVizeTasks)[taskConfigKey]!;
  assert.deepEqual((await resolveConfigExport(metadata.config!)).linter, linter);
  const calls: string[][] = [];
  await runTools("lint", ["src"], metadata, "vp", "native", async (_command, args) => {
    calls.push(args);
    if (args[0] === "native") {
      const file = args[args.indexOf("--config") + 1];
      assert.deepEqual(JSON.parse(readFileSync(file, "utf8")).linter, linter);
    }
    return 0;
  });
  assert.deepEqual(
    calls.map((args) =>
      args[0] === "native" ? [...args.slice(0, 3), "<temporary config>", ...args.slice(4)] : args,
    ),
    [
      ["native", "lint", "--config", "<temporary config>", "src"],
      ["vp", "lint", "src"],
    ],
  );
  for (const key of Object.keys(linter)) assert.ok(!(key in (resolved.lint ?? {})));
});
