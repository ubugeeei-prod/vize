import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/navigation/navigation-rail/";

/** NavigationRail is the side counterpart of the bottom tab bar. */
export const navigationRailFamilyCatalog = [
  {
    canonicalName: "navigation-rail",
    title: "Navigation Rail",
    packageSubpath: "./navigation-rail",
    entryFile: `${familyRoot}navigation-rail.ts`,
    sourceFiles: [
      `${familyRoot}navigation-rail.vue`,
      `${familyRoot}navigation-rail-item.vue`,
      `${familyRoot}navigation-rail-toggle.vue`,
      `${familyRoot}navigation-rail.ts`,
      `${familyRoot}navigation-rail-context.ts`,
      `${familyRoot}navigation-rail-types.ts`,
    ],
    behaviorContract: `${familyRoot}navigation-rail.behavior.md`,
    tests: [`${familyRoot}navigation-rail.test.ts`, `${familyRoot}navigation-rail-ssr.test.ts`],
    typeTests: [`${familyRoot}navigation-rail.types.test-d.ts`],
    rendererFixture: "NavigationRailConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "NavigationRail",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}navigation-rail",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 1_900,
      maximumCssGzipBytes: 0,
    },
    aliases: ["nav rail", "side rail", "side navigation", "vertical tab bar"],
    upstreamCoverage: ["HTML nav landmark", "aria-current=page", "Material navigation rail"],
    dependencies: ["context", "controllable-state", "id"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
