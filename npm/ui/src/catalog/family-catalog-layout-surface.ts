import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/layout/surface/";

export const surfaceLayoutFamilyCatalog = [
  {
    canonicalName: "surface",
    title: "Surface",
    packageSubpath: "./surface",
    entryFile: `${familyRoot}surface.ts`,
    sourceFiles: [
      `${familyRoot}surface.vue`,
      `${familyRoot}surface.ts`,
      `${familyRoot}surface-runtime.ts`,
      `${familyRoot}surface-types.ts`,
    ],
    behaviorContract: `${familyRoot}surface.behavior.md`,
    tests: [`${familyRoot}surface.test.ts`, `${familyRoot}surface-ssr.test.ts`],
    typeTests: [`${familyRoot}surface.types.test-d.ts`],
    rendererFixture: "SurfaceConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Surface",
      retainedSignature: 'data-vize-ui":(?:`surface`|"surface"|\'surface\')',
      maximumJavaScriptGzipBytes: 1_200,
      maximumCssGzipBytes: 0,
    },
    aliases: ["surface", "semantic surface", "section wrapper", "panel wrapper"],
    upstreamCoverage: [
      "HTML section element",
      "HTML article element",
      "HTML aside element",
      "ARIA labelledby and describedby IDREFs",
      "shadcn/ui Card",
      "Reka UI Primitive",
    ],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
