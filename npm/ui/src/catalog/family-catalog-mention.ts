import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const mentionFamilyRoot = "src/families/form/mention/";

export const mentionFamilyCatalog = [
  {
    canonicalName: "mention",
    title: "Mention",
    packageSubpath: "./mention",
    entryFile: `${mentionFamilyRoot}mention.ts`,
    sourceFiles: [
      `${mentionFamilyRoot}mention-caret.ts`,
      `${mentionFamilyRoot}mention-content.vue`,
      `${mentionFamilyRoot}mention-context.ts`,
      `${mentionFamilyRoot}mention-core.ts`,
      `${mentionFamilyRoot}mention-editable.vue`,
      `${mentionFamilyRoot}mention-empty.vue`,
      `${mentionFamilyRoot}mention-input.vue`,
      `${mentionFamilyRoot}mention-item.vue`,
      `${mentionFamilyRoot}mention-root.vue`,
      `${mentionFamilyRoot}mention-types.ts`,
      `${mentionFamilyRoot}mention.ts`,
    ],
    behaviorContract: `${mentionFamilyRoot}mention.behavior.md`,
    tests: [
      `${mentionFamilyRoot}mention.test.ts`,
      `${mentionFamilyRoot}mention-core.test.ts`,
      `${mentionFamilyRoot}mention-ssr.test.ts`,
    ],
    typeTests: [`${mentionFamilyRoot}mention.types.test-d.ts`],
    rendererFixture: "families/form/mention/mention-root.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "MentionRoot",
      retainedSignature: 'data-vize-ui":(?:`mention`|"mention"|\'mention\')',
      allowedRetainedFamilies: [
        "collection",
        "composite-navigation",
        "context",
        "controllable-state",
        "typeahead",
      ],
      maximumJavaScriptGzipBytes: 11_400,
      maximumCssGzipBytes: 0,
    },
    aliases: ["mention", "mentions", "at mention", "trigger autocomplete", "hashtag input"],
    upstreamCoverage: [
      "WAI-ARIA combobox pattern (listbox popup)",
      "Ant Design Mentions",
      "Mantine Mentions",
      "Tiptap Suggestion",
    ],
    dependencies: [
      "collection",
      "composite-navigation",
      "context",
      "controllable-state",
      "dismissable-layer",
      "id",
      "portal",
      "positioner",
      "presence",
    ],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
