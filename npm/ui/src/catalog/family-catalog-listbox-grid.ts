import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const listboxGridFamilyRoot = "src/families/selection/listbox-grid/";

export const listboxGridFamilyCatalog = [
  {
    canonicalName: "listbox-grid",
    title: "Listbox Grid",
    packageSubpath: "./listbox-grid",
    entryFile: `${listboxGridFamilyRoot}listbox-grid.ts`,
    sourceFiles: [
      `${listboxGridFamilyRoot}listbox-grid-context.ts`,
      `${listboxGridFamilyRoot}listbox-grid-item.vue`,
      `${listboxGridFamilyRoot}listbox-grid-model.ts`,
      `${listboxGridFamilyRoot}listbox-grid-types.ts`,
      `${listboxGridFamilyRoot}listbox-grid.ts`,
      `${listboxGridFamilyRoot}listbox-grid.vue`,
    ],
    behaviorContract: `${listboxGridFamilyRoot}listbox-grid.behavior.md`,
    tests: [
      `${listboxGridFamilyRoot}listbox-grid.test.ts`,
      `${listboxGridFamilyRoot}listbox-grid-ssr.test.ts`,
    ],
    typeTests: [`${listboxGridFamilyRoot}listbox-grid.types.test-d.ts`],
    rendererFixture: "families/selection/listbox-grid/listbox-grid.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "ListboxGrid",
      retainedSignature: 'data-vize-ui":(?:`listbox-grid`|"listbox-grid"|\'listbox-grid\')',
      allowedRetainedFamilies: ["collection", "context", "controllable-state", "typeahead"],
      maximumJavaScriptGzipBytes: 8_000,
      maximumCssGzipBytes: 0,
    },
    aliases: ["listbox grid", "icon picker", "color swatch picker", "rich select grid"],
    upstreamCoverage: [
      "WAI-ARIA listbox pattern (grid layout)",
      "React Aria ListBox grid layout",
      "Ark UI Color Picker swatches",
    ],
    dependencies: ["collection", "context", "controllable-state", "id", "typeahead"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
