import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "../../catalog/family-catalog-types.ts";

const root = "src/components/input-group/" as const;

/** Catalog metadata lives with the implementation it describes. */
export const inputGroupFamilyCatalogEntry = {
  canonicalName: "input-group",
  title: "Input Group",
  packageSubpath: "./input-group",
  entryFile: `${root}index.ts`,
  sourceFiles: [
    `${root}input-group.vue`,
    `${root}input-group-addon.vue`,
    `${root}input-group-input.vue`,
    `${root}input-group-textarea.vue`,
    `${root}input-group-button.vue`,
    `${root}context.ts`,
    `${root}index.ts`,
    `${root}types.ts`,
  ],
  behaviorContract: `${root}input-group.behavior.md`,
  documentation: `${root}input-group.guide.md`,
  tests: [`${root}tests/behavior.test.ts`],
  typeTests: [`${root}tests/input-group.types.test-d.ts`],
  rendererFixture: "components/input-group/input-group.vue",
  qualityGates: componentQualityGates,
  bundleBudget: {
    exportName: "InputGroup",
    retainedSignature: 'data-vize-ui":(?:`input-group`|"input-group"|\'input-group\')',
    maximumJavaScriptGzipBytes: 6_000,
    maximumCssGzipBytes: 0,
  },
  aliases: ["input addon", "affix input", "grouped field"],
  upstreamCoverage: ["shadcn/ui InputGroup", "Ant Design Input.Group", "Reka UI"],
  dependencies: [],
  maturity: "stable",
  owner: catalogOwner,
} as const satisfies UiFamilyCatalogEntry;
