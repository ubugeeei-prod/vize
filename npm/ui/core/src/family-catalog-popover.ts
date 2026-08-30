import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const popoverFamilyRoot = "src/families/overlays/popover/";

export const popoverFamilyCatalog = [
  {
    canonicalName: "popover",
    title: "Popover",
    packageSubpath: "./popover",
    entryFile: `${popoverFamilyRoot}popover.ts`,
    sourceFiles: [
      `${popoverFamilyRoot}popover-arrow.vue`,
      `${popoverFamilyRoot}popover-content.vue`,
      `${popoverFamilyRoot}popover-content-runtime.ts`,
      `${popoverFamilyRoot}popover-context.ts`,
      `${popoverFamilyRoot}popover-root.vue`,
      `${popoverFamilyRoot}popover-trigger.vue`,
      `${popoverFamilyRoot}popover.ts`,
      `${popoverFamilyRoot}popover-types.ts`,
    ],
    behaviorContract: `${popoverFamilyRoot}popover.behavior.md`,
    tests: [`${popoverFamilyRoot}popover.test.ts`, `${popoverFamilyRoot}popover-ssr.test.ts`],
    typeTests: [`${popoverFamilyRoot}popover.types.test-d.ts`],
    rendererFixture: "PopoverConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "PopoverRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}popover-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 3_000,
      maximumCssGzipBytes: 0,
    },
    aliases: ["floating dialog", "disclosure layer", "coach mark shell", "anchored popup"],
    upstreamCoverage: [
      "WAI-ARIA dialog pattern",
      "HTML Popover API authoring model",
      "Radix Popover",
      "Reka UI Popover",
    ],
    dependencies: [
      "context",
      "controllable-state",
      "dismissable-layer",
      "focus-guards",
      "focus-scope",
      "id",
      "inert-outside",
      "portal",
      "positioner",
      "presence",
      "scroll-lock",
    ],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
