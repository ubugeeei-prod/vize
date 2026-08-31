/** Accessible, unstyled page and section banner primitive. */
export { default as Banner } from "./banner.vue";

export { normalizeBannerAria } from "./banner-aria.ts";
export type { BannerAriaInput, NormalizedBannerAria } from "./banner-aria.ts";

export type {
  BannerAriaState,
  BannerElement,
  BannerEmits,
  BannerExpose,
  BannerLive,
  BannerProps,
  BannerRole,
  BannerSlots,
  BannerSlotState,
  BannerState,
  BannerTone,
} from "./banner-types.ts";
