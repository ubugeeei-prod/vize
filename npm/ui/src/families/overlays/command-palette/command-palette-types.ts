import type { CommandInfo } from "../../foundations/command/command.ts";

/** Whether the palette results are showing, mirrored to `data-state`. */
export type CommandPaletteState = "closed" | "open";

/** Availability of one palette item, mirrored to `data-state`. */
export type CommandPaletteItemState = "active" | "disabled" | "inactive";

/**
 * Score an item against the current search.
 *
 * Return `0` to hide the item; any positive number keeps it visible. Higher
 * scores are reserved for consumer ranking and do not reorder authored items.
 */
export type CommandPaletteFilter = (
  text: string,
  search: string,
  keywords: readonly string[],
) => number;

/** Build the live-region announcement for the number of visible results. */
export type CommandPaletteResultsLabel = (count: number, search: string) => string;

/** State exposed to CommandPaletteRoot slots. */
export interface CommandPaletteSlotState<Id extends string = string> {
  /** Whether results are showing. */
  readonly open: boolean;

  /** Current search text. */
  readonly search: string;

  /** Number of visible, registered items. */
  readonly resultCount: number;

  /** Whether results are loading. */
  readonly loading: boolean;

  /** Router commands matching the search, in registration order. */
  readonly commands: readonly CommandInfo<Id>[];

  /** Router commands listed in `recent`, most recent first. */
  readonly recentCommands: readonly CommandInfo<Id>[];
}

/** State exposed to CommandPaletteItem slots. */
export interface CommandPaletteItemSlotState {
  /** Whether this item is the active descendant. */
  readonly active: boolean;

  /** Whether selection is refused. */
  readonly disabled: boolean;

  /** Stable state token for styling and tests. */
  readonly state: CommandPaletteItemState;

  /** Searchable label of the item. */
  readonly textValue: string;

  /** Keyboard shortcut advertised through `aria-keyshortcuts`, or `null`. */
  readonly shortcut: string | null;
}

/** State exposed to CommandPaletteGroup slots. */
export interface CommandPaletteGroupSlotState {
  /** Group heading. */
  readonly heading: string;

  /** Number of visible items in the group. */
  readonly resultCount: number;
}

/** Public instance exposed by CommandPaletteRoot. */
export interface CommandPaletteRootExpose {
  /** Root-owned base id. */
  readonly id: string;

  /** Id of the rendered listbox. */
  readonly listId: string;

  /** Whether results are showing. */
  readonly open: boolean;

  /** Current search text. */
  readonly search: string;

  /** Number of visible items. */
  readonly resultCount: number;

  /** Id of the active item, or `null`. */
  readonly activeItemId: string | null;

  /** Replace the search text and report whether it changed. */
  readonly setSearch: (value: string) => boolean;

  /** Request a specific open value and report whether it changed. */
  readonly setOpen: (value: boolean) => boolean;

  /** Select the active item as if Enter was pressed. Returns whether an item was selected. */
  readonly selectActive: (event?: Event | null) => boolean;

  /** Focus the search input. */
  readonly focusInput: (options?: FocusOptions) => void;
}

/** Public instance exposed by CommandPaletteInput. */
export interface CommandPaletteInputExpose {
  /** Rendered input element. */
  readonly element: HTMLInputElement | null;

  /** Move focus to the input. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by CommandPaletteList. */
export interface CommandPaletteListExpose {
  /** Rendered listbox element. */
  readonly element: HTMLDivElement | null;
}

/** Public instance exposed by CommandPaletteItem. */
export interface CommandPaletteItemExpose extends CommandPaletteItemSlotState {
  /** Stable item id used by `aria-activedescendant`. */
  readonly id: string;

  /** Whether the current search keeps the item visible. */
  readonly visible: boolean;

  /** Rendered option element. */
  readonly element: HTMLDivElement | null;

  /** Select this item. Returns whether selection ran. */
  readonly select: (event?: Event | null) => boolean;
}

/** Public instance exposed by CommandPaletteDialog. */
export interface CommandPaletteDialogExpose {
  /** Whether the dialog is open. */
  readonly open: boolean;

  /** Request a specific open value and report whether it changed. */
  readonly setOpen: (value: boolean) => boolean;

  /** Toggle the dialog. */
  readonly toggle: () => boolean;
}
