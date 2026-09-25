import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const scrubberPreviewFamilyRoot = "src/families/media/scrubber-preview/";

export const scrubberPreviewFamilyCatalog = [
  {
    canonicalName: "scrubber-preview",
    title: "Scrubber Preview",
    packageSubpath: "./scrubber-preview",
    entryFile: `${scrubberPreviewFamilyRoot}scrubber-preview.ts`,
    sourceFiles: [
      `${scrubberPreviewFamilyRoot}scrubber-preview-root.vue`,
      `${scrubberPreviewFamilyRoot}scrubber-preview-thumbnail.vue`,
      `${scrubberPreviewFamilyRoot}scrubber-preview-time.vue`,
      `${scrubberPreviewFamilyRoot}scrubber-preview-track.vue`,
      `${scrubberPreviewFamilyRoot}scrubber-preview.ts`,
      `${scrubberPreviewFamilyRoot}scrubber-preview-capture.ts`,
      `${scrubberPreviewFamilyRoot}scrubber-preview-context.ts`,
      `${scrubberPreviewFamilyRoot}scrubber-preview-thumbnails.ts`,
      `${scrubberPreviewFamilyRoot}scrubber-preview-types.ts`,
    ],
    behaviorContract: `${scrubberPreviewFamilyRoot}scrubber-preview.behavior.md`,
    tests: [
      `${scrubberPreviewFamilyRoot}scrubber-preview.test.ts`,
      `${scrubberPreviewFamilyRoot}scrubber-preview-ssr.test.ts`,
      `${scrubberPreviewFamilyRoot}scrubber-preview-thumbnails.test.ts`,
    ],
    typeTests: [`${scrubberPreviewFamilyRoot}scrubber-preview.types.test-d.ts`],
    rendererFixture: "ScrubberPreviewConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "ScrubberPreviewRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}scrubber-preview-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 2_850,
      maximumCssGzipBytes: 0,
    },
    aliases: [
      "scrubber preview",
      "video thumbnail",
      "seek preview",
      "storyboard",
      "trick play",
      "thumbnail track",
    ],
    upstreamCoverage: [
      "WebVTT thumbnail tracks (JW Player / Video.js convention)",
      "Media Fragments URI xywh",
      "Vidstack slider thumbnails",
      "HTMLVideoElement + canvas frame capture",
    ],
    dependencies: ["context", "controllable-state"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
