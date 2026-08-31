/** Compile-only assertions for the public Dialog contract. */

import type {
  DialogContentExpose,
  DialogRole,
  DialogRootExpose,
  DialogSlotState,
  DialogState,
} from "./dialog.ts";
import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  DialogTrigger,
} from "./dialog.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: DialogRootExpose;
declare const content: DialogContentExpose;

type _StateIsLiteral = Expect<Equal<DialogState, "closed" | "open">>;
type _RoleIsLiteral = Expect<Equal<DialogRole, "alertdialog" | "dialog">>;
type _RootOpenIsBoolean = Expect<Equal<typeof root.open, boolean>>;
type _ContentElementIsDiv = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    DialogSlotState,
    {
      readonly open: boolean;
      readonly modal: boolean;
      readonly state: DialogState;
    }
  >
>;

const rootProps: InstanceType<typeof DialogRoot>["$props"] = {
  defaultOpen: true,
  id: "settings",
  modal: true,
  open: false,
  "onUpdate:open": (value: boolean) => value,
};
const contentProps: InstanceType<typeof DialogContent>["$props"] = {
  ariaDescribedby: null,
  ariaLabel: "Settings",
  closeOnEscape: true,
  forceMount: true,
  role: "dialog",
};
const slotState: DialogSlotState = { modal: true, open: true, state: "open" };

// @ts-expect-error Dialog state has a closed token contract.
const invalidState: DialogState = "opening";

// @ts-expect-error Dialog role is limited to WAI-ARIA dialog roles.
const invalidRole: DialogRole = "menu";

// @ts-expect-error open is boolean-only.
const badRootProps: InstanceType<typeof DialogRoot>["$props"] = { open: "true" };

// @ts-expect-error content role must be a DialogRole.
const badContentProps: InstanceType<typeof DialogContent>["$props"] = { role: "status" };

void DialogClose;
void DialogDescription;
void DialogOverlay;
void DialogPortal;
void DialogTitle;
void DialogTrigger;
void badContentProps;
void badRootProps;
void contentProps;
void invalidRole;
void invalidState;
void rootProps;
void slotState;
