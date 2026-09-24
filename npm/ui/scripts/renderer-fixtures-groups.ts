import { audioPlayerRendererFixtures } from "./renderer-fixtures-audio-player.ts";
import { avatarGroupRendererFixtures } from "./renderer-fixtures-avatar-group.ts";
import { colorPickerRendererFixtures } from "./renderer-fixtures-color-picker.ts";
import { commandRendererFixtures } from "./renderer-fixtures-commands.ts";
import { dataRendererFixtures } from "./renderer-fixtures-data.ts";
import { dialogRendererFixtures } from "./renderer-fixtures-dialog.ts";
import { disclosureRendererFixtures } from "./renderer-fixtures-disclosure.ts";
import { feedbackRendererFixtures } from "./renderer-fixtures-feedback.ts";
import { fileUploadRendererFixtures } from "./renderer-fixtures-file-upload.ts";
import { formInputRendererFixtures } from "./renderer-fixtures-form-inputs.ts";
import { iconRendererFixtures } from "./renderer-fixtures-icon.ts";
import { imageCropperRendererFixtures } from "./renderer-fixtures-image-cropper.ts";
import { layoutRendererFixtures } from "./renderer-fixtures-layout.ts";
import { lightboxRendererFixtures } from "./renderer-fixtures-lightbox.ts";
import { marqueeRendererFixtures } from "./renderer-fixtures-marquee.ts";
import { mediaPlayerRendererFixtures } from "./renderer-fixtures-media-player.ts";
import { mediaRendererFixtures } from "./renderer-fixtures-media.ts";
import { navigationRendererFixtures } from "./renderer-fixtures-navigation.ts";
import { overlayRendererFixtures } from "./renderer-fixtures-overlays.ts";
import { primitiveRendererFixtures } from "./renderer-fixtures-primitives.ts";
import { qrCodeRendererFixtures } from "./renderer-fixtures-qr-code.ts";
import { selectionRendererFixtures } from "./renderer-fixtures-selection.ts";
import { signaturePadRendererFixtures } from "./renderer-fixtures-signature-pad.ts";
import { structureRendererFixtures } from "./renderer-fixtures-structure.ts";
import { tourRendererFixtures } from "./renderer-fixtures-tour.ts";
import { videoPlayerRendererFixtures } from "./renderer-fixtures-video-player.ts";

export const groupedRendererFixtures = [
  ...audioPlayerRendererFixtures,
  ...avatarGroupRendererFixtures,
  ...colorPickerRendererFixtures,
  ...commandRendererFixtures,
  ...dataRendererFixtures,
  ...dialogRendererFixtures,
  ...disclosureRendererFixtures,
  ...feedbackRendererFixtures,
  ...fileUploadRendererFixtures,
  ...formInputRendererFixtures,
  ...iconRendererFixtures,
  ...imageCropperRendererFixtures,
  ...layoutRendererFixtures,
  ...lightboxRendererFixtures,
  ...marqueeRendererFixtures,
  ...mediaPlayerRendererFixtures,
  ...mediaRendererFixtures,
  ...navigationRendererFixtures,
  ...overlayRendererFixtures,
  ...primitiveRendererFixtures,
  ...qrCodeRendererFixtures,
  ...selectionRendererFixtures,
  ...signaturePadRendererFixtures,
  ...structureRendererFixtures,
  ...tourRendererFixtures,
  ...videoPlayerRendererFixtures,
] as const;
