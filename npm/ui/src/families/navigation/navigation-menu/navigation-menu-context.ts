import type { ComputedRef, ShallowRef } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { createContext } from "../../foundations/context/context.ts";
import type {
  NavigationMenuChangeReason,
  NavigationMenuDirection,
  NavigationMenuMotion,
  NavigationMenuOrientation,
  NavigationMenuValue,
} from "./navigation-menu-types.ts";

/** Shared state and actions for the NavigationMenu compound parts. */
export interface NavigationMenuContextValue {
  readonly value: ComputedRef<NavigationMenuValue>;
  readonly orientation: ComputedRef<NavigationMenuOrientation>;
  readonly dir: ComputedRef<NavigationMenuDirection>;
  readonly getTriggerId: (value: string) => string;
  readonly getContentId: (value: string) => string;
  readonly getMotion: (value: string) => NavigationMenuMotion | null;
  readonly registerItem: (input: {
    readonly value: string;
    readonly element: Readonly<ShallowRef<HTMLLIElement | null>>;
  }) => CollectionRegistration<string>;
  readonly registerTrigger: (
    value: string,
    element: Readonly<ShallowRef<HTMLButtonElement | null>>,
  ) => () => void;
  readonly registerContent: (
    value: string,
    element: Readonly<ShallowRef<HTMLDivElement | null>>,
  ) => () => void;
  readonly getTriggerElement: (value: NavigationMenuValue) => HTMLButtonElement | null;
  readonly getContentElement: (value: NavigationMenuValue) => HTMLDivElement | null;
  readonly setOpen: (value: NavigationMenuValue, reason: NavigationMenuChangeReason) => boolean;
  readonly toggle: (value: string) => void;
  readonly onTriggerEnter: (value: string) => void;
  readonly onPointerLeave: () => void;
  readonly onContentEnter: () => void;
  readonly focusContent: (value: string) => void;
}

export const navigationMenuContext = createContext<NavigationMenuContextValue>("NavigationMenu");

/** Item-level state shared with the trigger, content, and nested links. */
export interface NavigationMenuItemContextValue {
  readonly value: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
}

export const navigationMenuItemContext =
  createContext<NavigationMenuItemContextValue>("NavigationMenuItem");

/** Marker provided by NavigationMenuContent so nested links close the flyout on select. */
export interface NavigationMenuContentContextValue {
  readonly value: ComputedRef<string>;
}

export const navigationMenuContentContext =
  createContext<NavigationMenuContentContextValue>("NavigationMenuContent");
