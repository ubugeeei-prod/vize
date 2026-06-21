import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { compileAll, DEFAULT_PRECOMPILE_BATCH_SIZE, type VizePluginState } from "./state.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const workspaceRoot = path.resolve(__dirname, "../../../..");
const testRoot = path.join(
  workspaceRoot,
  "target",
  "vize-tests",
  "tests",
  "vite-plugin-vize",
  "precompile",
);
fs.mkdirSync(testRoot, { recursive: true });

const root = fs.mkdtempSync(path.join(testRoot, "tsx-scan-"));
const sourceRoot = path.join(root, "src");
fs.mkdirSync(sourceRoot, { recursive: true });

const appPath = path.join(sourceRoot, "App.vue");
const storyPath = path.join(sourceRoot, "Button.stories.tsx");

fs.writeFileSync(appPath, `<template><div>ok</div></template>`);
fs.writeFileSync(
  storyPath,
  `type Meta = typeof Button;
const Button = {};
export const Basic = () => <Button />;
`,
);

const state: VizePluginState = {
  cache: new Map(),
  ssrCache: new Map(),
  collectedCss: new Map(),
  precompileMetadata: new Map(),
  pendingHmrUpdateTypes: new Map(),
  isProduction: false,
  root,
  clientViteBase: "/",
  serverViteBase: "/",
  server: null,
  filter: () => true,
  scanPatterns: ["src/**/*.vue", "src/**/*.tsx"],
  precompileBatchSize: DEFAULT_PRECOMPILE_BATCH_SIZE,
  ignorePatterns: [],
  mergedOptions: {},
  initialized: true,
  dynamicImportAliasRules: [],
  cssAliasRules: [],
  extractCss: false,
  componentsCssFileName: "assets/vize-components.css",
  clientViteDefine: {},
  serverViteDefine: {},
  logger: {
    log() {},
    info() {},
    warn() {},
    error() {},
  } as never,
};

await compileAll(state);

assert.ok(state.cache.has(appPath), "Vue scan matches should still be pre-compiled");
assert.equal(
  state.cache.has(storyPath),
  false,
  "TSX scan matches should stay out of the SFC precompile cache",
);
assert.equal(
  state.precompileMetadata.has(storyPath),
  false,
  "TSX scan matches should not get SFC precompile metadata",
);

console.log("✅ vite-plugin-vize precompile tests passed!");
