import {
  catalogOwner,
  componentQualityGates,
  type UiFamilyCatalogEntry,
} from "./family-catalog-types.ts";

const imageCropperFamilyRoot = "src/families/media/image-cropper/";

export const imageCropperFamilyCatalog = [
  {
    canonicalName: "image-cropper",
    title: "Image Cropper",
    packageSubpath: "./image-cropper",
    entryFile: `${imageCropperFamilyRoot}image-cropper.ts`,
    sourceFiles: [
      `${imageCropperFamilyRoot}image-cropper-area.vue`,
      `${imageCropperFamilyRoot}image-cropper-canvas.ts`,
      `${imageCropperFamilyRoot}image-cropper-context.ts`,
      `${imageCropperFamilyRoot}image-cropper-geometry.ts`,
      `${imageCropperFamilyRoot}image-cropper-grid.vue`,
      `${imageCropperFamilyRoot}image-cropper-handle.vue`,
      `${imageCropperFamilyRoot}image-cropper-image.vue`,
      `${imageCropperFamilyRoot}image-cropper-pointer.ts`,
      `${imageCropperFamilyRoot}image-cropper-root.vue`,
      `${imageCropperFamilyRoot}image-cropper-types.ts`,
      `${imageCropperFamilyRoot}image-cropper-viewport.vue`,
      `${imageCropperFamilyRoot}image-cropper.ts`,
    ],
    behaviorContract: `${imageCropperFamilyRoot}image-cropper.behavior.md`,
    tests: [
      `${imageCropperFamilyRoot}image-cropper.test.ts`,
      `${imageCropperFamilyRoot}image-cropper-ssr.test.ts`,
      `${imageCropperFamilyRoot}image-cropper-geometry.test.ts`,
      `${imageCropperFamilyRoot}image-cropper-canvas.test.ts`,
    ],
    typeTests: [`${imageCropperFamilyRoot}image-cropper.types.test-d.ts`],
    rendererFixture: "ImageCropperConsumer.vue",
    qualityGates: componentQualityGates,
    bundleBudget: {
      exportName: "ImageCropperRoot",
      retainedSignature: "data-vize-ui[\\s\\S]{0,8}image-cropper-root",
      allowedRetainedFamilies: ["context", "controllable-state"],
      maximumJavaScriptGzipBytes: 5_000,
      maximumCssGzipBytes: 0,
    },
    aliases: ["image cropper", "crop", "avatar cropper", "photo editor", "image crop"],
    upstreamCoverage: ["Cropper.js", "react-easy-crop", "react-image-crop", "Ark UI Image Cropper"],
    dependencies: ["context", "controllable-state", "id", "measure"],
    maturity: "stable",
    owner: catalogOwner,
  },
] as const satisfies readonly UiFamilyCatalogEntry[];
