import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/accessibility/landmark/";

export const landmarkFamilyCatalog = [
  {
    canonicalName: "landmark",
    title: "Landmark",
    packageSubpath: "./landmark",
    entryFile: `${familyRoot}landmark.ts`,
    sourceFiles: [
      `${familyRoot}landmark-provider.vue`,
      `${familyRoot}landmark-runtime.ts`,
      `${familyRoot}landmark-types.ts`,
      `${familyRoot}landmark.ts`,
      `${familyRoot}landmark.vue`,
    ],
    behaviorContract: `${familyRoot}landmark.behavior.md`,
    tests: [`${familyRoot}landmark.test.ts`, `${familyRoot}landmark-ssr.test.ts`],
    typeTests: [`${familyRoot}landmark.types.test-d.ts`],
    rendererFixture: "LandmarkConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Landmark",
      retainedSignature: "data-vize-ui[^-a-z]{0,8}landmark",
      allowedRetainedFamilies: ["context"],
      maximumJavaScriptGzipBytes: 2_500,
      maximumCssGzipBytes: 0,
    },
    aliases: ["landmark", "landmark region", "F6 navigation", "skip regions"],
    upstreamCoverage: [
      "WAI-ARIA landmark roles",
      "HTML-AAM landmark mappings",
      "HTML search element",
      "F6 pane cycling",
    ],
    dependencies: ["collection", "context", "id"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
