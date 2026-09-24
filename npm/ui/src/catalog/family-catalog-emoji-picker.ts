import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const emojiPickerFamilyRoot = "src/families/selection/emoji-picker/";

export const emojiPickerFamilyCatalog = [
  {
    canonicalName: "emoji-picker",
    title: "Emoji Picker",
    packageSubpath: "./emoji-picker",
    entryFile: `${emojiPickerFamilyRoot}emoji-picker.ts`,
    sourceFiles: [
      `${emojiPickerFamilyRoot}emoji-picker-category.vue`,
      `${emojiPickerFamilyRoot}emoji-picker-context.ts`,
      `${emojiPickerFamilyRoot}emoji-picker-empty.vue`,
      `${emojiPickerFamilyRoot}emoji-picker-grid.vue`,
      `${emojiPickerFamilyRoot}emoji-picker-item.vue`,
      `${emojiPickerFamilyRoot}emoji-picker-model.ts`,
      `${emojiPickerFamilyRoot}emoji-picker-preview.vue`,
      `${emojiPickerFamilyRoot}emoji-picker-root.vue`,
      `${emojiPickerFamilyRoot}emoji-picker-search.vue`,
      `${emojiPickerFamilyRoot}emoji-picker-skin-tone.vue`,
      `${emojiPickerFamilyRoot}emoji-picker-types.ts`,
      `${emojiPickerFamilyRoot}emoji-picker.ts`,
    ],
    behaviorContract: `${emojiPickerFamilyRoot}emoji-picker.behavior.md`,
    tests: [
      `${emojiPickerFamilyRoot}emoji-picker.test.ts`,
      `${emojiPickerFamilyRoot}emoji-picker-ssr.test.ts`,
    ],
    typeTests: [`${emojiPickerFamilyRoot}emoji-picker.types.test-d.ts`],
    rendererFixture: "families/selection/emoji-picker/emoji-picker-root.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "EmojiPickerRoot",
      retainedSignature: 'data-vize-ui":(?:`emoji-picker`|"emoji-picker"|\'emoji-picker\')',
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 4_000,
      maximumCssGzipBytes: 0,
    },
    aliases: ["emoji picker", "emoji selector", "reaction picker"],
    upstreamCoverage: ["WAI-ARIA grid pattern", "emoji-mart", "Frimousse", "Ark UI"],
    dependencies: ["context", "controllable-state", "id", "listbox-grid"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
