import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { build, type Plugin, type ResolvedConfig } from "vite";

import { generateArtModule } from "./art-module.js";
import { generatePreviewModule } from "./preview/index.js";
import {
  emitStaticGallery,
  loadStaticRuntimeModule,
  museaStaticBuildConfig,
  resolveStaticRuntimeId,
  type StaticBuildInput,
} from "./static-export.js";
import type { ArtFileInfo } from "./types/index.js";

void test("Vue 2 static preview builds without a createApp runtime export", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(process.cwd(), ".tmp-musea-vue2-static-"));
  try {
    const artPath = path.join(tempDir, "stories", "Tag.art.vue");
    const outDir = path.join(tempDir, "dist");
    await fs.promises.mkdir(path.dirname(artPath), { recursive: true });
    await fs.promises.writeFile(
      artPath,
      '<art><variant name="Default" default><span>{{ tag }}</span></variant></art>',
      "utf8",
    );

    const art = createVue2Art(artPath);
    const artFiles = new Map([[art.path, art]]);
    let resolvedConfig: ResolvedConfig | undefined;

    await build({
      configFile: false,
      root: tempDir,
      logLevel: "silent",
      build: {
        outDir,
        emptyOutDir: true,
        minify: false,
      },
      plugins: [
        createVue2StaticBuildPlugin(
          artFiles,
          () => resolvedConfig,
          (config) => {
            resolvedConfig = config;
          },
        ),
      ],
    });

    const bundledJs = await readOutputJs(outDir);
    assert.doesNotMatch(bundledJs, /\bcreateApp\b/);
    assert.match(bundledJs, /\bnew Vue\b/);
    assert.equal(await fileExists(path.join(outDir, "__musea__", "api", "static.json")), true);
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

function createVue2Art(pathname: string): ArtFileInfo {
  return {
    path: pathname,
    metadata: { title: "Tag", tags: [], status: "ready" },
    variants: [{ name: "Default", template: "<span>{{ tag }}</span>", isDefault: true }],
    hasScriptSetup: true,
    scriptSetupContent: 'const tag = "ok"',
    hasScript: false,
    styleCount: 0,
    isInline: false,
  };
}

function createVue2StaticBuildPlugin(
  artFiles: Map<string, ArtFileInfo>,
  getConfig: () => ResolvedConfig | undefined,
  setConfig: (config: ResolvedConfig) => void,
): Plugin {
  return {
    name: "musea-vue2-static-build-test",
    enforce: "pre",
    config(userConfig) {
      return museaStaticBuildConfig(userConfig.build?.rollupOptions?.input as StaticBuildInput);
    },
    configResolved(config) {
      setConfig(config);
    },
    resolveId(id) {
      if (id === "vue") return "\0musea-test-vue2";
      if (id.startsWith("virtual:musea-preview:")) {
        return "\0musea-preview-test:" + id.slice("virtual:musea-preview:".length);
      }
      if (id.startsWith("virtual:musea-art:")) {
        return "\0musea-art-test:" + id.slice("virtual:musea-art:".length);
      }
      return resolveStaticRuntimeId(id);
    },
    load(id) {
      if (id === "\0musea-test-vue2") return fakeVue2Runtime();
      if (id.startsWith("\0musea-preview-test:")) {
        const { art, variantName } = resolvePreview(id, artFiles);
        return generatePreviewModule(art, "Default", variantName, [], null, 2);
      }
      if (id.startsWith("\0musea-art-test:")) {
        const artPath = id.slice("\0musea-art-test:".length);
        return generateArtModule(artFiles.get(artPath)!, artPath);
      }
      return loadStaticRuntimeModule(id, artFiles);
    },
    async generateBundle(_options, bundle) {
      const config = getConfig();
      assert.ok(config);
      await emitStaticGallery((asset) => void this.emitFile(asset), bundle, {
        config,
        artFiles,
        scanRoots: [config.root],
        tokensPath: undefined,
        basePath: "/__musea__",
        resolvedPreviewCss: [],
        resolvedPreviewSetup: null,
        devSessionToken: "vue2-static-test",
        themeConfig: undefined,
      });
    },
  };
}

function resolvePreview(
  id: string,
  artFiles: Map<string, ArtFileInfo>,
): { art: ArtFileInfo; variantName: string } {
  const rest = id.slice("\0musea-preview-test:".length);
  const lastColonIndex = rest.lastIndexOf(":");
  assert.notEqual(lastColonIndex, -1);
  const artPath = rest.slice(0, lastColonIndex);
  const variantName = rest.slice(lastColonIndex + 1);
  return { art: artFiles.get(artPath)!, variantName };
}

function fakeVue2Runtime(): string {
  return `
export default class Vue {
  constructor(options) { this.options = options; }
  $mount() {}
  $destroy() {}
}
export const defineComponent = (options) => options;
export const reactive = (value) => value;
export const h = (...args) => ({ args });
`;
}

async function readOutputJs(dir: string): Promise<string> {
  const chunks: string[] = [];
  for (const file of await collectFiles(dir)) {
    if (file.endsWith(".js")) chunks.push(await fs.promises.readFile(file, "utf8"));
  }
  return chunks.join("\n");
}

async function collectFiles(dir: string): Promise<string[]> {
  const entries = await fs.promises.readdir(dir, { withFileTypes: true });
  const files: string[] = [];
  for (const entry of entries) {
    const filePath = path.join(dir, entry.name);
    if (entry.isDirectory()) files.push(...(await collectFiles(filePath)));
    else files.push(filePath);
  }
  return files;
}

async function fileExists(filePath: string): Promise<boolean> {
  return fs.promises.access(filePath).then(
    () => true,
    () => false,
  );
}
