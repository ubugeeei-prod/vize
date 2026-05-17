/**
 * Shared post-render layout metrics.
 */

import type { LayoutResultNapi } from "@vizejs/fresco-native";

export interface DOMElement {
  id?: number;
  $el?: {
    id?: number;
  };
}

let lastRenderLayouts: LayoutResultNapi[] = [];

export function updateLastRenderLayouts(layouts: LayoutResultNapi[]) {
  lastRenderLayouts = layouts;
}

export function getLastRenderLayout(node: DOMElement | null | undefined): LayoutResultNapi | null {
  const id = node?.id ?? node?.$el?.id;
  if (id === undefined) return null;
  return lastRenderLayouts.find((layout) => layout.id === id) ?? null;
}

export function measureElement(node: DOMElement): { width: number; height: number } {
  const layout = getLastRenderLayout(node);
  return {
    width: layout?.width ?? 0,
    height: layout?.height ?? 0,
  };
}
