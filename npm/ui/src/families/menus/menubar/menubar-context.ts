import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CollectionRegistry } from "../../foundations/collection/collection.ts";
import type { TypeaheadController } from "../../interaction/typeahead/typeahead.ts";
import type { MenuLevelContextValue } from "../menu/menu-context.ts";
import type { MenuDirection, MenuEntryFocus } from "../menu/menu-types.ts";

/** Movement targets across menubar triggers. */
export type MenubarMoveDirection = "first" | "last" | "next" | "previous";

/** Collection value registered by each MenubarTrigger. */
export interface MenubarMenuRecord {
  readonly level: MenuLevelContextValue;
}

/** Roving focus and open-menu state shared by the menubar. */
export interface MenubarContextValue {
  readonly value: ComputedRef<string | null>;
  readonly dir: ComputedRef<MenuDirection>;
  readonly loop: ComputedRef<boolean>;
  readonly registry: CollectionRegistry<string, MenubarMenuRecord>;
  readonly typeahead: TypeaheadController<string>;
  readonly setValue: (value: string | null, event: Event | null) => boolean;
  /** Switch the open menu to `key` without returning focus to the previous trigger. */
  readonly switchTo: (key: string, event: Event | null, entry: MenuEntryFocus) => boolean;
  /** Move the roving tab stop; when a menu is open, open the destination menu too. */
  readonly move: (
    from: string,
    direction: MenubarMoveDirection,
    event: Event | null,
    entry?: MenuEntryFocus,
  ) => boolean;
}

/** Identity of one MenubarMenu for its trigger. */
export interface MenubarMenuContextValue {
  readonly value: ComputedRef<string>;
}

export const menubarContext = createContext<MenubarContextValue>("Menubar");
export const menubarMenuContext = createContext<MenubarMenuContextValue>("MenubarMenu");
