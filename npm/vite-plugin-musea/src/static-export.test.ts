import assert from "node:assert/strict";
import test from "node:test";

import { joinUrlPath, staticPreviewId } from "./static-data.js";
import { loadStaticRuntimeModule, VIRTUAL_STATIC_RUNTIME } from "./static-export.js";
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
