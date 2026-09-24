/** Compile-only assertions for the public CommandPalette contract. */

import { createCommandRouter } from "../../foundations/command/command.ts";
import type { CommandInfo } from "../../foundations/command/command.ts";
import type {
  CommandPaletteDialogExpose,
  CommandPaletteFilter,
  CommandPaletteItemExpose,
  CommandPaletteItemSlotState,
  CommandPaletteItemState,
  CommandPaletteResultsLabel,
  CommandPaletteRootExpose,
  CommandPaletteSlotState,
  CommandPaletteState,
} from "./command-palette.ts";
import {
  CommandPalette,
  CommandPaletteDialog,
  CommandPaletteEmpty,
  CommandPaletteGroup,
  CommandPaletteInput,
  CommandPaletteItem,
  CommandPaletteList,
  CommandPaletteLoading,
  CommandPaletteRoot,
  defaultCommandPaletteFilter,
  defaultCommandPaletteResultsLabel,
} from "./command-palette.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: CommandPaletteRootExpose;
declare const item: CommandPaletteItemExpose;
declare const dialog: CommandPaletteDialogExpose;
declare const slot: CommandPaletteSlotState<"reload" | "theme">;
declare const itemSlot: CommandPaletteItemSlotState;

type _State = Expect<Equal<CommandPaletteState, "closed" | "open">>;
type _ItemState = Expect<Equal<CommandPaletteItemState, "active" | "disabled" | "inactive">>;
type _Filter = Expect<
  Equal<CommandPaletteFilter, (text: string, search: string, keywords: readonly string[]) => number>
>;
type _Label = Expect<Equal<CommandPaletteResultsLabel, (count: number, search: string) => string>>;
type _SlotCommands = Expect<
  Equal<typeof slot.commands, readonly CommandInfo<"reload" | "theme">[]>
>;
type _Shortcut = Expect<Equal<typeof itemSlot.shortcut, string | null>>;
type _ActiveId = Expect<Equal<typeof root.activeItemId, string | null>>;
type _ItemElement = Expect<Equal<typeof item.element, HTMLDivElement | null>>;

const filter: CommandPaletteFilter = defaultCommandPaletteFilter;
const label: string = defaultCommandPaletteResultsLabel(2);
root.setSearch("reload");
root.selectActive(new KeyboardEvent("keydown"));
dialog.toggle();
item.select();

const router = createCommandRouter<"reload" | "theme">();
// Generic root: the router's id union flows into recent ids and select payloads.
CommandPaletteRoot({
  router,
  defaultRecent: ["reload"],
  onSelect: (commandId) => {
    type _CommandId = Expect<Equal<typeof commandId, "reload" | "theme" | null>>;
  },
});
// Generic item: `value` types the select payload.
CommandPaletteItem({
  value: { slug: "open" },
  textValue: "Open",
  onSelect: (value) => {
    type _Value = Expect<Equal<typeof value, { slug: string } | undefined>>;
  },
});

// @ts-expect-error recent ids must be registered command ids.
CommandPaletteRoot({ router, defaultRecent: ["deploy"] });

// @ts-expect-error filters return numeric scores.
const badFilter: CommandPaletteFilter = () => true;

const inputProps: InstanceType<typeof CommandPaletteInput>["$props"] = {
  ariaLabel: "Search",
  autofocus: true,
  placeholder: "Type",
};
const groupProps: InstanceType<typeof CommandPaletteGroup>["$props"] = { heading: "Files" };
const dialogProps: InstanceType<typeof CommandPaletteDialog>["$props"] = {
  closeOnSelect: false,
  shortcut: null,
};

// @ts-expect-error groups require a heading.
const badGroup: InstanceType<typeof CommandPaletteGroup>["$props"] = {};

// @ts-expect-error the shortcut is a string or null.
const badDialog: InstanceType<typeof CommandPaletteDialog>["$props"] = { shortcut: 1 };

void CommandPalette;
void CommandPaletteEmpty;
void CommandPaletteList;
void CommandPaletteLoading;
void badDialog;
void badFilter;
void badGroup;
void dialogProps;
void filter;
void groupProps;
void inputProps;
void label;
