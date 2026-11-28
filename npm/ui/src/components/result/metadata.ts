import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "../../catalog/family-catalog-types.ts";

const root = "src/components/result/" as const;

/** Catalog metadata lives with the implementation it describes. */
export const resultFamilyCatalogEntry = {
  canonicalName: "result",
  title: "Result",
  packageSubpath: "./result",
  entryFile: `${root}index.ts`,
  sourceFiles: [`${root}result.vue`, `${root}index.ts`, `${root}types.ts`],
  behaviorContract: `${root}result.behavior.md`,
  documentation: `${root}result.guide.md`,
  tests: [`${root}tests/behavior.test.ts`],
  typeTests: [`${root}tests/result.types.test-d.ts`],
  rendererFixture: "components/result/result.vue",
  qualityGates: componentQualityGates,
  bundleBudget: {
    exportName: "Result",
    retainedSignature: 'data-vize-ui":(?:`result`|"result"|\'result\')',
    maximumJavaScriptGzipBytes: 4_000,
    maximumCssGzipBytes: 0,
  },
  aliases: ["result page", "empty state", "error page", "404"],
  upstreamCoverage: ["Ant Design Result", "shadcn/ui", "Adobe Spectrum IllustratedMessage"],
  dependencies: [],
  maturity: "stable",
  owner: catalogOwner,
} as const satisfies UiFamilyCatalogEntry;
