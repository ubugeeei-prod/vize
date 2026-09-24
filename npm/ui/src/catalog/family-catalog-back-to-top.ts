import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/actions/back-to-top/";

export const backToTopFamilyCatalog = [
  {
    canonicalName: "back-to-top",
    title: "Back To Top",
    packageSubpath: "./back-to-top",
    entryFile: `${familyRoot}back-to-top.ts`,
    sourceFiles: [
      `${familyRoot}back-to-top-types.ts`,
      `${familyRoot}back-to-top.ts`,
      `${familyRoot}back-to-top.vue`,
    ],
    behaviorContract: `${familyRoot}back-to-top.behavior.md`,
    tests: [`${familyRoot}back-to-top.test.ts`, `${familyRoot}back-to-top-ssr.test.ts`],
    typeTests: [`${familyRoot}back-to-top.types.test-d.ts`],
    rendererFixture: "BackToTopConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "BackToTop",
      retainedSignature: "data-vize-ui[^-a-z]{0,8}back-to-top",
      allowedRetainedFamilies: [],
      maximumJavaScriptGzipBytes: 1_500,
      maximumCssGzipBytes: 0,
    },
    aliases: ["back to top", "scroll to top", "go to top"],
    upstreamCoverage: ["Ant Design FloatButton.BackTop", "Element Plus Backtop"],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
