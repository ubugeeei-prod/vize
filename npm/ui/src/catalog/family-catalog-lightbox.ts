import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const lightboxFamilyRoot = "src/families/media/lightbox/";

export const lightboxFamilyCatalog = [
  {
    canonicalName: "lightbox",
    title: "Lightbox",
    packageSubpath: "./lightbox",
    entryFile: `${lightboxFamilyRoot}lightbox.ts`,
    sourceFiles: [
      `${lightboxFamilyRoot}lightbox-close.vue`,
      `${lightboxFamilyRoot}lightbox-content.vue`,
      `${lightboxFamilyRoot}lightbox-context.ts`,
      `${lightboxFamilyRoot}lightbox-counter.vue`,
      `${lightboxFamilyRoot}lightbox-image.vue`,
      `${lightboxFamilyRoot}lightbox-item.vue`,
      `${lightboxFamilyRoot}lightbox-next.vue`,
      `${lightboxFamilyRoot}lightbox-previous.vue`,
      `${lightboxFamilyRoot}lightbox-root.vue`,
      `${lightboxFamilyRoot}lightbox-state.ts`,
      `${lightboxFamilyRoot}lightbox-thumbnail.vue`,
      `${lightboxFamilyRoot}lightbox-thumbnails.vue`,
      `${lightboxFamilyRoot}lightbox-trigger.vue`,
      `${lightboxFamilyRoot}lightbox-types.ts`,
      `${lightboxFamilyRoot}lightbox.ts`,
    ],
    behaviorContract: `${lightboxFamilyRoot}lightbox.behavior.md`,
    tests: [`${lightboxFamilyRoot}lightbox.test.ts`, `${lightboxFamilyRoot}lightbox-ssr.test.ts`],
    typeTests: [`${lightboxFamilyRoot}lightbox.types.test-d.ts`],
    rendererFixture: "LightboxConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "LightboxRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}lightbox-root",
      allowedRetainedFamilies: ["context", "controllable-state", "dialog"],
      maximumJavaScriptGzipBytes: 3_500,
      maximumCssGzipBytes: 0,
    },
    aliases: ["lightbox", "gallery", "image viewer", "media viewer", "photo viewer"],
    upstreamCoverage: [
      "WAI-ARIA Dialog (modal)",
      "WAI-ARIA Tabs (thumbnail picker)",
      "PhotoSwipe",
      "yet-another-react-lightbox",
    ],
    dependencies: ["context", "controllable-state", "dialog", "id", "image"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
