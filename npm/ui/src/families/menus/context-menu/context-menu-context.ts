import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { MenuEntryFocus } from "../menu/menu-types.ts";
import type { ContextMenuPoint } from "./context-menu-types.ts";

/** Anchor contract shared by ContextMenuRoot and ContextMenuTrigger. */
export interface ContextMenuContextValue {
  readonly disabled: ComputedRef<boolean>;
  /** Anchor the menu at a viewport point and open it. */
  readonly openAt: (point: ContextMenuPoint, event: Event | null, entry: MenuEntryFocus) => boolean;
}

export const contextMenuContext = createContext<ContextMenuContextValue>("ContextMenu");
