import { audioPlayerRendererFixtures } from "./renderer-fixtures-audio-player.ts";
import { audioVisualizerRendererFixtures } from "./renderer-fixtures-audio-visualizer.ts";
import { avatarGroupRendererFixtures } from "./renderer-fixtures-avatar-group.ts";
import { chartRendererFixtures } from "./renderer-fixtures-charts.ts";
import { colorPickerRendererFixtures } from "./renderer-fixtures-color-picker.ts";
import { commandPaletteRendererFixtures } from "./renderer-fixtures-command-palette.ts";
import { commandRendererFixtures } from "./renderer-fixtures-commands.ts";
import { dataRendererFixtures } from "./renderer-fixtures-data.ts";
import { dataViewRendererFixtures } from "./renderer-fixtures-data-views.ts";
import { dialogRendererFixtures } from "./renderer-fixtures-dialog.ts";
import { disclosureRendererFixtures } from "./renderer-fixtures-disclosure.ts";
import { drawerRendererFixtures } from "./renderer-fixtures-drawer.ts";
import { feedbackRendererFixtures } from "./renderer-fixtures-feedback.ts";
import { fileUploadRendererFixtures } from "./renderer-fixtures-file-upload.ts";
import { formCompositeRendererFixtures } from "./renderer-fixtures-form-composites.ts";
import { formInputRendererFixtures } from "./renderer-fixtures-form-inputs.ts";
import { formStructureRendererFixtures } from "./renderer-fixtures-form-structure.ts";
import { hotspotRendererFixtures } from "./renderer-fixtures-hotspot.ts";
import { iconRendererFixtures } from "./renderer-fixtures-icon.ts";
import { imageCompareRendererFixtures } from "./renderer-fixtures-image-compare.ts";
import { imageCropperRendererFixtures } from "./renderer-fixtures-image-cropper.ts";
import { layoutRendererFixtures } from "./renderer-fixtures-layout.ts";
import { lightboxRendererFixtures } from "./renderer-fixtures-lightbox.ts";
import { marqueeRendererFixtures } from "./renderer-fixtures-marquee.ts";
import { mediaPlayerRendererFixtures } from "./renderer-fixtures-media-player.ts";
import { mediaRendererFixtures } from "./renderer-fixtures-media.ts";
import { menuRendererFixtures } from "./renderer-fixtures-menus.ts";
import { mobileRendererFixtures } from "./renderer-fixtures-mobile.ts";
import { navigationRendererFixtures } from "./renderer-fixtures-navigation.ts";
import { overlay3aRendererFixtures } from "./renderer-fixtures-overlay-3a.ts";
import { overlay3bRendererFixtures } from "./renderer-fixtures-overlay-3b.ts";
import { overlay3cRendererFixtures } from "./renderer-fixtures-overlay-3c.ts";
import { overlayRendererFixtures } from "./renderer-fixtures-overlays.ts";
import { panZoomRendererFixtures } from "./renderer-fixtures-pan-zoom.ts";
import { primitiveRendererFixtures } from "./renderer-fixtures-primitives.ts";
import { qrCodeRendererFixtures } from "./renderer-fixtures-qr-code.ts";
import { scrubberPreviewRendererFixtures } from "./renderer-fixtures-scrubber-preview.ts";
import { selectionRendererFixtures } from "./renderer-fixtures-selection.ts";
import { signaturePadRendererFixtures } from "./renderer-fixtures-signature-pad.ts";
import { structureRendererFixtures } from "./renderer-fixtures-structure.ts";
import { toastRendererFixtures } from "./renderer-fixtures-toast.ts";
import { tourRendererFixtures } from "./renderer-fixtures-tour.ts";
import { videoPlayerRendererFixtures } from "./renderer-fixtures-video-player.ts";
import { wayfindingRendererFixtures } from "./renderer-fixtures-wayfinding.ts";
import { webcamCaptureRendererFixtures } from "./renderer-fixtures-webcam-capture.ts";

export const groupedRendererFixtures = [
  ...audioPlayerRendererFixtures,
  ...audioVisualizerRendererFixtures,
  ...avatarGroupRendererFixtures,
  ...chartRendererFixtures,
  ...colorPickerRendererFixtures,
  ...commandPaletteRendererFixtures,
  ...commandRendererFixtures,
  ...dataRendererFixtures,
  ...dataViewRendererFixtures,
  ...dialogRendererFixtures,
  ...disclosureRendererFixtures,
  ...drawerRendererFixtures,
  ...feedbackRendererFixtures,
  ...fileUploadRendererFixtures,
  ...formCompositeRendererFixtures,
  ...formInputRendererFixtures,
  ...formStructureRendererFixtures,
  ...hotspotRendererFixtures,
  ...iconRendererFixtures,
  ...imageCompareRendererFixtures,
  ...imageCropperRendererFixtures,
  ...layoutRendererFixtures,
  ...lightboxRendererFixtures,
  ...marqueeRendererFixtures,
  ...mediaPlayerRendererFixtures,
  ...mediaRendererFixtures,
  ...menuRendererFixtures,
  ...mobileRendererFixtures,
  ...navigationRendererFixtures,
  ...overlay3aRendererFixtures,
  ...overlay3bRendererFixtures,
  ...overlay3cRendererFixtures,
  ...overlayRendererFixtures,
  ...panZoomRendererFixtures,
  ...primitiveRendererFixtures,
  ...qrCodeRendererFixtures,
  ...scrubberPreviewRendererFixtures,
  ...selectionRendererFixtures,
  ...signaturePadRendererFixtures,
  ...structureRendererFixtures,
  ...toastRendererFixtures,
  ...tourRendererFixtures,
  ...videoPlayerRendererFixtures,
  ...wayfindingRendererFixtures,
  ...webcamCaptureRendererFixtures,
] as const;
