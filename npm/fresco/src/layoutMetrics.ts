/**
 * Shared post-render layout metrics.
 */

import type { LayoutResultNapi } from "@vizejs/fresco-native";

/** Renderer element or component public instance accepted by layout helpers. */
export interface DOMElement {
  id?: number;
  $el?: {
    id?: number;
  };
}

let lastRenderLayouts: LayoutResultNapi[] = [];

/** Replace the layout snapshot exposed by the most recent native render. */
export function updateLastRenderLayouts(layouts: LayoutResultNapi[]) {
  lastRenderLayouts = layouts;
}

/** Look up the latest native layout for an element or component public instance. */
export function getLastRenderLayout(node: DOMElement | null | undefined): LayoutResultNapi | null {
  const id = node?.id ?? node?.$el?.id;
  if (id === undefined) return null;
  return lastRenderLayouts.find((layout) => layout.id === id) ?? null;
}

/** Return the latest element dimensions, falling back to zero before layout. */
export function measureElement(node: DOMElement): { width: number; height: number } {
  const layout = getLastRenderLayout(node);
  return {
    width: layout?.width ?? 0,
    height: layout?.height ?? 0,
  };
}
