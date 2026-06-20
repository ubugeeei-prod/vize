import assert from "node:assert/strict";
import test from "node:test";

import { generatePreviewHtml } from "./html.ts";
import type { ArtFileInfo } from "../types/art.ts";

const art: ArtFileInfo = {
  path: "/repo/components/Button.art.vue",
  metadata: {
    title: "Button",
    tags: [],
    status: "ready",
  },
  variants: [
    {
      name: "Default",
      template: '<button class="px-4 py-2">Save</button>',
      isDefault: true,
      skipVrt: false,
    },
  ],
  hasScriptSetup: false,
  hasScript: false,
  styleCount: 0,
};

void test("preview reset stays in a low-priority cascade layer", () => {
  const html = generatePreviewHtml(art, art.variants[0], "/__musea__");
  const resetLayerIndex = html.indexOf("@layer musea-preview");

  assert.notEqual(resetLayerIndex, -1);
  assert.doesNotMatch(html, /\*\s*\{\s*box-sizing:\s*border-box;\s*margin:\s*0;\s*padding:\s*0;/);
  assert.match(html, /\*\s*,\s*\*::before\s*,\s*\*::after\s*\{\s*box-sizing:\s*border-box;/);
  assert.equal(/@layer musea-preview[\s\S]*\*\s*\{[^}]*padding:\s*0/.test(html), false);
  assert.equal(/@layer musea-preview[\s\S]*\*\s*\{[^}]*margin:\s*0/.test(html), false);
});
