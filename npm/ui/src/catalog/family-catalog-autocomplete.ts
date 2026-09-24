import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const autocompleteFamilyRoot = "src/families/selection/autocomplete/";

export const autocompleteFamilyCatalog = [
  {
    canonicalName: "autocomplete",
    title: "Autocomplete",
    packageSubpath: "./autocomplete",
    entryFile: `${autocompleteFamilyRoot}autocomplete.ts`,
    sourceFiles: [
      `${autocompleteFamilyRoot}autocomplete-history.ts`,
      `${autocompleteFamilyRoot}autocomplete-root.vue`,
      `${autocompleteFamilyRoot}autocomplete-types.ts`,
      `${autocompleteFamilyRoot}autocomplete.ts`,
    ],
    behaviorContract: `${autocompleteFamilyRoot}autocomplete.behavior.md`,
    tests: [
      `${autocompleteFamilyRoot}autocomplete.test.ts`,
      `${autocompleteFamilyRoot}autocomplete-ssr.test.ts`,
    ],
    typeTests: [`${autocompleteFamilyRoot}autocomplete.types.test-d.ts`],
    rendererFixture: "families/selection/autocomplete/autocomplete-root.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "AutocompleteRoot",
      retainedSignature: "data-vize-ui-preset[\\s\\S]{0,8}autocomplete",
      allowedRetainedFamilies: [
        "collection",
        "combobox",
        "composite-navigation",
        "context",
        "controllable-state",
        "typeahead",
      ],
      maximumJavaScriptGzipBytes: 14_500,
      // Combobox's accessible async status carries its structural stylesheet.
      maximumCssGzipBytes: 7_800,
    },
    aliases: ["autocomplete", "address autocomplete", "search suggestions", "recent searches"],
    upstreamCoverage: [
      "WAI-ARIA combobox pattern (list autocomplete)",
      "Algolia Autocomplete recent searches",
      "Google Places Autocomplete",
    ],
    dependencies: ["combobox", "controllable-state", "select"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
