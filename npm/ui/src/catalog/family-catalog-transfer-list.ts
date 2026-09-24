import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const transferListFamilyRoot = "src/families/selection/transfer-list/";

export const transferListFamilyCatalog = [
  {
    canonicalName: "transfer-list",
    title: "Transfer List",
    packageSubpath: "./transfer-list",
    entryFile: `${transferListFamilyRoot}transfer-list.ts`,
    sourceFiles: [
      `${transferListFamilyRoot}transfer-list-action.vue`,
      `${transferListFamilyRoot}transfer-list-context.ts`,
      `${transferListFamilyRoot}transfer-list-empty.vue`,
      `${transferListFamilyRoot}transfer-list-item.vue`,
      `${transferListFamilyRoot}transfer-list-model.ts`,
      `${transferListFamilyRoot}transfer-list-panel.vue`,
      `${transferListFamilyRoot}transfer-list-root.vue`,
      `${transferListFamilyRoot}transfer-list-search.vue`,
      `${transferListFamilyRoot}transfer-list-types.ts`,
      `${transferListFamilyRoot}transfer-list.ts`,
    ],
    behaviorContract: `${transferListFamilyRoot}transfer-list.behavior.md`,
    tests: [
      `${transferListFamilyRoot}transfer-list.test.ts`,
      `${transferListFamilyRoot}transfer-list-ssr.test.ts`,
    ],
    typeTests: [`${transferListFamilyRoot}transfer-list.types.test-d.ts`],
    rendererFixture: "families/selection/transfer-list/transfer-list-root.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "TransferListRoot",
      retainedSignature: 'data-vize-ui":(?:`transfer-list`|"transfer-list"|\'transfer-list\')',
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 4_000,
      maximumCssGzipBytes: 0,
    },
    aliases: ["transfer list", "dual listbox", "shuttle", "list builder"],
    upstreamCoverage: ["Ant Design Transfer", "PrimeVue PickList", "MUI Transfer List"],
    dependencies: [
      "collection",
      "composite-navigation",
      "context",
      "controllable-state",
      "id",
      "typeahead",
    ],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
