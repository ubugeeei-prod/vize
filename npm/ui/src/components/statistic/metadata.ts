import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "../../catalog/family-catalog-types.ts";

const root = "src/components/statistic/" as const;

/** Catalog metadata lives with the implementation it describes. */
export const statisticFamilyCatalogEntry = {
  canonicalName: "statistic",
  title: "Statistic",
  packageSubpath: "./statistic",
  entryFile: `${root}index.ts`,
  sourceFiles: [
    `${root}statistic.vue`,
    `${root}statistic-format.ts`,
    `${root}index.ts`,
    `${root}types.ts`,
  ],
  behaviorContract: `${root}statistic.behavior.md`,
  documentation: `${root}statistic.guide.md`,
  tests: [`${root}tests/behavior.test.ts`],
  typeTests: [`${root}tests/statistic.types.test-d.ts`],
  rendererFixture: "components/statistic/statistic.vue",
  qualityGates: componentQualityGates,
  bundleBudget: {
    exportName: "Statistic",
    retainedSignature: 'data-vize-ui":(?:`statistic`|"statistic"|\'statistic\')',
    maximumJavaScriptGzipBytes: 4_000,
    maximumCssGzipBytes: 0,
  },
  aliases: ["stat", "metric", "kpi", "number display"],
  upstreamCoverage: ["Ant Design Statistic", "shadcn/ui", "Adobe Spectrum"],
  dependencies: [],
  maturity: "stable",
  owner: catalogOwner,
} as const satisfies UiFamilyCatalogEntry;
