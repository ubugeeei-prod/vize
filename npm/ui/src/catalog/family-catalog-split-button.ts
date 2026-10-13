import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/actions/split-button/";

/** SplitButton pairs a primary action with a menu button on the shared menu core. */
export const splitButtonFamilyCatalog = [
  {
    canonicalName: "split-button",
    title: "Split Button",
    packageSubpath: "./split-button",
    entryFile: `${familyRoot}split-button.ts`,
    sourceFiles: [
      `${familyRoot}split-button.ts`,
      `${familyRoot}split-button-action.vue`,
      `${familyRoot}split-button-context.ts`,
      `${familyRoot}split-button-root.vue`,
      `${familyRoot}split-button-trigger.vue`,
      `${familyRoot}split-button-types.ts`,
    ],
    behaviorContract: `${familyRoot}split-button.behavior.md`,
    tests: [`${familyRoot}split-button.test.ts`, `${familyRoot}split-button-ssr.test.ts`],
    typeTests: [`${familyRoot}split-button.types.test-d.ts`],
    rendererFixture: "SplitButtonConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "SplitButtonTrigger",
      retainedSignature: "data-vize-ui[\"'`:=\\s]{1,4}split-button-trigger",
      allowedRetainedFamilies: ["context"],
      maximumJavaScriptGzipBytes: 1_500,
      maximumCssGzipBytes: 0,
    },
    aliases: ["split button", "menu button with default action", "dropdown button", "combo button"],
    upstreamCoverage: [
      "WAI-ARIA menu button pattern",
      "PrimeVue SplitButton",
      "Fluent UI SplitButton",
      "Carbon Combo Button",
    ],
    dependencies: [
      "collection",
      "context",
      "controllable-state",
      "direction",
      "dismissable-layer",
      "focus-scope",
      "id",
      "inert-outside",
      "menu",
      "pointer-grace",
      "portal",
      "positioner",
      "presence",
      "scroll-lock",
      "typeahead",
    ],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
