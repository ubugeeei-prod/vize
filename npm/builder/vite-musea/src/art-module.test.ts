import test from "node:test";
import assert from "node:assert/strict";

import { generateArtModule, parseScriptSetupForArt } from "./art-module.ts";
import { buildVariantSfcSource } from "./art-variant-sfc.ts";
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
  assert.match(code, /export const __styles__ = \["\.logo-preview \{ color: red; \}"\];/);
  // The bindings moved into the variant's own SFC, which is what compiles the
  // template now (#3857). Its imports are rebased too: a relative specifier
  // cannot resolve from a virtual module.
  const variant = buildVariantSfcSource(art, art.variants[0].template, "default", {
    artFilePath: art.path,
  });
  assert.match(variant, /import MfMatesLogo from "\/repo\/components\/MfMatesLogo\.vue"/);
  assert.match(variant, /import \{ mfVerticalInkPresets \} from "\/repo\/components\/presets"/);
});

void test("generateArtModule preserves multiline template literal indentation in script setup", () => {
  const art: ArtFileInfo = {
    path: "/repo/components/Markdown.art.vue",
    metadata: {
      title: "Markdown",
      tags: [],
      status: "ready",
    },
    variants: [
      {
        name: "Default",
        template: `<MarkdownView :content="md" />`,
        isDefault: true,
        skipVrt: false,
      },
    ],
    hasScriptSetup: true,
    scriptSetupContent: [
      `defineArt("./MarkdownView.vue", { title: "Markdown" });`,
      "const md = `# Title",
      "",
      "- a",
      "- b`;",
    ].join("\n"),
    hasScript: false,
    styleCount: 0,
  };

  // The setup body is carried into the variant SFC verbatim, so a multiline
  // template literal must not gain the indentation of its new surroundings.
  const variant = buildVariantSfcSource(art, art.variants[0].template, "Default", {
    artFilePath: art.path,
  });

  assert.ok(variant.includes("const md = `# Title\n\n- a\n- b`;"), variant);
  assert.doesNotMatch(variant, /`# Title\n    \n    - a\n    - b`/);
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
  // The variant imports it too, which is what makes `<BaseButton />` resolve in
  // its own SFC scope.
  const variant = buildVariantSfcSource(art, `<BaseButton />`, "Default", {
    artFilePath: art.path,
    componentImportPath: "/repo/components/base-button.vue",
    componentBindingName: "BaseButton",
  });
  assert.match(variant, /import BaseButton from "\/repo\/components\/base-button\.vue"/);
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
