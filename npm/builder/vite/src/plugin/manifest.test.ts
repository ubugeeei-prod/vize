import assert from "node:assert/strict";
import { test } from "node:test";

import { restoreManifestSourceIds, restoreSsrManifestSourceIds } from "./manifest.ts";

void test("manifest entries are keyed by SFC source path, as with plugin-vue", () => {
  // Void looks a route's CSS up as `pages/index.vue`; the query hid it.
  const manifest = {
    "virtual:void-pages-client": {
      src: "virtual:void-pages-client",
      dynamicImports: ["pages/index.vue?vue&vize"],
    },
    "pages/index.vue?vue&vize": {
      src: "pages/index.vue?vue&vize",
      imports: ["_runtime.js"],
    },
    "pages/index.vue?vue&type=style&index=0&lang.css": { src: "x" },
  };

  assert.equal(restoreManifestSourceIds(manifest), true);
  assert.deepEqual(Object.keys(manifest).sort(), [
    "pages/index.vue",
    "pages/index.vue?vue&type=style&index=0&lang.css",
    "virtual:void-pages-client",
  ]);
  assert.equal(
    (manifest as Record<string, { src?: string }>)["pages/index.vue"].src,
    "pages/index.vue",
  );
  assert.deepEqual(manifest["virtual:void-pages-client"].dynamicImports, ["pages/index.vue"]);
});

void test("an existing source key is never overwritten", () => {
  const manifest = {
    "App.vue": { src: "App.vue" },
    "App.vue?vue&vize": { src: "App.vue?vue&vize" },
  };
  assert.equal(restoreManifestSourceIds(manifest), false);
  assert.deepEqual(Object.keys(manifest), ["App.vue", "App.vue?vue&vize"]);
});

void test("ssr manifest module IDs drop the main SFC query", () => {
  const manifest: Record<string, string[]> = {
    "pages/index.vue?vue&vize": ["/assets/index.js"],
    "src/main.ts": ["/assets/main.js"],
  };
  assert.equal(restoreSsrManifestSourceIds(manifest), true);
  assert.deepEqual(manifest, {
    "pages/index.vue": ["/assets/index.js"],
    "src/main.ts": ["/assets/main.js"],
  });
});
