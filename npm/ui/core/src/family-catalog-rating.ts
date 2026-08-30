import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const ratingFamilyRoot = "src/families/form/rating/";

export const ratingFamilyCatalog = [
  {
    canonicalName: "rating",
    title: "Rating",
    packageSubpath: "./rating",
    entryFile: `${ratingFamilyRoot}rating.ts`,
    sourceFiles: [
      `${ratingFamilyRoot}rating.vue`,
      `${ratingFamilyRoot}rating.ts`,
      `${ratingFamilyRoot}rating-runtime.ts`,
      `${ratingFamilyRoot}rating-state.ts`,
      `${ratingFamilyRoot}rating-types.ts`,
    ],
    behaviorContract: `${ratingFamilyRoot}rating.behavior.md`,
    tests: [
      `${ratingFamilyRoot}rating.test.ts`,
      `${ratingFamilyRoot}rating-ssr.test.ts`,
      `${ratingFamilyRoot}rating-state.test.ts`,
    ],
    typeTests: [`${ratingFamilyRoot}rating.types.test-d.ts`],
    rendererFixture: "families/form/rating/rating.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Rating",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}rating",
      allowedRetainedFamilies: ["controllable-state"],
      maximumJavaScriptGzipBytes: 4_100,
      maximumCssGzipBytes: 0,
    },
    aliases: ["rating", "star rating", "score picker"],
    upstreamCoverage: [
      "HTML radio",
      "WAI-ARIA Radio Group",
      "React Aria RadioGroup",
      "Radix ToggleGroup rating examples",
    ],
    dependencies: ["controllable-state", "id"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
