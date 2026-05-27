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

void test("generateArtModule treats defineArt as a compiler macro and isolates setup by variant", () => {
  const art: ArtFileInfo = {
    path: "/repo/components/Button.art.vue",
    metadata: {
      title: "Button",
      component: "./Button.vue",
      tags: [],
      status: "ready",
    },
    variants: [
      { name: "Primary", template: `<Button :count="count" />`, isDefault: true, skipVrt: false },
      {
        name: "Secondary",
        template: `<Button :count="count" />`,
        isDefault: false,
        skipVrt: false,
      },
    ],
    hasScriptSetup: true,
    scriptSetupContent: `
import { ref } from "vue"

defineArt("./Button.vue", {
  title: "Button",
})

const count = ref(0)
`.trim(),
    scriptSetupIsolated: true,
    hasScript: false,
    styleCount: 0,
  };

  const code = generateArtModule(art, art.path);

  assert.match(code, /import Button from "\/repo\/components\/Button\.vue";/);
  assert.doesNotMatch(code, /\bdefineArt\s*\(/);
  assert.match(code, /components: \{ "Button": Button \}/);
  assert.match(code, /export const Primary = defineComponent\(\{[\s\S]*const count = ref\(0\)/);
  assert.match(code, /export const Secondary = defineComponent\(\{[\s\S]*const count = ref\(0\)/);
  assert.doesNotMatch(code, /return \{ Button,/);
});

void test("parseScriptSetupForArt infers defineArt component source literals", () => {
  const parsed = parseScriptSetupForArt(
    `
import { ref } from "vue"

defineArt("./base-button.vue", { title: "Base Button" });

const count = ref(0)
`.trim(),
  );

  assert.equal(parsed.defineArtComponentName, "BaseButton");
  assert.equal(parsed.defineArtComponentSource, "./base-button.vue");
  assert.deepEqual(parsed.returnNames.sort(), ["count", "ref"].sort());
});

void test("generateArtModule can resolve component only from defineArt source", () => {
  const art: ArtFileInfo = {
    path: "/repo/components/BaseButton.art.vue",
    metadata: {
      title: "Base Button",
      tags: [],
      status: "ready",
    },
    variants: [
      {
        name: "Default",
        template: `<BaseButton />`,
        isDefault: true,
        skipVrt: false,
      },
    ],
    hasScriptSetup: true,
    scriptSetupContent: `defineArt("./base-button.vue", { title: "Base Button" });`,
    hasScript: false,
    styleCount: 0,
  };

  const code = generateArtModule(art, art.path);

  assert.match(code, /import BaseButton from "\/repo\/components\/base-button\.vue";/);
  assert.match(code, /components: \{ "BaseButton": BaseButton \}/);
});

void test("generateArtModule shares setup when script setup isolate is false", () => {
  const art: ArtFileInfo = {
    path: "/repo/components/Button.art.vue",
    metadata: {
      title: "Button",
      component: "./Button.vue",
      tags: [],
      status: "ready",
    },
    variants: [
      { name: "Primary", template: `<Button :count="count" />`, isDefault: true, skipVrt: false },
      {
        name: "Secondary",
        template: `<Button :count="count" />`,
        isDefault: false,
        skipVrt: false,
      },
    ],
    hasScriptSetup: true,
    scriptSetupContent: `
import { ref } from "vue"
import Button from "./Button.vue"
const count = ref(0)
`.trim(),
    scriptSetupIsolated: false,
    hasScript: false,
    styleCount: 0,
  };

  const code = generateArtModule(art, art.path);

  assert.match(code, /const __museaSharedSetup = \(\(\) => \{/);
  assert.match(code, /return __museaSharedSetup;/);
  assert.equal((code.match(/const count = ref\(0\)/g) ?? []).length, 1);
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

void test("generated modules quote dynamic specifiers", () => {
  const art: ArtFileInfo = {
    path: `/repo/components/MfCard';sideEffect().art.vue`,
    metadata: {
      title: "Card",
      tags: [],
      status: "ready",
      component: `./MfCard';sideEffect().vue`,
    },
    variants: [
      {
        name: `default';sideEffect()`,
        template: "<Self />",
        isDefault: true,
        skipVrt: false,
      },
    ],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };

  const artCode = generateArtModule(art, art.path);
  const previewCode = generatePreviewModule(art, "Default", art.variants[0].name, [
    `/repo/theme';sideEffect().css`,
  ]);

  assert.match(
    artCode,
    /import MfCardSideEffect from "\/repo\/components\/MfCard';sideEffect\(\)\.vue";/,
  );
  assert.match(previewCode, /import "\/repo\/theme';sideEffect\(\)\.css";/);
  assert.match(
    previewCode,
    /import \* as artModule from "virtual:musea-art:\/repo\/components\/MfCard';sideEffect\(\)\.art\.vue";/,
  );
});
