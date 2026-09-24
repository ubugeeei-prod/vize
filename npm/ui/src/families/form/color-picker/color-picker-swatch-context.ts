import type { ComputedRef } from "vue";

import type { CollectionRegistry } from "../../foundations/collection/collection.ts";
import type { CompositeNavigationController } from "../../foundations/composite-navigation/composite-navigation.ts";
import { createContext } from "../../foundations/context/context.ts";

/** Shared roving-focus state for ColorPickerSwatchGroup and its swatches. */
export interface ColorPickerSwatchGroupContextValue {
  readonly registry: CollectionRegistry<string, string>;
  readonly navigation: CompositeNavigationController<string>;
  readonly disabled: ComputedRef<boolean>;
  readonly syncActiveValue: () => void;
  readonly getSwatchId: (value: string) => string;
}

export const colorPickerSwatchGroupContext =
  createContext<ColorPickerSwatchGroupContextValue>("ColorPickerSwatchGroup");

const safeSegment = /^[A-Za-z0-9][A-Za-z0-9_-]*$/;

/** Create a deterministic, DOM-id-safe segment for an arbitrary swatch value such as `#ff0000`. */
export function getSwatchIdSegment(value: string): string {
  if (safeSegment.test(value)) return `swatch-${value}`;
  const readable = value
    .replaceAll(/[^A-Za-z0-9_-]+/g, "-")
    .replaceAll(/^-+|-+$/g, "")
    .slice(0, 32);
  let hash = 0x811c9dc5;
  for (let index = 0; index < value.length; index++) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return `swatch-${readable || "empty"}-${hash.toString(36)}`;
}
