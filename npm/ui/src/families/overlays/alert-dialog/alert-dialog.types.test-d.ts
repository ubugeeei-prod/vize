/** Compile-only assertions for the public AlertDialog contract. */

import type {
  AlertDialogActionExpose,
  AlertDialogCancelExpose,
  AlertDialogContentExpose,
  AlertDialogRootExpose,
  AlertDialogSlotState,
  AlertDialogState,
} from "./alert-dialog.ts";
import {
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogRoot,
} from "./alert-dialog.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: AlertDialogRootExpose;
declare const content: AlertDialogContentExpose;
declare const action: AlertDialogActionExpose;
declare const cancel: AlertDialogCancelExpose;

type _StateIsDialogState = Expect<Equal<AlertDialogState, "closed" | "open">>;
type _SlotStateIsExact = Expect<
  Equal<
    AlertDialogSlotState,
    {
      readonly open: boolean;
      readonly modal: boolean;
      readonly state: AlertDialogState;
    }
  >
>;
type _RootExposesDialogIds = Expect<Equal<typeof root.contentId, string>>;
type _ContentElementIsNative = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _ActionAndCancelShareCloseExpose = Expect<Equal<typeof action.focus, typeof cancel.focus>>;

const rootProps: InstanceType<typeof AlertDialogRoot>["$props"] = {
  defaultOpen: true,
  id: "confirm",
};
const contentProps: InstanceType<typeof AlertDialogContent>["$props"] = {
  closeOnEscape: true,
  closeOnFocusOutside: false,
  closeOnPointerDownOutside: false,
};
const actionProps: InstanceType<typeof AlertDialogAction>["$props"] = { type: "submit" };
const cancelProps: InstanceType<typeof AlertDialogCancel>["$props"] = { type: "button" };

// @ts-expect-error AlertDialogContent fixes the announced role to alertdialog.
const roleProps: InstanceType<typeof AlertDialogContent>["$props"] = { role: "dialog" };

const badContentProps: InstanceType<typeof AlertDialogContent>["$props"] = {
  // @ts-expect-error outside pointer dismissal is a boolean opt-in.
  closeOnPointerDownOutside: "yes",
};

void actionProps;
void badContentProps;
void cancelProps;
void content;
void contentProps;
void roleProps;
void rootProps;
