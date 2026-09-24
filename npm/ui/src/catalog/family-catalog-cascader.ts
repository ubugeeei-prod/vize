import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const cascaderFamilyRoot = "src/families/selection/cascader/";

export const cascaderFamilyCatalog = [
  {
    canonicalName: "cascader",
    title: "Cascader",
    packageSubpath: "./cascader",
    entryFile: `${cascaderFamilyRoot}cascader.ts`,
    sourceFiles: [
      `${cascaderFamilyRoot}cascader-column.vue`,
      `${cascaderFamilyRoot}cascader-content.vue`,
      `${cascaderFamilyRoot}cascader-context.ts`,
      `${cascaderFamilyRoot}cascader-item.vue`,
      `${cascaderFamilyRoot}cascader-model.ts`,
      `${cascaderFamilyRoot}cascader-root.vue`,
      `${cascaderFamilyRoot}cascader-trigger.vue`,
      `${cascaderFamilyRoot}cascader-types.ts`,
      `${cascaderFamilyRoot}cascader-value.vue`,
      `${cascaderFamilyRoot}cascader.ts`,
    ],
    behaviorContract: `${cascaderFamilyRoot}cascader.behavior.md`,
    tests: [`${cascaderFamilyRoot}cascader.test.ts`, `${cascaderFamilyRoot}cascader-ssr.test.ts`],
    typeTests: [`${cascaderFamilyRoot}cascader.types.test-d.ts`],
    rendererFixture: "families/selection/cascader/cascader-root.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "CascaderRoot",
      retainedSignature: 'data-vize-ui":(?:`cascader`|"cascader"|\'cascader\')',
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 5_400,
      maximumCssGzipBytes: 0,
    },
    aliases: ["cascader", "cascade select", "multi-level select", "hierarchical picker"],
    upstreamCoverage: ["Ant Design Cascader", "Element Plus Cascader", "PrimeVue CascadeSelect"],
    dependencies: [
      "context",
      "controllable-state",
      "dismissable-layer",
      "id",
      "portal",
      "positioner",
      "presence",
    ],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
