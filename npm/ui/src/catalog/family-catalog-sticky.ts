import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/layout/sticky/";

export const stickyFamilyCatalog = [
  {
    canonicalName: "sticky",
    title: "Sticky",
    packageSubpath: "./sticky",
    entryFile: `${familyRoot}sticky.ts`,
    sourceFiles: [
      `${familyRoot}sticky-geometry.ts`,
      `${familyRoot}sticky-types.ts`,
      `${familyRoot}sticky.ts`,
      `${familyRoot}sticky.vue`,
    ],
    behaviorContract: `${familyRoot}sticky.behavior.md`,
    tests: [`${familyRoot}sticky.test.ts`, `${familyRoot}sticky-ssr.test.ts`],
    typeTests: [`${familyRoot}sticky.types.test-d.ts`],
    rendererFixture: "StickyConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "Sticky",
      retainedSignature: "data-vize-ui[^-a-z]{0,8}sticky(?![-a-z])",
      allowedRetainedFamilies: ["measure"],
      maximumJavaScriptGzipBytes: 2_000,
      maximumCssGzipBytes: 0,
    },
    aliases: ["sticky", "affix", "sticky header", "pinned"],
    upstreamCoverage: [
      "CSS position: sticky",
      "Ant Design Affix",
      "IntersectionObserver stuck detection",
    ],
    dependencies: ["measure"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
