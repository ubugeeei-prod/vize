import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "../../catalog/family-catalog-types.ts";

const root = "src/components/diff-view/" as const;

/** Catalog metadata lives with the implementation it describes. */
export const diffViewFamilyCatalogEntry = {
  canonicalName: "diff-view",
  title: "Diff View",
  packageSubpath: "./diff-view",
  entryFile: `${root}index.ts`,
  sourceFiles: [
    `${root}diff-view.vue`,
    `${root}diff-parse.ts`,
    `${root}index.ts`,
    `${root}types.ts`,
  ],
  behaviorContract: `${root}diff-view.behavior.md`,
  documentation: `${root}diff-view.guide.md`,
  tests: [`${root}tests/behavior.test.ts`],
  typeTests: [`${root}tests/diff-view.types.test-d.ts`],
  rendererFixture: "components/diff-view/diff-view.vue",
  qualityGates: componentQualityGates,
  bundleBudget: {
    exportName: "DiffView",
    retainedSignature: 'data-vize-ui":(?:`diff-view`|"diff-view"|\'diff-view\')',
    maximumJavaScriptGzipBytes: 4_000,
    maximumCssGzipBytes: 0,
  },
  aliases: ["diff", "patch view", "unified diff", "code diff"],
  upstreamCoverage: ["shadcn/ui", "GitHub diff", "Ant Design"],
  dependencies: [],
  maturity: "stable",
  owner: catalogOwner,
} as const satisfies UiFamilyCatalogEntry;
