/** Public props for the watermark overlay. */
export interface WatermarkProps {
  /**
   * Text repeated across the overlay. Markup is escaped.
   *
   * @default ""
   */
  readonly text?: string;

  /**
   * Opacity of the overlay, from 0 to 1. The slotted content stays opaque.
   *
   * @default 0.15
   */
  readonly opacity?: number;

  /**
   * Tile size in CSS pixels.
   *
   * @default 160
   */
  readonly gap?: number;

  /**
   * Clockwise rotation of each tile, in degrees.
   *
   * @default -24
   */
  readonly rotate?: number;
}
