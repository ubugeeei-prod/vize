import type { ComputedRef, ShallowRef } from "vue";

import type {
  CollectionRegistration,
  CollectionRegistry,
} from "../../foundations/collection/collection.ts";
import type { CommandInfo } from "../../foundations/command/command.ts";
import type { CompositeNavigationController } from "../../foundations/composite-navigation/composite-navigation.ts";
import { createContext } from "../../foundations/context/context.ts";
import type { CommandPaletteState } from "./command-palette-types.ts";

/** Reactive item data registered with the palette root. */
export interface CommandPaletteItemEntry {
  readonly id: string;
  readonly textValue: () => string;
  readonly keywords: () => readonly string[];
  readonly disabled: () => boolean;
  readonly forceMount: () => boolean;
  readonly element: Readonly<ShallowRef<HTMLDivElement | null>>;
  readonly select: (event: Event | null) => boolean;
}

/**
 * Shared state for the CommandPalette compound components.
 *
 * Item values cross a provide/inject boundary, so the context stores
 * type-erased entries; CommandPaletteItem keeps its own typed value.
 */
export interface CommandPaletteContextValue {
  readonly id: ComputedRef<string>;
  readonly listId: ComputedRef<string>;
  readonly inputId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<CommandPaletteState>;
  readonly search: ComputedRef<string>;
  readonly loading: ComputedRef<boolean>;
  readonly resultCount: ComputedRef<number>;
  readonly activeItemId: ComputedRef<string | null>;
  readonly commands: ComputedRef<readonly CommandInfo<string>[]>;
  readonly findCommand: (id: string) => CommandInfo<string> | undefined;
  readonly runCommand: (id: string) => boolean;
  readonly registry: CollectionRegistry<string, CommandPaletteItemEntry>;
  readonly navigation: CompositeNavigationController<string>;
  readonly inputElement: ShallowRef<HTMLInputElement | null>;
  readonly isVisible: (id: string) => boolean;
  readonly registerItem: (entry: CommandPaletteItemEntry) => CollectionRegistration<string>;
  readonly setSearch: (value: string) => boolean;
  readonly setOpen: (value: boolean) => boolean;
  readonly setActive: (id: string) => void;
  readonly selectActive: (event: Event | null) => boolean;
  readonly didSelect: (commandId: string | null, event: Event | null) => void;
}

export const commandPaletteContext = createContext<CommandPaletteContextValue>("CommandPalette");

/** Membership registry published by CommandPaletteGroup. */
export interface CommandPaletteGroupContextValue {
  readonly headingId: ComputedRef<string>;
  readonly addMember: (id: string) => () => void;
}

export const commandPaletteGroupContext =
  createContext<CommandPaletteGroupContextValue>("CommandPaletteGroup");

/** Hook published by CommandPaletteDialog so a nested root can close it after selection. */
export interface CommandPaletteDialogContextValue {
  readonly didSelect: () => void;
}

export const commandPaletteDialogContext =
  createContext<CommandPaletteDialogContextValue>("CommandPaletteDialog");
