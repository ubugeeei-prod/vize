import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/layout/icon/";

export const iconLayoutFamilyCatalog = [
  {
    canonicalName: "icon",
    title: "Icon",
    packageSubpath: "./icon",
    entryFile: `${familyRoot}icon.ts`,
    sourceFiles: [`${familyRoot}icon.vue`, `${familyRoot}icon.ts`, `${familyRoot}icon-types.ts`],
    behaviorContract: `${familyRoot}icon.behavior.md`,
    tests: [`${familyRoot}icon.test.ts`, `${familyRoot}icon-ssr.test.ts`],
    typeTests: [`${familyRoot}icon.types.test-d.ts`],
    rendererFixture: "IconConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Icon",
      retainedSignature: 'data-vize-ui":(?:`icon`|"icon"|\'icon\')',
      maximumJavaScriptGzipBytes: 1_900,
      maximumCssGzipBytes: 0,
    },
    aliases: ["svg icon", "glyph", "pictogram", "icon composer"],
    upstreamCoverage: ["SVG title element", "SVG desc element", "React Aria SVG semantics"],
    dependencies: ["id"],
    maturity: "stable",
    owner: catalogOwner,
  },
  {
    canonicalName: "icon-button",
    title: "IconButton",
    packageSubpath: "./icon-button",
    entryFile: `${familyRoot}icon-button.ts`,
    sourceFiles: [
      `${familyRoot}icon-button.vue`,
      `${familyRoot}icon-button.ts`,
      `${familyRoot}icon-types.ts`,
    ],
    behaviorContract: `${familyRoot}icon-button.behavior.md`,
    tests: [`${familyRoot}icon-button.test.ts`, `${familyRoot}icon-button-ssr.test.ts`],
    typeTests: [`${familyRoot}icon.types.test-d.ts`],
    rendererFixture: "IconButtonConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "IconButton",
      retainedSignature: 'data-vize-ui":(?:`icon-button`|"icon-button"|\'icon-button\')',
      maximumJavaScriptGzipBytes: 2_200,
      maximumCssGzipBytes: 0,
    },
    aliases: ["icon button", "toolbar icon", "icon-only action", "glyph button"],
    upstreamCoverage: ["HTML button", "WAI-ARIA Button Pattern", "React Aria Button"],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
