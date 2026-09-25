import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const imageCompareFamilyRoot = "src/families/media/image-compare/";

export const imageCompareFamilyCatalog = [
  {
    canonicalName: "image-compare",
    title: "Image Compare",
    packageSubpath: "./image-compare",
    entryFile: `${imageCompareFamilyRoot}image-compare.ts`,
    sourceFiles: [
      `${imageCompareFamilyRoot}image-compare-after.vue`,
      `${imageCompareFamilyRoot}image-compare-before.vue`,
      `${imageCompareFamilyRoot}image-compare-context.ts`,
      `${imageCompareFamilyRoot}image-compare-handle.vue`,
      `${imageCompareFamilyRoot}image-compare-label.vue`,
      `${imageCompareFamilyRoot}image-compare-root.vue`,
      `${imageCompareFamilyRoot}image-compare-types.ts`,
      `${imageCompareFamilyRoot}image-compare-value.ts`,
      `${imageCompareFamilyRoot}image-compare.ts`,
    ],
    behaviorContract: `${imageCompareFamilyRoot}image-compare.behavior.md`,
    tests: [
      `${imageCompareFamilyRoot}image-compare.test.ts`,
      `${imageCompareFamilyRoot}image-compare-ssr.test.ts`,
      `${imageCompareFamilyRoot}image-compare-value.test.ts`,
    ],
    typeTests: [`${imageCompareFamilyRoot}image-compare.types.test-d.ts`],
    rendererFixture: "ImageCompareConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "ImageCompareRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}image-compare-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 2_700,
      maximumCssGzipBytes: 0,
    },
    aliases: ["image compare", "before after", "comparison slider", "image diff", "reveal slider"],
    upstreamCoverage: [
      "WAI-ARIA Slider",
      "img-comparison-slider",
      "react-compare-slider",
      "Ark UI Splitter-style comparisons",
    ],
    dependencies: ["context", "controllable-state", "id"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
