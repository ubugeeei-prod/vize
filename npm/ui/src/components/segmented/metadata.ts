import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "../../catalog/family-catalog-types.ts";

const root = "src/components/segmented/" as const;

/** Catalog metadata lives with the implementation it describes. */
export const segmentedFamilyCatalogEntry = {
  canonicalName: "segmented",
  title: "Segmented",
  packageSubpath: "./segmented",
  entryFile: `${root}index.ts`,
  sourceFiles: [
    `${root}segmented.vue`,
    `${root}segmented-item.vue`,
    `${root}segmented-keyboard.ts`,
    `${root}context.ts`,
    `${root}index.ts`,
    `${root}types.ts`,
  ],
  behaviorContract: `${root}segmented.behavior.md`,
  documentation: `${root}segmented.guide.md`,
  tests: [`${root}tests/behavior.test.ts`],
  typeTests: [`${root}tests/segmented.types.test-d.ts`],
  rendererFixture: "components/segmented/segmented.vue",
  qualityGates: componentQualityGates,
  bundleBudget: {
    exportName: "Segmented",
    retainedSignature: 'data-vize-ui":(?:`segmented`|"segmented"|\'segmented\')',
    maximumJavaScriptGzipBytes: 6_000,
    maximumCssGzipBytes: 0,
  },
  aliases: ["segmented control", "radio group", "exclusive toggle"],
  upstreamCoverage: ["Adobe React Aria SegmentedControl", "shadcn/ui ToggleGroup", "Reka UI"],
  dependencies: [],
  maturity: "stable",
  owner: catalogOwner,
} as const satisfies UiFamilyCatalogEntry;
