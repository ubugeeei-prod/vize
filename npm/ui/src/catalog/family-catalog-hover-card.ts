import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const hoverCardFamilyRoot = "src/families/overlays/hover-card/";

export const hoverCardFamilyCatalog = [
  {
    canonicalName: "hover-card",
    title: "Hover Card",
    packageSubpath: "./hover-card",
    entryFile: `${hoverCardFamilyRoot}hover-card.ts`,
    sourceFiles: [
      `${hoverCardFamilyRoot}hover-card-content.vue`,
      `${hoverCardFamilyRoot}hover-card-context.ts`,
      `${hoverCardFamilyRoot}hover-card-root.vue`,
      `${hoverCardFamilyRoot}hover-card-trigger.vue`,
      `${hoverCardFamilyRoot}hover-card-types.ts`,
      `${hoverCardFamilyRoot}hover-card.ts`,
    ],
    behaviorContract: `${hoverCardFamilyRoot}hover-card.behavior.md`,
    tests: [
      `${hoverCardFamilyRoot}hover-card.test.ts`,
      `${hoverCardFamilyRoot}hover-card-ssr.test.ts`,
    ],
    typeTests: [`${hoverCardFamilyRoot}hover-card.types.test-d.ts`],
    rendererFixture: "HoverCardConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "HoverCardRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}hover-card-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 2_400,
      maximumCssGzipBytes: 0,
    },
    aliases: ["hover card", "preview card", "link preview", "profile card"],
    upstreamCoverage: ["Radix UI HoverCard", "Reka UI HoverCard", "Ark UI HoverCard"],
    dependencies: [
      "context",
      "controllable-state",
      "dismissable-layer",
      "hover",
      "id",
      "pointer-grace",
      "portal",
      "positioner",
      "presence",
      "primitive",
    ],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
