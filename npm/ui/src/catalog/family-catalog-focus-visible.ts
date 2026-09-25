import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const familyRoot = "src/families/accessibility/focus-visible/";

export const focusVisibleFamilyCatalog = [
  {
    canonicalName: "focus-visible",
    title: "Focus Visible",
    packageSubpath: "./focus-visible",
    entryFile: `${familyRoot}focus-visible.ts`,
    sourceFiles: [
      `${familyRoot}focus-visible-context.ts`,
      `${familyRoot}focus-visible-provider.vue`,
      `${familyRoot}focus-visible-runtime.ts`,
      `${familyRoot}focus-visible-types.ts`,
      `${familyRoot}focus-visible.ts`,
    ],
    behaviorContract: `${familyRoot}focus-visible.behavior.md`,
    tests: [`${familyRoot}focus-visible.test.ts`, `${familyRoot}focus-visible-ssr.test.ts`],
    typeTests: [`${familyRoot}focus-visible.types.test-d.ts`],
    rendererFixture: "FocusVisibleConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "FocusVisibleProvider",
      retainedSignature: "data-vize-ui[^-a-z]{0,8}focus-visible-provider",
      allowedRetainedFamilies: ["context", "interaction-modality"],
      maximumJavaScriptGzipBytes: 2_900,
      maximumCssGzipBytes: 0,
    },
    aliases: ["focus ring", "focus-visible", "keyboard focus indicator"],
    upstreamCoverage: ["CSS :focus-visible", "React Aria FocusRing", "WCAG 2.4.7 Focus Visible"],
    dependencies: ["context", "interaction-modality"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
