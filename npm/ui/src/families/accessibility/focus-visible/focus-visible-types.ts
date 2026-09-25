import type { ComputedRef, ShallowRef } from "vue";

import type { InteractionModality } from "../interaction-modality/interaction-modality.ts";

/** Input modality mirrored to `data-vize-modality`. */
export type FocusVisibleModality = InteractionModality;

/** Reactive focus-visible state returned by {@link useFocusVisible}. */
export interface FocusVisibleState {
  /**
   * Whether focus should be indicated: inside a provider, whether a descendant
   * currently carries the focus-visible attribute; standalone, whether the last
   * interaction was keyboard or virtual.
   */
  readonly isFocusVisible: ComputedRef<boolean>;

  /** Latest document interaction modality, or `null` before any input. */
  readonly modality: Readonly<ShallowRef<FocusVisibleModality | null>>;
}

/** State exposed to the FocusVisibleProvider slot. */
export interface FocusVisibleSlotState {
  /** Whether a descendant currently shows a focus indicator. */
  readonly isFocusVisible: boolean;

  /** Latest interaction modality, or `null` before mount and before any input. */
  readonly modality: FocusVisibleModality | null;
}

/** Public instance exposed by FocusVisibleProvider. */
export interface FocusVisibleProviderExpose extends FocusVisibleSlotState {
  /** Rendered provider element. */
  readonly element: HTMLDivElement | null;

  /** Descendant that currently carries the focus-visible attribute. */
  readonly focusVisibleElement: Element | null;
}
