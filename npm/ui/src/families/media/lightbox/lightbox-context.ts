import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  LightboxChangeReason,
  LightboxDirection,
  LightboxMessages,
  LightboxPartSlotState,
  LightboxState,
} from "./lightbox-types.ts";

/** Shared state and actions for the Lightbox compound parts. */
export interface LightboxContextValue {
  readonly id: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<LightboxState>;
  readonly index: ComputedRef<number>;
  readonly count: ComputedRef<number>;
  readonly loop: ComputedRef<boolean>;
  readonly dir: ComputedRef<LightboxDirection>;
  readonly closeOnSwipeDown: ComputedRef<boolean>;
  readonly canGoPrevious: ComputedRef<boolean>;
  readonly canGoNext: ComputedRef<boolean>;
  readonly messages: ComputedRef<LightboxMessages>;
  readonly slotState: ComputedRef<LightboxPartSlotState>;
  readonly getItemId: (index: number) => string;
  readonly goTo: (index: number, reason: LightboxChangeReason) => boolean;
  readonly step: (delta: 1 | -1, reason: LightboxChangeReason) => boolean;
  readonly openAt: (index: number) => boolean;
  readonly close: () => boolean;
  readonly registerThumbnail: (index: number, element: HTMLButtonElement) => () => void;
  readonly focusThumbnail: (index: number) => void;
}

export const lightboxContext = createContext<LightboxContextValue>("Lightbox");
