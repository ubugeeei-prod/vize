import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  DEFAULT_PRECOMPILE_BATCH_SIZE,
  DEFAULT_PRECOMPILE_IGNORE_PATTERNS,
  type VizePluginState,
} from "./state.ts";
import { compileAll } from "./precompile-run.ts";

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

const workspaceRootForScan = fs.mkdtempSync(path.join(testRoot, "workspace-scan-"));
const workspaceSourceRoot = path.join(workspaceRootForScan, "src");
fs.mkdirSync(workspaceSourceRoot, { recursive: true });
const workspaceApp = path.join(workspaceSourceRoot, "App.vue");
fs.writeFileSync(workspaceApp, `<template><div>workspace</div></template>`);

const nestedNodeModulesSfc = path.join(
  workspaceRootForScan,
  "packages",
  "app",
  "node_modules",
  "linked",
  "Nested.vue",
);
fs.mkdirSync(path.dirname(nestedNodeModulesSfc), { recursive: true });
fs.writeFileSync(nestedNodeModulesSfc, `<template><div>nested dependency</div></template>`);

const externalPackage = fs.mkdtempSync(path.join(testRoot, "external-package-"));
const externalSfc = path.join(externalPackage, "Linked.vue");
fs.writeFileSync(externalSfc, `<template><div>external dependency</div></template>`);
const symlinkPath = path.join(workspaceRootForScan, "packages", "app", "linked-package");
let symlinkCreated = false;
try {
  fs.symlinkSync(externalPackage, symlinkPath, "dir");
  symlinkCreated = true;
} catch {
  // Some Windows developer shells cannot create directory symlinks. The nested
  // node_modules assertion still covers the default workspace guard.
}

const workspaceState: VizePluginState = {
  ...state,
  cache: new Map(),
  ssrCache: new Map(),
  collectedCss: new Map(),
  precompileMetadata: new Map(),
  pendingHmrUpdateTypes: new Map(),
  root: workspaceRootForScan,
  scanPatterns: ["**/*.vue"],
  ignorePatterns: [...DEFAULT_PRECOMPILE_IGNORE_PATTERNS],
};

await compileAll(workspaceState);

assert.ok(workspaceState.cache.has(workspaceApp), "First-party SFCs should still pre-compile");
assert.equal(
  workspaceState.cache.has(nestedNodeModulesSfc),
  false,
  "Pre-compilation should ignore nested node_modules in pnpm workspaces",
);
if (symlinkCreated) {
  assert.equal(
    [...workspaceState.cache.keys()].some((filePath) => filePath.includes("Linked.vue")),
    false,
    "Pre-compilation should not follow directory symlinks during workspace scans",
  );
}

console.log("✅ vite-plugin-vize precompile tests passed!");
