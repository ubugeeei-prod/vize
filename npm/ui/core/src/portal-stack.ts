import { shallowReadonly, shallowRef } from "vue";
import type { InjectionKey, ShallowRef } from "vue";

/** One mounted portal layer tracked by the shared nested-portal stack. */
export interface PortalStackEntry {
  /** Number of portal ancestors above this layer. Root portals report `0`. */
  readonly depth: number;
  /** Portalled node that hosts this layer's content. */
  readonly element: HTMLElement;
}

/**
 * Injection key carrying the nesting depth of the closest portal ancestor.
 *
 * `Portal` provides `depth + 1` for its subtree, so nesting survives
 * relocation: injection follows the component tree, not the document.
 */
export const portalDepthKey: InjectionKey<number> = Symbol("PortalDepth");

let sequence = 0;
const order = new Map<PortalStackEntry, number>();
const entries = shallowRef<readonly PortalStackEntry[]>(Object.freeze([]));

function sortedEntries(): readonly PortalStackEntry[] {
  return Object.freeze(
    [...order.keys()].sort((left, right) => {
      if (left.depth !== right.depth) return left.depth - right.depth;
      return (order.get(left) ?? 0) - (order.get(right) ?? 0);
    }),
  );
}

/**
 * Track one mounted portal layer and return its release callback.
 *
 * Registration happens client-side only (`Portal` registers on mount), so the
 * stack never accumulates request-global state during server rendering.
 */
export function registerPortalLayer(entry: PortalStackEntry): () => void {
  sequence += 1;
  order.set(entry, sequence);
  entries.value = sortedEntries();
  return () => {
    if (!order.delete(entry)) return;
    entries.value = sortedEntries();
  };
}

/**
 * Live, shared list of mounted portal layers.
 *
 * Entries are ordered shallow-to-deep (ties resolve by registration), so the
 * last entry is the layer that should sit on top.
 */
export function usePortalStack(): Readonly<ShallowRef<readonly PortalStackEntry[]>> {
  return shallowReadonly(entries);
}

/** The deepest (visually topmost) mounted portal layer, if any. */
export function topPortalLayer(): PortalStackEntry | null {
  return entries.value.at(-1) ?? null;
}
