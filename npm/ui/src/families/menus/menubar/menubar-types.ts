import type { MenuDirection, MenuSlotState } from "../menu/menu-types.ts";

/** State exposed to MenubarRoot slots. */
export interface MenubarSlotState {
  /** Value of the open menu, or `null` when every menu is closed. */
  readonly value: string | null;

  /** Resolved reading direction. */
  readonly dir: MenuDirection;
}

/** State exposed to MenubarMenu and MenubarTrigger slots. */
export interface MenubarMenuSlotState extends MenuSlotState {
  /** Stable value identifying this menu within the menubar. */
  readonly value: string;
}

/** State exposed to MenubarTrigger slots. */
export interface MenubarTriggerSlotState extends MenubarMenuSlotState {
  /** Whether the trigger owns the menubar's roving tab stop / highlight. */
  readonly highlighted: boolean;

  /** Whether the trigger is disabled. */
  readonly disabled: boolean;
}

/** Imperative surface exposed by MenubarRoot. */
export interface MenubarRootExpose extends MenubarSlotState {
  /** Rendered `role="menubar"` element. */
  readonly element: HTMLDivElement | null;

  /** Open the menu with `value` (or close every menu with `null`). */
  readonly setValue: (value: string | null) => boolean;

  /** Move focus to the first enabled trigger. */
  readonly focus: () => void;
}

/** Imperative surface exposed by MenubarMenu. */
export interface MenubarMenuExpose {
  /** Stable value identifying this menu. */
  readonly value: string;

  /** Whether this menu is open. */
  readonly open: boolean;

  /** Open this menu (entry focus on the first item) or close it. */
  readonly setOpen: (value: boolean) => boolean;
}

/** Imperative surface exposed by MenubarTrigger. */
export interface MenubarTriggerExpose {
  /** Rendered `role="menuitem"` trigger. */
  readonly element: HTMLButtonElement | null;

  /** Move focus (and the roving tab stop) to this trigger. */
  readonly focus: () => void;
}
