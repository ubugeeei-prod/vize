import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";
import { build, type Plugin, type ResolvedConfig } from "vite";

import { staticPreviewId } from "./static-data.js";
import {
  emitStaticGallery,
  museaStaticBuildInput,
  loadStaticRuntimeModule,
  resolveStaticRuntimeId,
  VIRTUAL_STATIC_RUNTIME,
  type StaticBuildInput,
} from "./static-export.js";
import type { ArtFileInfo } from "./types/index.js";

void test("static build input keeps user entries and names the preview runtime", () => {
  assert.deepEqual(museaStaticBuildInput("/repo/musea/index.html"), {
    "musea-static-entry": "/repo/musea/index.html",
    "musea-static-runtime": VIRTUAL_STATIC_RUNTIME,
  });
  assert.deepEqual(museaStaticBuildInput(["/repo/a.html", "/repo/b.html"]), {
    "musea-static-entry-1": "/repo/a.html",
    "musea-static-entry-2": "/repo/b.html",
    "musea-static-runtime": VIRTUAL_STATIC_RUNTIME,
  });
  assert.deepEqual(museaStaticBuildInput({ app: "/repo/index.html" }), {
    app: "/repo/index.html",
    "musea-static-runtime": VIRTUAL_STATIC_RUNTIME,
  });
});

void test("static build emits previews with a callable side-effect runtime", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(process.cwd(), ".tmp-musea-static-build-"));
  const previousLoadedPreviews = Reflect.get(globalThis, "__museaLoadedPreviews");
  try {
    const htmlEntry = path.join(tempDir, "musea", "index.html");
    const entryScript = path.join(tempDir, "src", "main.js");
    const artPath = path.join(tempDir, "stories", "Button.art.vue");
    const outDir = path.join(tempDir, "dist", "musea");
    await fs.promises.mkdir(path.dirname(htmlEntry), { recursive: true });
    await fs.promises.mkdir(path.dirname(entryScript), { recursive: true });
    await fs.promises.mkdir(path.dirname(artPath), { recursive: true });
    await fs.promises.writeFile(
      htmlEntry,
      '<div id="app"></div><script type="module" src="/src/main.js"></script>',
      "utf8",
    );
    await fs.promises.writeFile(entryScript, 'globalThis.__museaUserEntry = "built";', "utf8");
    await fs.promises.writeFile(
      artPath,
      '<art><variant name="Default" default><button>OK</button></variant></art>',
      "utf8",
    );

    const art = createArt(artPath);
    const artFiles = new Map([[art.path, art]]);
    let resolvedConfig: ResolvedConfig | undefined;

    await build({
      configFile: false,
      root: tempDir,
      logLevel: "silent",
      build: {
        outDir,
        emptyOutDir: true,
        rollupOptions: { input: htmlEntry },
      },
      plugins: [
        createStaticBuildTestPlugin(
          artFiles,
          () => resolvedConfig,
          (config) => {
            resolvedConfig = config;
          },
        ),
      ],
    });

    const previewId = staticPreviewId(art.path, "Default");
    const previewPath = path.join(outDir, "__musea__", "preview", `${previewId}.html`);
    const previewHtml = await fs.promises.readFile(previewPath, "utf8");
    const runtimePath = path.resolve(
      path.dirname(previewPath),
      previewRuntimeSpecifier(previewHtml),
    );
    const previousWindow = Reflect.get(globalThis, "window");
    const runtimeWindow: Record<string, unknown> = {};
    Object.defineProperty(globalThis, "window", {
      configurable: true,
      value: runtimeWindow,
    });

    try {
      await import(pathToFileURL(runtimePath).href);
      const loadPreview = runtimeWindow.__MUSEA_LOAD_PREVIEW__;
      assert.equal(typeof loadPreview, "function");
      await (loadPreview as (id: string) => Promise<void>)(previewId);
      assert.deepEqual(Reflect.get(globalThis, "__museaLoadedPreviews"), [`${art.path}:Default`]);
    } finally {
      if (previousWindow === undefined) {
        Reflect.deleteProperty(globalThis, "window");
      } else {
        Object.defineProperty(globalThis, "window", {
          configurable: true,
          value: previousWindow,
        });
      }
    }

    assert.equal(await fileExists(path.join(outDir, "musea", "index.html")), true);
    assert.equal(await fileExists(path.join(outDir, "__musea__", "api", "static.json")), true);
    assert.match(await fs.promises.readFile(runtimePath, "utf8"), /__MUSEA_LOAD_PREVIEW__/);
  } finally {
    if (previousLoadedPreviews === undefined) {
      Reflect.deleteProperty(globalThis, "__museaLoadedPreviews");
    } else {
      Reflect.set(globalThis, "__museaLoadedPreviews", previousLoadedPreviews);
    }
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

function createArt(pathname: string): ArtFileInfo {
  return {
    path: pathname,
    metadata: { title: "Button", tags: [], status: "ready" },
    variants: [{ name: "Default", template: "<Button />", isDefault: true }],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
    isInline: false,
  };
}

function previewRuntimeSpecifier(html: string): string {
  assert.doesNotMatch(html, /import\s+\{\s*loadMuseaPreview\s*\}\s+from/);
  const imports = [...html.matchAll(/import\s+("[^"]+");/g)];
  assert.equal(imports.length, 1);
  return JSON.parse(imports[0]![1]!) as string;
}

function createStaticBuildTestPlugin(
  artFiles: Map<string, ArtFileInfo>,
  getConfig: () => ResolvedConfig | undefined,
  setConfig: (config: ResolvedConfig) => void,
): Plugin {
  return {
    name: "musea-static-build-test",
    options(options) {
      options.input = museaStaticBuildInput(options.input as StaticBuildInput);
      return null;
    },
    configResolved(config) {
      setConfig(config);
    },
    resolveId(id) {
      if (id.startsWith("virtual:musea-preview:")) {
        return "\0musea-preview-test:" + id.slice("virtual:musea-preview:".length);
      }
      return resolveStaticRuntimeId(id);
    },
    load(id) {
      if (id.startsWith("\0musea-preview-test:")) {
        const previewKey = id.slice("\0musea-preview-test:".length);
        return `
const previous = globalThis.__museaLoadedPreviews || [];
globalThis.__museaLoadedPreviews = [...previous, ${JSON.stringify(previewKey)}];
`;
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
        devSessionToken: "static-build-test",
        themeConfig: undefined,
      });
    },
  };
}

async function fileExists(filePath: string): Promise<boolean> {
  return fs.promises.access(filePath).then(
    () => true,
    () => false,
  );
}
