import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/layout/list/";

export const listLayoutFamilyCatalog = [
  {
    canonicalName: "list",
    title: "List",
    packageSubpath: "./list",
    entryFile: `${familyRoot}list.ts`,
    sourceFiles: [`${familyRoot}list.vue`, `${familyRoot}list.ts`, `${familyRoot}list-types.ts`],
    behaviorContract: `${familyRoot}list.behavior.md`,
    tests: [`${familyRoot}list.test.ts`, `${familyRoot}list-ssr.test.ts`],
    typeTests: [`${familyRoot}list.types.test-d.ts`],
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
