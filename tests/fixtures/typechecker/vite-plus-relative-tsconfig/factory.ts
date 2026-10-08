import assert from "node:assert/strict";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { taskConfigKey } from "../../../../npm/builder/vite/src/vite-plus/types.ts";
import type {
  ConfigWithVizeTasks,
  VizePlusConfigFactory,
} from "../../../../npm/builder/vite/src/vite-plus/types.ts";
import { repoRoot } from "./fixture.ts";

export async function sourceTaskConfig(input: string) {
  const sourceProvider = pathToFileURL(
    path.join(repoRoot, "npm/builder/vite/src/vite-plus.ts"),
  ).href;
  const providerInput = input.replace(
    '"@vizejs/vite-plugin/vite-plus"',
    JSON.stringify(sourceProvider),
  );
  assert.notEqual(providerInput, input);
  const loaded = (await import(
    `data:text/javascript;base64,${Buffer.from(providerInput).toString("base64")}`
  )) as { default: VizePlusConfigFactory };
  // Only provider package resolution changes; every defineConfig argument survives.
  const configured = (await loaded.default({
    command: "build",
    mode: "production",
  })) as ConfigWithVizeTasks;
  const metadata = configured[taskConfigKey];
  assert(metadata);
  return { metadata, providerInput, sourceProvider };
}
