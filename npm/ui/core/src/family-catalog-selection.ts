import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

export const selectionFamilyCatalog = [
  {
    canonicalName: "listbox",
    title: "Listbox",
    packageSubpath: "./listbox",
    entryFile: "src/listbox.ts",
    sourceFiles: [
      "src/listbox.vue",
      "src/listbox-item.vue",
      "src/listbox.ts",
      "src/listbox-context.ts",
      "src/listbox-types.ts",
      "src/listbox-value.ts",
    ],
    behaviorContract: "src/listbox.behavior.md",
    tests: ["src/listbox.test.ts", "src/listbox-ssr.test.ts"],
    typeTests: ["src/listbox.types.test-d.ts"],
    rendererFixture: "ListboxConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Listbox",
      retainedSignature: 'data-vize-ui":(?:`listbox`|"listbox"|\'listbox\')',
      allowedRetainedFamilies: [
        "collection",
        "composite-navigation",
        "context",
        "controllable-state",
        "typeahead",
      ],
      maximumJavaScriptGzipBytes: 9_100,
      maximumCssGzipBytes: 0,
    },
    aliases: ["listbox", "option list", "single select", "multi select"],
    upstreamCoverage: [
      "WAI-ARIA listbox pattern",
      "React Aria ListBox",
      "Ariakit Select",
      "Reka UI Listbox",
    ],
    dependencies: [
      "collection",
      "composite-navigation",
      "context",
      "controllable-state",
      "id",
      "typeahead",
    ],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
