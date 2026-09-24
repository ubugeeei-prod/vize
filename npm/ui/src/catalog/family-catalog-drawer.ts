import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const drawerFamilyRoot = "src/families/overlays/drawer/";
const confirmFamilyRoot = "src/families/overlays/confirm/";

export const drawerFamilyCatalog = [
  {
    canonicalName: "drawer",
    title: "Drawer",
    packageSubpath: "./drawer",
    entryFile: `${drawerFamilyRoot}drawer.ts`,
    sourceFiles: [
      `${drawerFamilyRoot}drawer-content.vue`,
      `${drawerFamilyRoot}drawer-context.ts`,
      `${drawerFamilyRoot}drawer-handle.vue`,
      `${drawerFamilyRoot}drawer-root.vue`,
      `${drawerFamilyRoot}drawer-snap.ts`,
      `${drawerFamilyRoot}drawer.ts`,
      `${drawerFamilyRoot}drawer-types.ts`,
    ],
    behaviorContract: `${drawerFamilyRoot}drawer.behavior.md`,
    tests: [`${drawerFamilyRoot}drawer.test.ts`, `${drawerFamilyRoot}drawer-ssr.test.ts`],
    typeTests: [`${drawerFamilyRoot}drawer.types.test-d.ts`],
    rendererFixture: "DrawerConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "DrawerRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,32}drawer-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 3_500,
      maximumCssGzipBytes: 0,
    },
    aliases: ["sheet", "bottom sheet", "side sheet", "off-canvas", "vaul"],
    upstreamCoverage: [
      "HTML dialog element",
      "WAI-ARIA dialog pattern",
      "Vaul Drawer",
      "Radix Dialog sheet recipes",
    ],
    dependencies: [
      "context",
      "controllable-state",
      "dialog",
      "dismissable-layer",
      "focus-scope",
      "id",
      "scroll-lock",
    ],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];

export const confirmFamilyCatalog = [
  {
    canonicalName: "confirm",
    title: "Confirm",
    packageSubpath: "./confirm",
    entryFile: `${confirmFamilyRoot}confirm.ts`,
    sourceFiles: [
      `${confirmFamilyRoot}confirm-provider.vue`,
      `${confirmFamilyRoot}confirm-runtime.ts`,
      `${confirmFamilyRoot}confirm.ts`,
      `${confirmFamilyRoot}confirm-types.ts`,
    ],
    behaviorContract: `${confirmFamilyRoot}confirm.behavior.md`,
    tests: [`${confirmFamilyRoot}confirm.test.ts`, `${confirmFamilyRoot}confirm-ssr.test.ts`],
    typeTests: [`${confirmFamilyRoot}confirm.types.test-d.ts`],
    rendererFixture: "ConfirmConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "ConfirmProvider",
      retainedSignature: "confirm-provider",
      allowedRetainedFamilies: [
        "alert-dialog",
        "context",
        "controllable-state",
        "dialog",
        "dismissable-layer",
        "focus-guards",
        "focus-scope",
        "inert-outside",
        "portal",
        "scroll-lock",
      ],
      maximumJavaScriptGzipBytes: 15_600,
      maximumCssGzipBytes: 0,
    },
    aliases: ["useConfirm", "confirm dialog", "promise dialog", "window.confirm replacement"],
    upstreamCoverage: [
      "window.confirm",
      "WAI-ARIA alertdialog pattern",
      "Radix Alert Dialog",
      "Mantine modals.openConfirmModal",
    ],
    dependencies: ["alert-dialog", "context", "dialog", "id", "portal"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
