import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

export const listLayoutFamilyCatalog = [
  {
    canonicalName: "list",
    title: "List",
    packageSubpath: "./list",
    entryFile: "src/list.ts",
    sourceFiles: ["src/list.vue", "src/list.ts", "src/list-types.ts"],
    behaviorContract: "src/list.behavior.md",
    tests: ["src/list.test.ts", "src/list-ssr.test.ts"],
    typeTests: ["src/list.types.test-d.ts"],
    rendererFixture: "ListConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "List",
      retainedSignature: 'data-vize-ui":(?:`list`|"list"|\'list\')',
      maximumJavaScriptGzipBytes: 900,
      maximumCssGzipBytes: 0,
    },
    aliases: ["ordered list", "unordered list", "list group", "content list"],
    upstreamCoverage: ["HTML ul element", "HTML ol element", "Reka UI Primitive"],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
