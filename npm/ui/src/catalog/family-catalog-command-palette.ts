import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const commandPaletteFamilyRoot = "src/families/overlays/command-palette/";

export const commandPaletteFamilyCatalog = [
  {
    canonicalName: "command-palette",
    title: "Command Palette",
    packageSubpath: "./command-palette",
    entryFile: `${commandPaletteFamilyRoot}command-palette.ts`,
    sourceFiles: [
      `${commandPaletteFamilyRoot}command-palette-context.ts`,
      `${commandPaletteFamilyRoot}command-palette-dialog.vue`,
      `${commandPaletteFamilyRoot}command-palette-empty.vue`,
      `${commandPaletteFamilyRoot}command-palette-filter.ts`,
      `${commandPaletteFamilyRoot}command-palette-group.vue`,
      `${commandPaletteFamilyRoot}command-palette-input.vue`,
      `${commandPaletteFamilyRoot}command-palette-item.vue`,
      `${commandPaletteFamilyRoot}command-palette-list.vue`,
      `${commandPaletteFamilyRoot}command-palette-loading.vue`,
      `${commandPaletteFamilyRoot}command-palette-root.vue`,
      `${commandPaletteFamilyRoot}command-palette-types.ts`,
      `${commandPaletteFamilyRoot}command-palette.ts`,
    ],
    behaviorContract: `${commandPaletteFamilyRoot}command-palette.behavior.md`,
    tests: [
      `${commandPaletteFamilyRoot}command-palette.test.ts`,
      `${commandPaletteFamilyRoot}command-palette-ssr.test.ts`,
    ],
    typeTests: [`${commandPaletteFamilyRoot}command-palette.types.test-d.ts`],
    rendererFixture: "CommandPaletteConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "CommandPaletteRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}command-palette-root",
      allowedRetainedFamilies: [
        "collection",
        "composite-navigation",
        "context",
        "controllable-state",
        "live-region",
        "typeahead",
      ],
      maximumJavaScriptGzipBytes: 9_800,
      maximumCssGzipBytes: 0,
    },
    aliases: ["command palette", "command menu", "cmdk", "spotlight", "quick open"],
    upstreamCoverage: [
      "WAI-ARIA Combobox with listbox popup",
      "cmdk",
      "Ark UI Combobox",
      "VS Code Command Palette",
    ],
    dependencies: [
      "collection",
      "command",
      "composite-navigation",
      "context",
      "controllable-state",
      "dialog",
      "id",
      "live-region",
      "shortcut",
    ],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
