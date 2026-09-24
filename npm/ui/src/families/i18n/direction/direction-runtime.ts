import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { Direction } from "./direction-types.ts";

/**
 * Writing-direction context. `DirectionProvider` and `LocaleProvider` publish it;
 * direction-aware families read it through {@link useResolvedDirection}.
 */
export const directionContext = createContext<ComputedRef<Direction>>("Direction");

/** Normalize an arbitrary `dir` value to a supported direction, or `undefined`. */
export function toDirection(value: unknown): Direction | undefined {
  return value === "ltr" || value === "rtl" ? value : undefined;
}

/**
 * Resolve a component's writing direction.
 *
 * Precedence: the component's own `dir` prop, then the nearest provided direction,
 * then `"ltr"`. The document's `dir` attribute is intentionally not read, so server
 * and client resolve identically and hydration stays stable.
 */
export function useResolvedDirection(
  local?: MaybeRefOrGetter<Direction | null | undefined>,
): ComputedRef<Direction> {
  const provided = directionContext.useOptional();
  return computed(() => toDirection(toValue(local)) ?? provided?.value ?? "ltr");
}
