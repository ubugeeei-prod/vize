import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import vm from "node:vm";
import type { ResolvedConfig } from "vite";

import { joinUrlPath, staticPreviewId } from "./static-data.js";
import { emitGalleryShell } from "./static-gallery-shell.js";
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

function assetText(assets: Map<string, string>, fileName: string): string {
  const value = assets.get(fileName);
  assert.notEqual(value, undefined);
  return value as string;
}

function executeStaticGlobals(indexHtml: string): Record<string, unknown> {
  const script = [...indexHtml.matchAll(/<script>([\s\S]*?)<\/script>/g)].map(
    (match) => match[1] ?? "",
  )[0];
  assert.notEqual(script, undefined);
  const context = { window: {} as Record<string, unknown> };
  vm.runInNewContext(script as string, context);
  return JSON.parse(JSON.stringify(context.window)) as Record<string, unknown>;
}

// Regression test for #3109: with a prebuilt gallery shell and a basePath that
// carries a subpath, the base rewrite must run before the globals are injected —
// rewriting afterwards expanded the already prefixed preview URLs a second time
// (`/site/__musea__/preview/…` → `/site/site/__musea__/preview/…`).
void test("emitGalleryShell keeps subpath base paths single-prefixed in the prebuilt shell", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-gallery-shell-"));
  try {
    const galleryDistDir = path.join(tempDir, "gallery");
    await fs.promises.mkdir(galleryDistDir, { recursive: true });
    await fs.promises.writeFile(
      path.join(galleryDistDir, "index.html"),
      '<html><head><link rel="stylesheet" href="/__musea__/assets/app.css"></head><body></body></html>',
      "utf8",
    );
    await fs.promises.writeFile(
      path.join(galleryDistDir, "app.js"),
      'fetch("/__musea__/api/arts");',
      "utf8",
    );

    const art = createArt("/repo/src/Button.art.vue");
    const previewId = staticPreviewId(art.path, "Default");
    const basePath = "/site/__musea__";
    const ctx = {
      config: { root: tempDir } as ResolvedConfig,
      artFiles: new Map([[art.path, art]]),
      scanRoots: [tempDir],
      tokensPath: undefined,
      basePath,
      resolvedPreviewCss: [],
      resolvedPreviewSetup: null,
      devSessionToken: "static-test",
      themeConfig: undefined,
      tokenPreviewConfig: undefined,
    };
    const payload = {
      arts: [],
      previews: { [art.path]: { Default: joinUrlPath(basePath, "preview", `${previewId}.html`) } },
    };

    const assets = new Map<string, string>();
    await emitGalleryShell(
      (asset) => {
        assets.set(
          asset.fileName,
          typeof asset.source === "string"
            ? asset.source
            : Buffer.from(asset.source).toString("utf8"),
        );
      },
      "site/__musea__",
      ctx,
      payload,
      galleryDistDir,
    );

    const indexHtml = assetText(assets, "site/__musea__/index.html");
    assert.equal(indexHtml.includes("/site/site/"), false);
    assert.equal(indexHtml.includes('href="/site/__musea__/assets/app.css"'), true);
    assert.deepEqual(executeStaticGlobals(indexHtml), {
      __MUSEA_BASE_PATH__: basePath,
      __MUSEA_STATIC__: true,
      __MUSEA_STATIC_PREVIEWS__: {
        [art.path]: {
          Default: `/site/__musea__/preview/${previewId}.html`,
        },
      },
    });
    assert.equal(assetText(assets, "site/__musea__/app.js"), 'fetch("/site/__musea__/api/arts");');
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});
