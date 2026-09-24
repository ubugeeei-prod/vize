import {
  catalogOwner,
  componentQualityGates,
  interactionQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const auditRoot = "src/families/accessibility/a11y-audit/";
const directionRoot = "src/families/i18n/direction/";

export const a11yAuditFamilyCatalog = [
  {
    canonicalName: "a11y-audit",
    title: "Accessibility Audit",
    packageSubpath: "./a11y-audit",
    entryFile: `${auditRoot}a11y-audit.ts`,
    sourceFiles: [
      `${auditRoot}a11y-audit-rules.ts`,
      `${auditRoot}a11y-audit-runtime.ts`,
      `${auditRoot}a11y-audit-types.ts`,
      `${auditRoot}a11y-audit.ts`,
    ],
    behaviorContract: `${auditRoot}a11y-audit.behavior.md`,
    tests: [`${auditRoot}a11y-audit.test.ts`],
    typeTests: [`${auditRoot}a11y-audit.types.test-d.ts`],
    rendererFixture: "A11yAuditConsumer.vue",
    qualityGates: interactionQualityGates,
    bundleBudget: {
      exportName: "auditAccessibility",
      retainedSignature: "VIZE_UI_A11Y_AUDIT",
      allowedRetainedFamilies: [],
      maximumJavaScriptGzipBytes: 1_500,
      maximumCssGzipBytes: 0,
    },
    aliases: ["a11y audit", "accessibility linter", "accessible name check", "axe-lite"],
    upstreamCoverage: [
      "WAI-ARIA accessible name computation",
      "axe-core name rules",
      "Vue devtools warnings",
    ],
    dependencies: [],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];

export const directionFamilyCatalog = [
  {
    canonicalName: "direction",
    title: "Direction",
    packageSubpath: "./direction",
    entryFile: `${directionRoot}direction.ts`,
    sourceFiles: [
      `${directionRoot}direction-provider.vue`,
      `${directionRoot}direction-runtime.ts`,
      `${directionRoot}direction-types.ts`,
      `${directionRoot}direction.ts`,
    ],
    behaviorContract: `${directionRoot}direction.behavior.md`,
    tests: [`${directionRoot}direction.test.ts`, `${directionRoot}direction-ssr.test.ts`],
    typeTests: [`${directionRoot}direction.types.test-d.ts`],
    rendererFixture: "families/i18n/direction/direction-provider.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "DirectionProvider",
      retainedSignature: "data-vize-ui[^-a-z]{0,8}direction-provider",
      allowedRetainedFamilies: ["context"],
      maximumJavaScriptGzipBytes: 1_200,
      maximumCssGzipBytes: 0,
    },
    aliases: ["direction provider", "rtl", "bidi", "writing direction"],
    upstreamCoverage: [
      "HTML dir attribute",
      "Radix DirectionProvider",
      "React Aria useLocale direction",
    ],
    dependencies: ["context"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
