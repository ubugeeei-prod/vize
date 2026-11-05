import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { CompiledModule } from "../types.ts";
import type { VizePluginState } from "./state.ts";
import { loadHook } from "./load.ts";

const emptyState: VizePluginState = {
  cache: new Map(),
  ssrCache: new Map(),
  collectedCss: new Map(),
  precompileMetadata: new Map(),
  pendingHmrUpdateTypes: new Map(),
  isProduction: true,
  root: "/project",
  clientViteBase: "/",
  serverViteBase: "/",
  server: null,
  filter: () => false,
  scanPatterns: null,
  precompileBatchSize: 128,
  ignorePatterns: [],
  mergedOptions: { exclude: [/node_modules/] },
  initialized: true,
  dynamicImportAliasRules: [],
  cssAliasRules: [],
  extractCss: true,
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

function moduleWith(block: CompiledModule["customBlocks"]): CompiledModule {
  return {
    code: "export default { name: 'CompiledComponent' }",
    scopeId: "msg12345",
    hasScoped: false,
    customBlocks: block,
  };
}

function loadedCode(value: ReturnType<typeof loadHook>): string {
  if (typeof value === "string") return value;
  if (value && typeof value === "object") return value.code;
  return "";
}

const vueFile = "/src/App.vue";
const content = "Hello from custom block\n";
const state: VizePluginState = {
  ...emptyState,
  cache: new Map([
    [vueFile, moduleWith([{ type: "message", content, src: null, attrs: {}, index: 0 }])],
  ]),
};

for (const id of [`${vueFile}?vue&type=message&index=0`, `${vueFile}?vue=&type=message&index=0`]) {
  const code = loadedCode(loadHook(state, id, { ssr: false }));
  assert.equal(code, content, id);
  assert.doesNotMatch(
    code,
    /export default/,
    `${id} must return the raw block, not the compiled component`,
  );
}

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-custom-block-"));
try {
  const srcFile = path.join(dir, "message.txt");
  const srcOwner = path.join(dir, "App.vue");
  fs.writeFileSync(srcFile, "from src file\n");
  const watched: string[] = [];
  const srcState: VizePluginState = {
    ...emptyState,
    root: dir,
    cache: new Map([
      [
        srcOwner,
        moduleWith([
          {
            type: "message",
            content: "inline should not win",
            src: "message.txt",
            attrs: { src: "message.txt" },
            index: 0,
          },
        ]),
      ],
    ]),
  };
  const srcCode = loadedCode(
    loadHook(srcState, `${srcOwner}?vue&type=message&index=0&src=true`, {
      ssr: false,
      addWatchFile(file) {
        watched.push(file);
      },
    }),
  );
  assert.equal(srcCode, "from src file\n", "a src custom block reads the external file");
  assert.deepEqual(watched, [srcFile], "the external file is a watch dependency");
} finally {
  fs.rmSync(dir, { recursive: true, force: true });
}

console.log("✅ vite-plugin-vize custom block load tests passed!");
