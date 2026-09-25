import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/actions/floating-action-button/";

export const floatingActionButtonFamilyCatalog = [
  {
    canonicalName: "floating-action-button",
    title: "Floating Action Button",
    packageSubpath: "./floating-action-button",
    entryFile: `${familyRoot}floating-action-button.ts`,
    sourceFiles: [
      `${familyRoot}floating-action-button-types.ts`,
      `${familyRoot}floating-action-button.ts`,
      `${familyRoot}floating-action-button.vue`,
      `${familyRoot}speed-dial-action.vue`,
      `${familyRoot}speed-dial-content.vue`,
      `${familyRoot}speed-dial-context.ts`,
      `${familyRoot}speed-dial-root.vue`,
      `${familyRoot}speed-dial-trigger.vue`,
    ],
    behaviorContract: `${familyRoot}floating-action-button.behavior.md`,
    tests: [
      `${familyRoot}floating-action-button.test.ts`,
      `${familyRoot}floating-action-button-ssr.test.ts`,
    ],
    typeTests: [`${familyRoot}floating-action-button.types.test-d.ts`],
    rendererFixture: "FloatingActionButtonConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "FloatingActionButton",
      retainedSignature: "data-vize-ui[^-a-z]{0,8}floating-action-button",
      allowedRetainedFamilies: ["context"],
      maximumJavaScriptGzipBytes: 1_200,
      maximumCssGzipBytes: 0,
    },
    aliases: ["fab", "floating action button", "speed dial", "float button"],
    upstreamCoverage: [
      "Material FAB",
      "Material Speed Dial",
      "WAI-ARIA Menu Button",
      "Ant Design FloatButton",
    ],
    dependencies: [
      "collection",
      "composite-navigation",
      "context",
      "controllable-state",
      "dismissable-layer",
      "id",
    ],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
