import type { Rect, VirtualElement } from "../../overlays/positioner/positioner.ts";

/**
 * Item-aligned positioning for Select popups.
 *
 * The positioner places floating content below a reference box. Item-aligned
 * mode feeds it a zero-height virtual reference placed so that, once the
 * listbox sits directly below that reference, the anchor option (the selected
 * option, or the first one) overlaps the trigger and is vertically centred on
 * it, like a native macOS select. Collision shifting still keeps the whole
 * listbox inside the viewport.
 */

const zeroRect: Rect = Object.freeze({ height: 0, width: 0, x: 0, y: 0 });

/** Compute the virtual reference rect for item-aligned placement. */
export function itemAlignedReferenceRect(
  trigger: Rect,
  content: Rect | null,
  anchor: Rect | null,
): Rect {
  if (content === null || anchor === null) {
    return { height: trigger.height, width: trigger.width, x: trigger.x, y: trigger.y };
  }
  const anchorOffset = anchor.y - content.y;
  const centring = (trigger.height - anchor.height) / 2;
  return {
    height: 0,
    width: trigger.width,
    x: trigger.x,
    y: trigger.y + centring - anchorOffset,
  };
}

function measure(element: Element | null): Rect | null {
  if (element === null) return null;
  const rect = element.getBoundingClientRect();
  return { height: rect.height, width: rect.width, x: rect.x, y: rect.y };
}

/** Create a live virtual reference for item-aligned placement. */
export function createItemAlignedReference(sources: {
  readonly trigger: () => Element | null;
  readonly content: () => Element | null;
  readonly anchor: () => Element | null;
}): VirtualElement {
  return {
    getBoundingClientRect: () =>
      itemAlignedReferenceRect(
        measure(sources.trigger()) ?? zeroRect,
        measure(sources.content()),
        measure(sources.anchor()),
      ),
  };
}
