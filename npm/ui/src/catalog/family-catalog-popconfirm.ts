import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const popconfirmFamilyRoot = "src/families/overlays/popconfirm/";

export const popconfirmFamilyCatalog = [
  {
    canonicalName: "popconfirm",
    title: "Popconfirm",
    packageSubpath: "./popconfirm",
    entryFile: `${popconfirmFamilyRoot}popconfirm.ts`,
    sourceFiles: [
      `${popconfirmFamilyRoot}popconfirm-cancel.vue`,
      `${popconfirmFamilyRoot}popconfirm-confirm.vue`,
      `${popconfirmFamilyRoot}popconfirm-content.vue`,
      `${popconfirmFamilyRoot}popconfirm-context.ts`,
      `${popconfirmFamilyRoot}popconfirm-root.vue`,
      `${popconfirmFamilyRoot}popconfirm-trigger.vue`,
      `${popconfirmFamilyRoot}popconfirm-types.ts`,
      `${popconfirmFamilyRoot}popconfirm.ts`,
    ],
    behaviorContract: `${popconfirmFamilyRoot}popconfirm.behavior.md`,
    tests: [
      `${popconfirmFamilyRoot}popconfirm.test.ts`,
      `${popconfirmFamilyRoot}popconfirm-ssr.test.ts`,
    ],
    typeTests: [`${popconfirmFamilyRoot}popconfirm.types.test-d.ts`],
    rendererFixture: "PopconfirmConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "PopconfirmRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}popconfirm-root",
      allowedRetainedFamilies: ["context", "controllable-state", "popover"],
      maximumJavaScriptGzipBytes: 3_500,
      maximumCssGzipBytes: 0,
    },
    aliases: ["popconfirm", "inline confirm", "confirm popover", "are you sure"],
    upstreamCoverage: ["Ant Design Popconfirm", "Element Plus Popconfirm", "WAI-ARIA alertdialog"],
    dependencies: ["context", "controllable-state", "id", "popover", "positioner"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
