import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const webcamCaptureFamilyRoot = "src/families/media/webcam-capture/";

export const webcamCaptureFamilyCatalog = [
  {
    canonicalName: "webcam-capture",
    title: "Webcam Capture",
    packageSubpath: "./webcam-capture",
    entryFile: `${webcamCaptureFamilyRoot}webcam-capture.ts`,
    sourceFiles: [
      `${webcamCaptureFamilyRoot}webcam-capture-context.ts`,
      `${webcamCaptureFamilyRoot}webcam-capture-device-select.vue`,
      `${webcamCaptureFamilyRoot}webcam-capture-frame.ts`,
      `${webcamCaptureFamilyRoot}webcam-capture-media.ts`,
      `${webcamCaptureFamilyRoot}webcam-capture-photo.vue`,
      `${webcamCaptureFamilyRoot}webcam-capture-root.vue`,
      `${webcamCaptureFamilyRoot}webcam-capture-shutter.vue`,
      `${webcamCaptureFamilyRoot}webcam-capture-start-button.vue`,
      `${webcamCaptureFamilyRoot}webcam-capture-status-message.vue`,
      `${webcamCaptureFamilyRoot}webcam-capture-stop-button.vue`,
      `${webcamCaptureFamilyRoot}webcam-capture-switch-camera.vue`,
      `${webcamCaptureFamilyRoot}webcam-capture-types.ts`,
      `${webcamCaptureFamilyRoot}webcam-capture-video.vue`,
      `${webcamCaptureFamilyRoot}webcam-capture.ts`,
    ],
    behaviorContract: `${webcamCaptureFamilyRoot}webcam-capture.behavior.md`,
    tests: [
      `${webcamCaptureFamilyRoot}webcam-capture.test.ts`,
      `${webcamCaptureFamilyRoot}webcam-capture-ssr.test.ts`,
      `${webcamCaptureFamilyRoot}webcam-capture-media.test.ts`,
    ],
    typeTests: [`${webcamCaptureFamilyRoot}webcam-capture.types.test-d.ts`],
    rendererFixture: "WebcamCaptureConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "WebcamCaptureRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}webcam-capture-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 4_450,
      maximumCssGzipBytes: 0,
    },
    aliases: ["webcam", "camera capture", "photo booth", "selfie camera", "getUserMedia"],
    upstreamCoverage: [
      "MediaDevices.getUserMedia",
      "HTMLCanvasElement.toBlob",
      "@vizejs/composable useUserMedia",
      "react-webcam",
    ],
    dependencies: ["context", "controllable-state", "id"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
