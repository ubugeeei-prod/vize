import { accordionFamilyCatalog } from "./family-catalog-accordion.ts";
import { popconfirmFamilyCatalog } from "./family-catalog-popconfirm.ts";
import { notificationCenterFamilyCatalog } from "./family-catalog-notification-center.ts";
import { commandPaletteFamilyCatalog } from "./family-catalog-command-palette.ts";
import { floatingActionButtonFamilyCatalog } from "./family-catalog-floating-action-button.ts";
import { backToTopFamilyCatalog } from "./family-catalog-back-to-top.ts";
import { sidebarFamilyCatalog } from "./family-catalog-sidebar.ts";
import { resizableFamilyCatalog } from "./family-catalog-resizable.ts";
import { stickyFamilyCatalog } from "./family-catalog-sticky.ts";
import { actionFamilyCatalog } from "./family-catalog-actions.ts";
import { accessibilityFamilyCatalog } from "./family-catalog-accessibility.ts";
import { basicFamilyCatalog } from "./family-catalog-basics.ts";
import { chartFamilyCatalog } from "./family-catalog-charts.ts";
import { dataFamilyCatalog } from "./family-catalog-data.ts";
import { dataViewFamilyCatalog } from "./family-catalog-data-views.ts";
import { dateTimeFamilyCatalog } from "./family-catalog-date-time.ts";
import { confirmFamilyCatalog, drawerFamilyCatalog } from "./family-catalog-drawer.ts";
import { feedbackFamilyCatalog } from "./family-catalog-feedback.ts";
import { focusFamilyCatalog } from "./family-catalog-focus.ts";
import { foundationFamilyCatalog } from "./family-catalog-foundations.ts";
import { formInputFamilyCatalog } from "./family-catalog-form-inputs.ts";
import { helperFamilyCatalog } from "./family-catalog-helpers.ts";
import { formStructureFamilyCatalog } from "./family-catalog-form-structure.ts";
import { hoverCardFamilyCatalog } from "./family-catalog-hover-card.ts";
import { formCompositeFamilyCatalog } from "./family-catalog-form-composites.ts";
import { i18nFamilyCatalog } from "./family-catalog-i18n.ts";
import { interactionGestureFamilyCatalog } from "../families/interaction/catalog/gestures.ts";
import { interactionSupportFamilyCatalog } from "../families/interaction/catalog/support.ts";
import { layoutFamilyCatalog } from "./family-catalog-layout.ts";
import { menuFamilyCatalog } from "./family-catalog-menus.ts";
import { navigationFamilyCatalog } from "./family-catalog-navigation.ts";
import { overlayFamilyCatalog } from "./family-catalog-overlays.ts";
import { ratingFamilyCatalog } from "./family-catalog-rating.ts";
import { audioPlayerFamilyCatalog } from "./family-catalog-audio-player.ts";
import { avatarGroupFamilyCatalog } from "./family-catalog-avatar-group.ts";
import { colorPickerFamilyCatalog } from "./family-catalog-color-picker.ts";
import { fileUploadFamilyCatalog } from "./family-catalog-file-upload.ts";
import { imageCropperFamilyCatalog } from "./family-catalog-image-cropper.ts";
import { infiniteScrollFamilyCatalog } from "./family-catalog-infinite-scroll.ts";
import { lightboxFamilyCatalog } from "./family-catalog-lightbox.ts";
import { marqueeFamilyCatalog } from "./family-catalog-marquee.ts";
import { mediaFamilyCatalog } from "./family-catalog-media.ts";
import { mediaPlayerFamilyCatalog } from "./family-catalog-media-player.ts";
import { qrCodeFamilyCatalog } from "./family-catalog-qr-code.ts";
import { signaturePadFamilyCatalog } from "./family-catalog-signature-pad.ts";
import { tourFamilyCatalog } from "./family-catalog-tour.ts";
import { videoPlayerFamilyCatalog } from "./family-catalog-video-player.ts";
import { selectionFamilyCatalog } from "./family-catalog-selection.ts";
import { sliderFamilyCatalog } from "./family-catalog-slider.ts";
import { structureFamilyCatalog } from "./family-catalog-structure.ts";
import { autocompleteFamilyCatalog } from "./family-catalog-autocomplete.ts";
import { listboxGridFamilyCatalog } from "./family-catalog-listbox-grid.ts";
import { mentionFamilyCatalog } from "./family-catalog-mention.ts";
import { transferListFamilyCatalog } from "./family-catalog-transfer-list.ts";
import { emojiPickerFamilyCatalog } from "./family-catalog-emoji-picker.ts";
import { cascaderFamilyCatalog } from "./family-catalog-cascader.ts";
import { toastFamilyCatalog } from "./family-catalog-toast.ts";
import { tagsInputFamilyCatalog } from "./family-catalog-tags-input.ts";
import { typographyFamilyCatalog } from "./family-catalog-typography.ts";
import { wayfindingFamilyCatalog } from "./family-catalog-wayfinding.ts";
import type { UiFamilyCatalogEntry } from "./family-catalog-types.ts";

