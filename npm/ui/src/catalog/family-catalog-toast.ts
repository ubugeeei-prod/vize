import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const toastFamilyRoot = "src/families/feedback/toast/";

export const toastFamilyCatalog = [
  {
    canonicalName: "toast",
    title: "Toast",
    packageSubpath: "./toast",
    entryFile: `${toastFamilyRoot}toast.ts`,
    sourceFiles: [
      `${toastFamilyRoot}toast-action.vue`,
      `${toastFamilyRoot}toast-close.vue`,
      `${toastFamilyRoot}toast-context.ts`,
      `${toastFamilyRoot}toast-description.vue`,
      `${toastFamilyRoot}toast-hotkey.ts`,
      `${toastFamilyRoot}toast-provider.vue`,
      `${toastFamilyRoot}toast-root.vue`,
      `${toastFamilyRoot}toast-store.ts`,
      `${toastFamilyRoot}toast-title.vue`,
      `${toastFamilyRoot}toast-types.ts`,
      `${toastFamilyRoot}toast-viewport.vue`,
      `${toastFamilyRoot}toast.ts`,
      `${toastFamilyRoot}use-toast.ts`,
    ],
    behaviorContract: `${toastFamilyRoot}toast.behavior.md`,
    tests: [
      `${toastFamilyRoot}toast.test.ts`,
      `${toastFamilyRoot}toast-ssr.test.ts`,
      `${toastFamilyRoot}toast-store.test.ts`,
    ],
    typeTests: [`${toastFamilyRoot}toast.types.test-d.ts`],
    rendererFixture: "ToastConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "ToastProvider",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}toast-provider",
      allowedRetainedFamilies: ["context"],
      maximumJavaScriptGzipBytes: 6_000,
      maximumCssGzipBytes: 0,
    },
    aliases: ["toast", "toaster", "snackbar", "notification", "sonner"],
    upstreamCoverage: [
      "WAI-ARIA live regions",
      "Radix Toast",
      "Sonner",
      "React Aria Toast",
      "Reka UI Toast",
    ],
    dependencies: ["context", "id", "live-region", "presence"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
