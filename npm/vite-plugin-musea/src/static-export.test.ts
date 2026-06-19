import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import vm from "node:vm";
import type { ResolvedConfig } from "vite";

import { joinUrlPath, staticPreviewId } from "./static-data.js";
import {
  emitStaticGallery,
  loadStaticRuntimeModule,
  VIRTUAL_STATIC_RUNTIME,
} from "./static-export.js";
import type { ArtFileInfo } from "./types/index.js";

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

void test("static gallery paths stay under the configured Musea base path", () => {
  const previewId = staticPreviewId("/repo/src/Button.art.vue", "Default");

  assert.equal(previewId.length, 20);
  assert.equal(
    joinUrlPath("/__musea__", "preview", `${previewId}.html`),
    `/__musea__/preview/${previewId}.html`,
  );
});

void test("static runtime keeps discovered preview modules reachable", () => {
  const art = createArt("/repo/src/Button.art.vue");
  const code = loadStaticRuntimeModule("\0musea-static-runtime", new Map([[art.path, art]]));

  assert.ok(code);
  assert.match(code, /loadMuseaPreview/);
  assert.match(code, /virtual:musea-preview:\/repo\/src\/Button\.art\.vue:Default/);
  assert.equal(loadStaticRuntimeModule(VIRTUAL_STATIC_RUNTIME, new Map()), null);
});

void test("emitStaticGallery packages browser-facing static output", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-static-output-"));
  try {
    const artPath = path.join(tempDir, "src", "Button.art.vue");
    await fs.promises.mkdir(path.dirname(artPath), { recursive: true });
    await fs.promises.writeFile(artPath, "<art></art>", "utf8");
    const art = createArt(artPath);
    const assets = new Map<string, string>();

    await emitStaticGallery(
      (asset) => {
        assets.set(
          asset.fileName,
          typeof asset.source === "string"
            ? asset.source
            : Buffer.from(asset.source).toString("utf8"),
        );
      },
      {
        "assets/musea-static-runtime.js": {
          type: "chunk",
          name: "musea-static-runtime",
          fileName: "assets/musea-static-runtime.js",
          facadeModuleId: null,
        },
      },
      {
        config: { root: tempDir } as ResolvedConfig,
        artFiles: new Map([[art.path, art]]),
        scanRoots: [tempDir],
        tokensPath: undefined,
        basePath: "/__musea__",
        resolvedPreviewCss: [],
        resolvedPreviewSetup: null,
        devSessionToken: "static-test",
        themeConfig: undefined,
      },
    );

    const previewId = staticPreviewId(art.path, "Default");
    const staticPayload = JSON.parse(assetText(assets, "__musea__/api/static.json")) as {
      arts: Array<{ path: string; metadata: { title: string }; variants: Array<{ name: string }> }>;
      previews: Record<string, Record<string, string>>;
    };
    const artsPayload = JSON.parse(assetText(assets, "__musea__/api/arts")) as Array<{
      path: string;
      metadata: { title: string };
      variants: Array<{ name: string }>;
    }>;

    assert.deepEqual(staticPayload.arts.map(compactArtPayload), [
      { path: art.path, title: "Button", variants: ["Default"] },
    ]);
    assert.deepEqual(artsPayload.map(compactArtPayload), staticPayload.arts.map(compactArtPayload));
    assert.deepEqual(staticPayload.previews, {
      [art.path]: {
        Default: `/__musea__/preview/${previewId}.html`,
      },
    });

    const requiredAssets = new Set([
      "__musea__/index.html",
      "__musea__/api/static.json",
      "__musea__/api/arts",
      `__musea__/preview/${previewId}.html`,
      "index.html",
    ]);
    assert.deepEqual([...assets.keys()].filter((fileName) => requiredAssets.has(fileName)).sort(), [
      "__musea__/api/arts",
      "__musea__/api/static.json",
      "__musea__/index.html",
      `__musea__/preview/${previewId}.html`,
      "index.html",
    ]);

    const globals = executeStaticGlobals(assetText(assets, "__musea__/index.html"));
    assert.deepEqual(globals, {
      __MUSEA_BASE_PATH__: "/__musea__",
      __MUSEA_STATIC__: true,
      __MUSEA_STATIC_PREVIEWS__: {
        [art.path]: {
          Default: `/__musea__/preview/${previewId}.html`,
        },
      },
    });

    assert.equal(
      previewRuntimeSpecifier(assetText(assets, `__musea__/preview/${previewId}.html`)),
      "../../assets/musea-static-runtime.js",
    );
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("static gallery runtime reports static-mode mutation and missing detail errors", async () => {
  const originalWindow = Reflect.get(globalThis, "window");
  const originalFetch = globalThis.fetch;
  Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: {
      __MUSEA_BASE_PATH__: "/__musea__",
      __MUSEA_STATIC__: true,
      __MUSEA_STATIC_PREVIEWS__: {},
    },
  });
  globalThis.fetch = async () =>
    new Response(
      JSON.stringify({
        arts: [],
        previews: {},
        details: {},
        tokens: undefined,
        tokenUsage: undefined,
      }),
      { status: 200, headers: { "content-type": "application/json" } },
    );

  try {
    const moduleUrl = new URL(`../gallery/staticApi.ts?static-mode-${Date.now()}`, import.meta.url)
      .href;
    const { fetchStaticDetail, staticMutationError } = (await import(moduleUrl)) as {
      fetchStaticDetail: (artPath: string) => Promise<unknown>;
      staticMutationError: () => Error;
    };

    assert.equal(
      staticMutationError().message,
      "This action is not available in a static Musea gallery.",
    );
    await assert.rejects(
      () => fetchStaticDetail("/missing/Button.art.vue"),
      new Error("Art not found: /missing/Button.art.vue"),
    );
  } finally {
    if (originalWindow === undefined) {
      Reflect.deleteProperty(globalThis, "window");
    } else {
      Object.defineProperty(globalThis, "window", {
        configurable: true,
        value: originalWindow,
      });
    }
    globalThis.fetch = originalFetch;
  }
});

function compactArtPayload(art: {
  path: string;
  metadata: { title: string };
  variants: Array<{ name: string }>;
}) {
  return {
    path: art.path,
    title: art.metadata.title,
    variants: art.variants.map((variant) => variant.name),
  };
}

function assetText(assets: Map<string, string>, fileName: string): string {
  const value = assets.get(fileName);
  assert.notEqual(value, undefined);
  return value as string;
}

function executeStaticGlobals(indexHtml: string): Record<string, unknown> {
  const script = plainScripts(indexHtml)[0];
  assert.notEqual(script, undefined);
  const context = { window: {} as Record<string, unknown> };
  vm.runInNewContext(script as string, context);
  return JSON.parse(JSON.stringify(context.window)) as Record<string, unknown>;
}

function plainScripts(html: string): string[] {
  return [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].map((match) => match[1] ?? "");
}

function previewRuntimeSpecifier(html: string): string {
  const imports = [...html.matchAll(/import\s+\{\s*loadMuseaPreview\s*\}\s+from\s+("[^"]+")/g)];
  assert.equal(imports.length, 1);
  return JSON.parse(imports[0]![1]!) as string;
}