export {
  UI_FAMILY_CATALOG_SCHEMA_VERSION,
  type UiFamilyBundleBudget,
  type UiFamilyCatalogEntry,
  type UiFamilyMaturity,
  type UiFamilyQualityGate,
} from "./family-catalog-types.ts";

const allFamilyCatalogEntries = [
  ...accordionFamilyCatalog,
  ...popconfirmFamilyCatalog,
  ...notificationCenterFamilyCatalog,
  ...commandPaletteFamilyCatalog,
  ...floatingActionButtonFamilyCatalog,
  ...backToTopFamilyCatalog,
  ...sidebarFamilyCatalog,
  ...resizableFamilyCatalog,
  ...stickyFamilyCatalog,
  ...actionFamilyCatalog,
  ...accessibilityFamilyCatalog,
  ...basicFamilyCatalog,
  ...chartFamilyCatalog,
  ...confirmFamilyCatalog,
  ...dataFamilyCatalog,
  ...dataViewFamilyCatalog,
  ...dateTimeFamilyCatalog,
  ...drawerFamilyCatalog,
  ...feedbackFamilyCatalog,
  ...foundationFamilyCatalog,
  ...helperFamilyCatalog,
  ...focusFamilyCatalog,
  ...formInputFamilyCatalog,
  ...formStructureFamilyCatalog,
  ...hoverCardFamilyCatalog,
  ...formCompositeFamilyCatalog,
  ...i18nFamilyCatalog,
  ...interactionGestureFamilyCatalog,
  ...interactionSupportFamilyCatalog,
  ...layoutFamilyCatalog,
  ...menuFamilyCatalog,
  ...navigationFamilyCatalog,
  ...overlayFamilyCatalog,
  ...ratingFamilyCatalog,
  ...audioPlayerFamilyCatalog,
  ...avatarGroupFamilyCatalog,
  ...colorPickerFamilyCatalog,
  ...fileUploadFamilyCatalog,
  ...imageCropperFamilyCatalog,
  ...infiniteScrollFamilyCatalog,
  ...lightboxFamilyCatalog,
  ...marqueeFamilyCatalog,
  ...mediaFamilyCatalog,
  ...mediaPlayerFamilyCatalog,
  ...qrCodeFamilyCatalog,
  ...signaturePadFamilyCatalog,
  ...tourFamilyCatalog,
  ...videoPlayerFamilyCatalog,
  ...selectionFamilyCatalog,
  ...sliderFamilyCatalog,
  ...structureFamilyCatalog,
  ...autocompleteFamilyCatalog,
  ...listboxGridFamilyCatalog,
  ...mentionFamilyCatalog,
  ...transferListFamilyCatalog,
  ...emojiPickerFamilyCatalog,
  ...cascaderFamilyCatalog,
  ...toastFamilyCatalog,
  ...tagsInputFamilyCatalog,
  ...typographyFamilyCatalog,
  ...wayfindingFamilyCatalog,
] as const satisfies readonly UiFamilyCatalogEntry[];

// Lane modules group families thematically, so canonical order is restored
// here rather than by concatenation order.
export const uiFamilyCatalog = [...allFamilyCatalogEntries].sort((a, b) =>
  a.canonicalName < b.canonicalName ? -1 : a.canonicalName > b.canonicalName ? 1 : 0,
);

export type UiFamilyCatalog = typeof uiFamilyCatalog;
export type UiFamilyName = UiFamilyCatalog[number]["canonicalName"];
