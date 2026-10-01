import assert from "node:assert/strict";
import { test } from "node:test";
import { defineConfig } from "../vite-plus.ts";
import { resolveConfigExport } from "../config.ts";
import { taskConfigKey, type ConfigWithVizeTasks } from "./types.ts";

void test("Vite+ shares Oxfmt import sorting with native Vue formatting", async () => {
  const sorting = {
    groups: ["external", ["internal", "sibling"]],
    internalPattern: ["project/"],
    newlinesBetween: false,
    order: "desc" as const,
  };
  const result = await defineConfig(
    { fmt: { sortImports: sorting } },
    { plugin: false, tasks: false },
  )({ command: "build", mode: "production" });
  const metadata = (result as ConfigWithVizeTasks)[taskConfigKey]!;
  assert.deepEqual((await resolveConfigExport(metadata.config!)).formatter?.sortImports, sorting);
  assert.deepEqual(result.fmt?.sortImports, sorting);
  assert.ok(result.fmt?.ignorePatterns?.includes("**/*.vue"));
});

void test("explicit fmt.vize sorting overrides inherited Oxfmt settings, including false", async () => {
  for (const override of [false, { order: "desc" as const }] as const) {
    const result = await defineConfig(
      { fmt: { sortImports: { order: "asc" }, vize: { sortImports: override } } },
      { plugin: false, tasks: false },
    )({ command: "build", mode: "production" });
    const metadata = (result as ConfigWithVizeTasks)[taskConfigKey]!;
    assert.deepEqual(
      (await resolveConfigExport(metadata.config!)).formatter?.sortImports,
      override,
    );
    assert.deepEqual(result.fmt?.sortImports, { order: "asc" });
  }
});
