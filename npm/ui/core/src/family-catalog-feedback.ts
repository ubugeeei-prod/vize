import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

export const feedbackFamilyCatalog = [
  {
    canonicalName: "alert",
    title: "Alert",
    packageSubpath: "./alert",
    entryFile: "src/alert.ts",
    sourceFiles: ["src/alert.vue", "src/alert.ts", "src/alert-types.ts"],
    behaviorContract: "src/alert.behavior.md",
    tests: ["src/alert.test.ts"],
    typeTests: ["src/alert.types.test-d.ts"],
    rendererFixture: "alert.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Alert",
      retainedSignature: "data-vize-ui[\\s\\S]{0,16}alert",
      maximumJavaScriptGzipBytes: 750,
      maximumCssGzipBytes: 0,
    },
    aliases: ["inline alert", "status alert", "callout primitive"],
    upstreamCoverage: ["WAI-ARIA alert role", "WAI-ARIA status role", "shadcn/ui Alert"],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
