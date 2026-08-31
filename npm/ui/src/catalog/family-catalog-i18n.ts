import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const localeFamilyRoot = "src/families/i18n/locale/";

export const i18nFamilyCatalog = [
  {
    canonicalName: "locale",
    title: "Locale and Direction",
    packageSubpath: "./locale",
    entryFile: `${localeFamilyRoot}locale.ts`,
    sourceFiles: [
      `${localeFamilyRoot}locale-provider.vue`,
      `${localeFamilyRoot}locale.ts`,
      `${localeFamilyRoot}locale-runtime.ts`,
      `${localeFamilyRoot}locale-text.ts`,
    ],
    behaviorContract: `${localeFamilyRoot}locale.behavior.md`,
    tests: [`${localeFamilyRoot}locale.test.ts`, `${localeFamilyRoot}locale-ssr.test.ts`],
    typeTests: [`${localeFamilyRoot}locale.types.test-d.ts`],
    rendererFixture: "families/i18n/locale/locale-provider.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "LocaleProvider",
      retainedSignature: "data-vize-ui.+locale",
      allowedRetainedFamilies: ["context"],
      maximumJavaScriptGzipBytes: 1_400,
      maximumCssGzipBytes: 0,
    },
    aliases: ["direction", "rtl", "writing mode", "i18n provider"],
    upstreamCoverage: ["HTML dir", "HTML lang", "Intl.Locale"],
    dependencies: ["context"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
