import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const infiniteScrollFamilyRoot = "src/families/data/infinite-scroll/";

export const infiniteScrollFamilyCatalog = [
  {
    canonicalName: "infinite-scroll",
    title: "Infinite Scroll",
    packageSubpath: "./infinite-scroll",
    entryFile: `${infiniteScrollFamilyRoot}infinite-scroll.ts`,
    sourceFiles: [
      `${infiniteScrollFamilyRoot}infinite-scroll-context.ts`,
      `${infiniteScrollFamilyRoot}infinite-scroll-item.vue`,
      `${infiniteScrollFamilyRoot}infinite-scroll-load-more.vue`,
      `${infiniteScrollFamilyRoot}infinite-scroll-root.vue`,
      `${infiniteScrollFamilyRoot}infinite-scroll-sentinel.vue`,
      `${infiniteScrollFamilyRoot}infinite-scroll-status.vue`,
      `${infiniteScrollFamilyRoot}infinite-scroll-types.ts`,
      `${infiniteScrollFamilyRoot}infinite-scroll.ts`,
    ],
    behaviorContract: `${infiniteScrollFamilyRoot}infinite-scroll.behavior.md`,
    tests: [
      `${infiniteScrollFamilyRoot}infinite-scroll.test.ts`,
      `${infiniteScrollFamilyRoot}infinite-scroll-ssr.test.ts`,
    ],
    typeTests: [`${infiniteScrollFamilyRoot}infinite-scroll.types.test-d.ts`],
    rendererFixture: "InfiniteScrollConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "InfiniteScrollRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}infinite-scroll-root",
      allowedRetainedFamilies: ["context"],
      maximumJavaScriptGzipBytes: 2_350,
      maximumCssGzipBytes: 0,
    },
    aliases: ["infinite scroll", "load more", "feed", "endless scroll", "pagination sentinel"],
    upstreamCoverage: [
      "WAI-ARIA Feed",
      "IntersectionObserver",
      "VueUse useInfiniteScroll",
      "React Aria useLoadMore",
    ],
    dependencies: ["context", "id", "measure"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
