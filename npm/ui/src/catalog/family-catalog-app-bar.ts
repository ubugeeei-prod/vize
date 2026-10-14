import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/navigation/app-bar/";

/** AppBar is the top application bar that reports scroll elevation and hide-on-scroll state. */
export const appBarFamilyCatalog = [
  {
    canonicalName: "app-bar",
    title: "App Bar",
    packageSubpath: "./app-bar",
    entryFile: `${familyRoot}app-bar.ts`,
    sourceFiles: [
      `${familyRoot}app-bar.vue`,
      `${familyRoot}app-bar.ts`,
      `${familyRoot}app-bar-types.ts`,
    ],
    behaviorContract: `${familyRoot}app-bar.behavior.md`,
    tests: [`${familyRoot}app-bar.test.ts`, `${familyRoot}app-bar-ssr.test.ts`],
    typeTests: [`${familyRoot}app-bar.types.test-d.ts`],
    rendererFixture: "AppBarConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "AppBar",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}app-bar",
      maximumJavaScriptGzipBytes: 1_600,
      maximumCssGzipBytes: 0,
    },
    aliases: ["top app bar", "app header", "navigation bar", "header bar"],
    upstreamCoverage: ["HTML header landmark", "Material top app bar", "Vuetify v-app-bar"],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
