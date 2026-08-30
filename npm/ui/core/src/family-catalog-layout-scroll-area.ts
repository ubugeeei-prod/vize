import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/layout/scroll-area/";

export const scrollAreaLayoutFamilyCatalog = [
  {
    canonicalName: "scroll-area",
    title: "Scroll Area",
    packageSubpath: "./scroll-area",
    entryFile: `${familyRoot}scroll-area.ts`,
    sourceFiles: [
      `${familyRoot}scroll-area.vue`,
      `${familyRoot}scroll-area.ts`,
      `${familyRoot}scroll-area.css`,
      `${familyRoot}scroll-area-runtime.ts`,
      `${familyRoot}scroll-area-types.ts`,
    ],
    behaviorContract: `${familyRoot}scroll-area.behavior.md`,
    tests: [
      `${familyRoot}scroll-area.test.ts`,
      `${familyRoot}scroll-area-state.test.ts`,
      `${familyRoot}scroll-area-ssr.test.ts`,
    ],
    typeTests: [`${familyRoot}scroll-area.types.test-d.ts`],
    rendererFixture: "ScrollAreaConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "ScrollArea",
      retainedSignature: 'data-vize-ui":(?:`scroll-area`|"scroll-area"|\'scroll-area\')',
      maximumJavaScriptGzipBytes: 2_800,
      maximumCssGzipBytes: 7_600,
    },
    aliases: ["scroll area", "scroll viewport", "native scroll container", "overflow region"],
    upstreamCoverage: [
      "CSS overflow",
      "CSS overscroll-behavior",
      "CSS scrollbar-gutter",
      "CSS scrollbar-width",
      "WAI-ARIA named region",
      "Radix UI ScrollArea",
      "Reka UI ScrollArea",
    ],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
