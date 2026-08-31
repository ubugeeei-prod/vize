import { parsePlacement } from "./positioner-geometry.ts";
import type {
  AvailableSize,
  AvailableSizeInput,
  PositionerArrowStyle,
  PositionerStrategy,
  PositionerStyle,
} from "./positioner-types.ts";

/** Inline style for the floating host at a translated position. */
export function hostStyle(strategy: PositionerStrategy, x: number, y: number): PositionerStyle {
  return `position:${strategy};left:0px;top:0px;transform:translate(${String(x)}px, ${String(y)}px)`;
}

/** Inline style for the arrow along the facing edge. */
export function arrowStyle(arrowX: number | null, arrowY: number | null): PositionerArrowStyle {
  return `position:absolute;left:${String(arrowX ?? 0)}px;top:${String(arrowY ?? 0)}px`;
}

/**
 * Measure the space a floating element may occupy at a resolved placement.
 *
 * The main axis stops at the reference edge (minus the configured offset);
 * the cross axis spans the viewport. Both respect the collision padding and
 * never go negative, mirroring how flip and shift read the same viewport.
 */
export function computeAvailableSize(input: AvailableSizeInput): AvailableSize {
  const padding = input.collisionPadding ?? 0;
  const offset = input.offset ?? 0;
  const { reference, viewport } = input;
  const { side } = parsePlacement(input.placement);

  let width = viewport.width - padding * 2;
  let height = viewport.height - padding * 2;
  if (side === "top") {
    height = reference.y - offset - (viewport.y + padding);
  } else if (side === "bottom") {
    height = viewport.y + viewport.height - padding - (reference.y + reference.height + offset);
  } else if (side === "left") {
    width = reference.x - offset - (viewport.x + padding);
  } else {
    width = viewport.x + viewport.width - padding - (reference.x + reference.width + offset);
  }

  return {
    height: Math.max(0, height),
    width: Math.max(0, width),
  };
}

/**
 * Publish available space as host constraints and CSS custom properties.
 *
 * The returned fragment is appended to the floating host style when the
 * `size` strategy is enabled: `max-width`/`max-height` clamp the box while
 * `--vize-ui-positioner-available-width` and
 * `--vize-ui-positioner-available-height` stay consumable from consumer CSS.
 */
export function sizeStyle(size: AvailableSize): string {
  const width = String(size.width);
  const height = String(size.height);
  return (
    `;max-width:${width}px;max-height:${height}px` +
    `;--vize-ui-positioner-available-width:${width}px` +
    `;--vize-ui-positioner-available-height:${height}px`
  );
}
