import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const panZoomFamilyRoot = "src/families/media/pan-zoom/";

export const panZoomFamilyCatalog = [
  {
    canonicalName: "pan-zoom",
    title: "Pan Zoom",
    packageSubpath: "./pan-zoom",
    entryFile: `${panZoomFamilyRoot}pan-zoom.ts`,
    sourceFiles: [
      `${panZoomFamilyRoot}pan-zoom-content.vue`,
      `${panZoomFamilyRoot}pan-zoom-context.ts`,
      `${panZoomFamilyRoot}pan-zoom-fit.vue`,
      `${panZoomFamilyRoot}pan-zoom-reset.vue`,
      `${panZoomFamilyRoot}pan-zoom-root.vue`,
      `${panZoomFamilyRoot}pan-zoom-status.vue`,
      `${panZoomFamilyRoot}pan-zoom-transform.ts`,
      `${panZoomFamilyRoot}pan-zoom-types.ts`,
      `${panZoomFamilyRoot}pan-zoom-viewport.vue`,
      `${panZoomFamilyRoot}pan-zoom-zoom-in.vue`,
      `${panZoomFamilyRoot}pan-zoom-zoom-out.vue`,
      `${panZoomFamilyRoot}pan-zoom.ts`,
    ],
    behaviorContract: `${panZoomFamilyRoot}pan-zoom.behavior.md`,
    tests: [
      `${panZoomFamilyRoot}pan-zoom.test.ts`,
      `${panZoomFamilyRoot}pan-zoom-ssr.test.ts`,
      `${panZoomFamilyRoot}pan-zoom-transform.test.ts`,
    ],
    typeTests: [`${panZoomFamilyRoot}pan-zoom.types.test-d.ts`],
    rendererFixture: "PanZoomConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "PanZoomRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}pan-zoom-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 2_700,
      maximumCssGzipBytes: 0,
    },
    aliases: ["pan zoom", "zoomable", "zoom pan pinch", "image zoom", "canvas viewport"],
    upstreamCoverage: [
      "react-zoom-pan-pinch",
      "panzoom",
      "Pointer Events pinch gestures",
      "WheelEvent deltaMode",
    ],
    dependencies: ["context", "controllable-state"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
