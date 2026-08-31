import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const sliderFamilyRoot = "src/families/form/slider/";

export const sliderFamilyCatalog = [
  {
    canonicalName: "slider",
    title: "Slider",
    packageSubpath: "./slider",
    entryFile: `${sliderFamilyRoot}slider.ts`,
    sourceFiles: [
      `${sliderFamilyRoot}slider.vue`,
      `${sliderFamilyRoot}slider.ts`,
      `${sliderFamilyRoot}slider-state.ts`,
      `${sliderFamilyRoot}slider-types.ts`,
    ],
    behaviorContract: `${sliderFamilyRoot}slider.behavior.md`,
    tests: [`${sliderFamilyRoot}slider.test.ts`, `${sliderFamilyRoot}slider-ssr.test.ts`],
    typeTests: [`${sliderFamilyRoot}slider.types.test-d.ts`],
    rendererFixture: "families/form/slider/slider.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Slider",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}slider",
      allowedRetainedFamilies: ["controllable-state"],
      maximumJavaScriptGzipBytes: 3_600,
      maximumCssGzipBytes: 0,
    },
    aliases: ["range input", "single-thumb slider", "volume slider"],
    upstreamCoverage: ["HTML range input", "WAI-ARIA Slider", "React Aria Slider"],
    dependencies: ["controllable-state", "id"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
