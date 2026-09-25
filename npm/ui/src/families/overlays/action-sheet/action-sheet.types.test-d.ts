/** Compile-only assertions for the public ActionSheet contract. */

import {
  ActionSheet,
  ActionSheetCancel,
  ActionSheetItem,
  ActionSheetMenu,
  type ActionSheetSelectEvent,
} from "./action-sheet.ts";
import { DrawerClose, DrawerRoot } from "../drawer/drawer.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _RootIsDrawer = Expect<Equal<typeof ActionSheet, typeof DrawerRoot>>;
type _CancelIsClose = Expect<Equal<typeof ActionSheetCancel, typeof DrawerClose>>;
type _SelectValue = Expect<Equal<ActionSheetSelectEvent["value"], string>>;

const itemProps: InstanceType<typeof ActionSheetItem>["$props"] = {
  value: "delete",
  destructive: true,
  onSelect: (event: ActionSheetSelectEvent) => event.preventDefault(),
};
const menuProps: InstanceType<typeof ActionSheetMenu>["$props"] = { loop: false };

// @ts-expect-error actions need a value.
const missingValue: InstanceType<typeof ActionSheetItem>["$props"] = {};

void itemProps;
void menuProps;
void missingValue;
