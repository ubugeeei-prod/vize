import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";
import { audioPlayerRuntimeFixtures } from "../families/media/audio-player/runtime-conformance-audio-player-fixtures.ts";
import { avatarGroupRuntimeFixtures } from "../families/layout/avatar-group/runtime-conformance-avatar-group-fixtures.ts";
import { carouselRuntimeFixtures } from "../families/media/carousel/runtime-conformance-carousel-fixtures.ts";
import { colorPickerRuntimeFixtures } from "../families/form/color-picker/runtime-conformance-color-picker-fixtures.ts";
import { fileUploadRuntimeFixtures } from "../families/form/file-upload/runtime-conformance-file-upload-fixtures.ts";
import { imageRuntimeFixtures } from "../families/media/image/runtime-conformance-image-fixtures.ts";
import { imageCropperRuntimeFixtures } from "../families/media/image-cropper/runtime-conformance-image-cropper-fixtures.ts";
import { infiniteScrollRuntimeFixtures } from "../families/data/infinite-scroll/runtime-conformance-infinite-scroll-fixtures.ts";
import { lightboxRuntimeFixtures } from "../families/media/lightbox/runtime-conformance-lightbox-fixtures.ts";
import { marqueeRuntimeFixtures } from "../families/media/marquee/runtime-conformance-marquee-fixtures.ts";
import { mediaPlayerRuntimeFixtures } from "../families/media/media-player/runtime-conformance-media-player-fixtures.ts";
import { qrCodeRuntimeFixtures } from "../families/media/qr-code/runtime-conformance-qr-code-fixtures.ts";
import { signaturePadRuntimeFixtures } from "../families/media/signature-pad/runtime-conformance-signature-pad-fixtures.ts";
import { tourRuntimeFixtures } from "../families/overlays/tour/runtime-conformance-tour-fixtures.ts";
import { videoPlayerRuntimeFixtures } from "../families/media/video-player/runtime-conformance-video-player-fixtures.ts";

/** SSR and hydration fixtures for rich media, picker, upload, and onboarding families. */
export const richRuntimeFixtures: readonly RuntimeFixture[] = [
  ...audioPlayerRuntimeFixtures,
  ...avatarGroupRuntimeFixtures,
  ...carouselRuntimeFixtures,
  ...colorPickerRuntimeFixtures,
  ...fileUploadRuntimeFixtures,
  ...imageRuntimeFixtures,
  ...imageCropperRuntimeFixtures,
  ...infiniteScrollRuntimeFixtures,
  ...lightboxRuntimeFixtures,
  ...marqueeRuntimeFixtures,
  ...mediaPlayerRuntimeFixtures,
  ...qrCodeRuntimeFixtures,
  ...signaturePadRuntimeFixtures,
  ...tourRuntimeFixtures,
  ...videoPlayerRuntimeFixtures,
];
