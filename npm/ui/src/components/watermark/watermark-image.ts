/** Geometry of one repeated watermark tile. */
export interface WatermarkImageOptions {
  readonly text: string;
  readonly gap: number;
  readonly rotate: number;
}

/** Escape text that is interpolated into an SVG document. */
export function escapeXml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&apos;");
}

/**
 * Build a repeated SVG tile as a CSS image.
 *
 * The tile is a data URI so the overlay renders during SSR without measuring
 * the host or touching `window`.
 */
export function watermarkImage(options: WatermarkImageOptions): string {
  const center = options.gap / 2;
  const svg =
    `<svg xmlns="http://www.w3.org/2000/svg" width="${options.gap}" height="${options.gap}">` +
    `<text x="50%" y="50%" fill="currentColor" font-size="16" text-anchor="middle" ` +
    `dominant-baseline="middle" transform="rotate(${options.rotate} ${center} ${center})">` +
    `${escapeXml(options.text)}</text></svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}
