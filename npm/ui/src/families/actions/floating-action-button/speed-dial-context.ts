import type { ComputedRef, ShallowRef } from "vue";

import type { CollectionRegistry } from "../../foundations/collection/collection.ts";
import type { CompositeNavigationController } from "../../foundations/composite-navigation/composite-navigation.ts";
import { createContext } from "../../foundations/context/context.ts";
import type {
  SpeedDialDirection,
  SpeedDialSelectEvent,
  SpeedDialState,
} from "./floating-action-button-types.ts";

/** Shared state and actions for the SpeedDial compound components. */
export interface SpeedDialContextValue {
  readonly id: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly direction: ComputedRef<SpeedDialDirection>;
  readonly state: ComputedRef<SpeedDialState>;
  readonly registry: CollectionRegistry<string, string>;
  readonly navigation: CompositeNavigationController<string>;
  readonly triggerElement: ShallowRef<HTMLButtonElement | null>;
  readonly contentElement: ShallowRef<HTMLDivElement | null>;
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;
  readonly openAndFocus: (event?: Event | null) => boolean;
  readonly close: (options?: { readonly focusTrigger?: boolean }, event?: Event | null) => boolean;
  readonly select: (value: string, event: Event) => SpeedDialSelectEvent;
}

export const speedDialContext = createContext<SpeedDialContextValue>("SpeedDial");
