import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/accessibility/media-preferences/";

export const mediaPreferencesFamilyCatalog = [
  {
    canonicalName: "media-preferences",
    title: "Media Preferences",
    packageSubpath: "./media-preferences",
    entryFile: `${familyRoot}media-preferences.ts`,
    sourceFiles: [
      `${familyRoot}media-preferences-context.ts`,
      `${familyRoot}media-preferences-provider.vue`,
      `${familyRoot}media-preferences-runtime.ts`,
      `${familyRoot}media-preferences-types.ts`,
      `${familyRoot}media-preferences.ts`,
    ],
    behaviorContract: `${familyRoot}media-preferences.behavior.md`,
    tests: [`${familyRoot}media-preferences.test.ts`, `${familyRoot}media-preferences-ssr.test.ts`],
    typeTests: [`${familyRoot}media-preferences.types.test-d.ts`],
    rendererFixture: "MediaPreferencesConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "MediaPreferencesProvider",
      retainedSignature: "data-vize-ui[^-a-z]{0,8}media-preferences-provider",
      allowedRetainedFamilies: ["context"],
      maximumJavaScriptGzipBytes: 2_000,
      maximumCssGzipBytes: 0,
    },
    aliases: [
      "reduced motion",
      "reduced transparency",
      "forced colors",
      "high contrast",
      "prefers-contrast",
      "color scheme",
    ],
    upstreamCoverage: [
      "CSS prefers-reduced-motion",
      "CSS prefers-reduced-transparency",
      "CSS forced-colors",
      "CSS prefers-contrast",
      "CSS prefers-color-scheme",
      "User-Agent Client Hints",
    ],
    dependencies: ["context"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
