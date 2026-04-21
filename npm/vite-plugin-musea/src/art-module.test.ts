import test from "node:test";
import assert from "node:assert/strict";

import { generateArtModule, parseScriptSetupForArt } from "./art-module.ts";
import { generatePreviewModule } from "./preview/index.ts";
import type { ArtFileInfo } from "./types/art.ts";

void test("parseScriptSetupForArt keeps multiline imports out of setup body and returns function declarations", () => {
  const script = `
import {
  mfComponentColorTokens,
  mfPrimitiveBaseColors,
} from "./token-preview-data"
import "../generated/tokens.css"

function formatPreview() {
  return mfComponentColorTokens
}
`.trim();

  const parsed = parseScriptSetupForArt(script);

  assert.equal(parsed.imports.length, 2);
  assert.equal(
    parsed.setupBody.some((line) => line.includes("mfPrimitiveBaseColors")),
    false,
  );
  assert.deepEqual(
    parsed.returnNames.sort(),
    ["formatPreview", "mfComponentColorTokens", "mfPrimitiveBaseColors"].sort(),
  );
});

void test("generateArtModule rebases side-effect imports and emits setup for import-only script setup", () => {
  const art: ArtFileInfo = {
    path: "/repo/components/MfLogo.art.vue",
    metadata: {
      title: "Logo",
      tags: [],
      status: "ready",
    },
    variants: [
      {
        name: "default",
        template: `<MfMatesLogo :presets="mfVerticalInkPresets" />`,
        isDefault: true,
        skipVrt: false,
      },
    ],
    hasScriptSetup: true,
    scriptSetupContent: `
import MfMatesLogo from "./MfMatesLogo.vue"
import { mfVerticalInkPresets } from "./presets"
import "../generated/tokens.css"
`.trim(),
    hasScript: false,
    styleCount: 1,
    styleBlocks: [".logo-preview { color: red; }"],
  };

  const code = generateArtModule(art, art.path);

  assert.doesNotMatch(code, /import "..\/generated\/tokens\.css"/);
  assert.match(code, /import "\/repo\/generated\/tokens\.css";?/);
  assert.match(code, /return \{ MfMatesLogo, mfVerticalInkPresets \};/);
  assert.match(code, /export const __styles__ = \["\.logo-preview \{ color: red; \}"\];/);
});

void test("generatePreviewModule injects art-scoped styles from the virtual art module", () => {
  const art: ArtFileInfo = {
    path: "/repo/components/MfCard.art.vue",
    metadata: {
      title: "Card",
      tags: [],
      status: "ready",
    },
    variants: [
      {
        name: "default",
        template: "<div class='card-art-media'></div>",
        isDefault: true,
        skipVrt: false,
      },
    ],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 1,
    styleBlocks: [".card-art-media { display: block; }"],
  };

  const code = generatePreviewModule(art, "Default", "default");

  assert.match(code, /ensureArtStyles\(artModule\.__styles__\);/);
  assert.match(code, /document\.createElement\('style'\)/);
});
