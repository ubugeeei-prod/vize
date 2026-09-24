import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";
import { carouselRuntimeFixtures } from "../families/media/carousel/runtime-conformance-carousel-fixtures.ts";
import { colorPickerRuntimeFixtures } from "../families/form/color-picker/runtime-conformance-color-picker-fixtures.ts";
import { fileUploadRuntimeFixtures } from "../families/form/file-upload/runtime-conformance-file-upload-fixtures.ts";
import { imageRuntimeFixtures } from "../families/media/image/runtime-conformance-image-fixtures.ts";
import { infiniteScrollRuntimeFixtures } from "../families/data/infinite-scroll/runtime-conformance-infinite-scroll-fixtures.ts";
import { qrCodeRuntimeFixtures } from "../families/media/qr-code/runtime-conformance-qr-code-fixtures.ts";
import { tourRuntimeFixtures } from "../families/overlays/tour/runtime-conformance-tour-fixtures.ts";

/** SSR and hydration fixtures for rich media, picker, upload, and onboarding families. */
export const richRuntimeFixtures: readonly RuntimeFixture[] = [
  ...carouselRuntimeFixtures,
  ...colorPickerRuntimeFixtures,
  ...fileUploadRuntimeFixtures,
  ...imageRuntimeFixtures,
  ...infiniteScrollRuntimeFixtures,
  ...qrCodeRuntimeFixtures,
  ...tourRuntimeFixtures,
];
