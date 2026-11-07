import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "../../catalog/family-catalog-types.ts";

const root = "src/components/button/" as const;

/** Catalog metadata lives with the implementation it describes. */
export const buttonFamilyCatalogEntry = {
  canonicalName: "button",
  title: "Button",
  packageSubpath: "./button",
  entryFile: `${root}index.ts`,
  sourceFiles: [
    `${root}button.vue`,
    `${root}index.ts`,
    `${root}keyboard.ts`,
    `${root}styles.css`,
    `${root}types.ts`,
  ],
  behaviorContract: `${root}button.behavior.md`,
  documentation: `${root}button.guide.md`,
  tests: [`${root}tests/behavior.test.ts`],
  typeTests: [`${root}tests/button.types.test-d.ts`],
  rendererFixture: "components/button/button.vue",
  qualityGates: componentQualityGates,
  bundleBudget: {
    exportName: "Button",
    retainedSignature: 'data-vize-ui":(?:`button`|"button"|\'button\')',
    maximumJavaScriptGzipBytes: 1_000,
    maximumCssGzipBytes: 0,
  },
  aliases: ["action-button", "button primitive"],
  upstreamCoverage: ["shadcn/ui Button", "Reka UI Primitive", "React Aria Button"],
  dependencies: ["primitive", "press"],
  maturity: "stable",
  owner: catalogOwner,
} as const satisfies UiFamilyCatalogEntry;

/** CSS-only asset emitted for consumers that opt into the visual treatment. */
export const buttonStyleAsset = {
  fileName: "component-button.css",
  source: `${root}styles.css`,
} as const;
