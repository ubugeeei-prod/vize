export { escapeXml, watermarkImage } from "./watermark-image.ts";
export type { WatermarkImageOptions } from "./watermark-image.ts";
export type { WatermarkProps } from "./types.ts";

/** SSR-safe repeated text overlay that never intercepts pointer events. */
export { default as Watermark } from "./watermark.vue";
