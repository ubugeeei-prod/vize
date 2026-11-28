import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "../../catalog/family-catalog-types.ts";

const root = "src/components/descriptions/" as const;

/** Catalog metadata lives with the implementation it describes. */
export const descriptionsFamilyCatalogEntry = {
  canonicalName: "descriptions",
  title: "Descriptions",
  packageSubpath: "./descriptions",
  entryFile: `${root}index.ts`,
  sourceFiles: [
    `${root}descriptions.vue`,
    `${root}descriptions-item.vue`,
    `${root}index.ts`,
    `${root}types.ts`,
  ],
  behaviorContract: `${root}descriptions.behavior.md`,
  documentation: `${root}descriptions.guide.md`,
  tests: [`${root}tests/behavior.test.ts`],
  typeTests: [`${root}tests/descriptions.types.test-d.ts`],
  rendererFixture: "components/descriptions/descriptions.vue",
  qualityGates: componentQualityGates,
  bundleBudget: {
    exportName: "Descriptions",
    retainedSignature: 'data-vize-ui":(?:`descriptions`|"descriptions"|\'descriptions\')',
    maximumJavaScriptGzipBytes: 4_000,
    maximumCssGzipBytes: 0,
  },
  aliases: ["description list", "definition list", "property list", "detail list"],
  upstreamCoverage: ["Ant Design Descriptions", "HTML dl", "shadcn/ui"],
  dependencies: [],
  maturity: "stable",
  owner: catalogOwner,
} as const satisfies UiFamilyCatalogEntry;
