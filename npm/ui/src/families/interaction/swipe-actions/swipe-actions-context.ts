import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { SwipeActionsOpen, SwipeActionsSide } from "./swipe-actions-types.ts";

/** Shared state for SwipeActions parts. */
export interface SwipeActionsContextValue {
  readonly open: ComputedRef<SwipeActionsOpen>;
  readonly offset: ComputedRef<number>;
  readonly disabled: ComputedRef<boolean>;
  readonly contentId: ComputedRef<string>;
  readonly registerTray: (side: SwipeActionsSide, element: HTMLElement | null) => void;
  readonly registerContent: (element: HTMLElement | null) => void;
  readonly onPointerdown: (event: PointerEvent) => void;
  readonly onContentKeydown: (event: KeyboardEvent) => void;
  readonly openSide: (side: SwipeActionsSide, focus?: boolean) => boolean;
  readonly close: () => boolean;
}

/** Typed context shared by SwipeActions and its parts. */
export const swipeActionsContext = createContext<SwipeActionsContextValue>("SwipeActions");
