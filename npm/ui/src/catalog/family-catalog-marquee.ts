import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const marqueeFamilyRoot = "src/families/media/marquee/";

export const marqueeFamilyCatalog = [
  {
    canonicalName: "marquee",
    title: "Marquee",
    packageSubpath: "./marquee",
    entryFile: `${marqueeFamilyRoot}marquee.ts`,
    sourceFiles: [
      `${marqueeFamilyRoot}marquee-content.vue`,
      `${marqueeFamilyRoot}marquee-context.ts`,
      `${marqueeFamilyRoot}marquee-geometry.ts`,
      `${marqueeFamilyRoot}marquee-pause-button.vue`,
      `${marqueeFamilyRoot}marquee-root.vue`,
      `${marqueeFamilyRoot}marquee-types.ts`,
      `${marqueeFamilyRoot}marquee.ts`,
    ],
    behaviorContract: `${marqueeFamilyRoot}marquee.behavior.md`,
    tests: [`${marqueeFamilyRoot}marquee.test.ts`, `${marqueeFamilyRoot}marquee-ssr.test.ts`],
    typeTests: [`${marqueeFamilyRoot}marquee.types.test-d.ts`],
    rendererFixture: "MarqueeConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "MarqueeRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}marquee-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 3_200,
      maximumCssGzipBytes: 0,
    },
    aliases: ["marquee", "ticker", "news ticker", "logo wall", "infinite scroller"],
    upstreamCoverage: ["WAI-ARIA marquee role", "WCAG 2.2.2 Pause, Stop, Hide", "Magic UI Marquee"],
    dependencies: ["context", "controllable-state", "id", "measure"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
