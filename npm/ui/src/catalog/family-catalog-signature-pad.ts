import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const signaturePadFamilyRoot = "src/families/media/signature-pad/";

export const signaturePadFamilyCatalog = [
  {
    canonicalName: "signature-pad",
    title: "Signature Pad",
    packageSubpath: "./signature-pad",
    entryFile: `${signaturePadFamilyRoot}signature-pad.ts`,
    sourceFiles: [
      `${signaturePadFamilyRoot}signature-pad-canvas.vue`,
      `${signaturePadFamilyRoot}signature-pad-clear.vue`,
      `${signaturePadFamilyRoot}signature-pad-context.ts`,
      `${signaturePadFamilyRoot}signature-pad-guide.vue`,
      `${signaturePadFamilyRoot}signature-pad-path.ts`,
      `${signaturePadFamilyRoot}signature-pad-redo.vue`,
      `${signaturePadFamilyRoot}signature-pad-root.vue`,
      `${signaturePadFamilyRoot}signature-pad-types.ts`,
      `${signaturePadFamilyRoot}signature-pad-undo.vue`,
      `${signaturePadFamilyRoot}signature-pad.ts`,
    ],
    behaviorContract: `${signaturePadFamilyRoot}signature-pad.behavior.md`,
    tests: [
      `${signaturePadFamilyRoot}signature-pad.test.ts`,
      `${signaturePadFamilyRoot}signature-pad-ssr.test.ts`,
      `${signaturePadFamilyRoot}signature-pad-path.test.ts`,
    ],
    typeTests: [`${signaturePadFamilyRoot}signature-pad.types.test-d.ts`],
    rendererFixture: "SignaturePadConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "SignaturePadRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}signature-pad-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 4_150,
      maximumCssGzipBytes: 0,
    },
    aliases: ["signature pad", "signature", "e-signature", "drawing pad", "sign here"],
    upstreamCoverage: [
      "signature_pad",
      "perfect-freehand",
      "Pointer Events getCoalescedEvents",
      "Mantine Signature examples",
    ],
    dependencies: ["context", "controllable-state", "id"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
