import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const tourFamilyRoot = "src/families/overlays/tour/";

export const tourFamilyCatalog = [
  {
    canonicalName: "tour",
    title: "Tour",
    packageSubpath: "./tour",
    entryFile: `${tourFamilyRoot}tour.ts`,
    sourceFiles: [
      `${tourFamilyRoot}tour-arrow.vue`,
      `${tourFamilyRoot}tour-close.vue`,
      `${tourFamilyRoot}tour-content.vue`,
      `${tourFamilyRoot}tour-context.ts`,
      `${tourFamilyRoot}tour-description.vue`,
      `${tourFamilyRoot}tour-next.vue`,
      `${tourFamilyRoot}tour-prev.vue`,
      `${tourFamilyRoot}tour-progress.vue`,
      `${tourFamilyRoot}tour-root.vue`,
      `${tourFamilyRoot}tour-spotlight.vue`,
      `${tourFamilyRoot}tour-state.ts`,
      `${tourFamilyRoot}tour-step.vue`,
      `${tourFamilyRoot}tour-title.vue`,
      `${tourFamilyRoot}tour-types.ts`,
      `${tourFamilyRoot}tour.ts`,
    ],
    behaviorContract: `${tourFamilyRoot}tour.behavior.md`,
    tests: [
      `${tourFamilyRoot}tour.test.ts`,
      `${tourFamilyRoot}tour-ssr.test.ts`,
      `${tourFamilyRoot}tour-state.test.ts`,
    ],
    typeTests: [`${tourFamilyRoot}tour.types.test-d.ts`],
    rendererFixture: "TourConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "TourRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}tour-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 3_900,
      maximumCssGzipBytes: 0,
    },
    aliases: ["tour", "product tour", "onboarding", "walkthrough", "coachmark", "guided tour"],
    upstreamCoverage: [
      "WAI-ARIA Dialog (non-modal)",
      "Ark UI Tour",
      "Shepherd.js",
      "Driver.js",
      "Reactour",
    ],
    dependencies: [
      "context",
      "controllable-state",
      "dismissable-layer",
      "focus-scope",
      "id",
      "portal",
      "positioner",
      "presence",
    ],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
