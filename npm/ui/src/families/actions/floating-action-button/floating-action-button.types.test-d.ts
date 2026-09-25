/** Compile-only assertions for the public FloatingActionButton and SpeedDial contract. */

import type {
  FloatingActionButtonPlacement,
  SpeedDialActionSlotState,
  SpeedDialChangeReason,
  SpeedDialDirection,
  SpeedDialRootExpose,
  SpeedDialSelectEvent,
  SpeedDialState,
} from "./floating-action-button.ts";
import {
  FloatingActionButton,
  SpeedDial,
  SpeedDialAction,
  SpeedDialContent,
  SpeedDialRoot,
  SpeedDialTrigger,
} from "./floating-action-button.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: SpeedDialRootExpose;
declare const actionSlot: SpeedDialActionSlotState;
declare const selection: SpeedDialSelectEvent;

type _Direction = Expect<Equal<SpeedDialDirection, "down" | "left" | "right" | "up">>;
type _State = Expect<Equal<SpeedDialState, "closed" | "open">>;
type _Placement = Expect<
  Equal<
    FloatingActionButtonPlacement,
    "bottom-center" | "bottom-end" | "bottom-start" | "top-center" | "top-end" | "top-start"
  >
>;
type _Reason = Expect<
  Equal<
    SpeedDialChangeReason,
    "action" | "escape" | "hover" | "keyboard" | "outside" | "pointer" | "programmatic"
  >
>;
type _SelectionValue = Expect<Equal<typeof selection.value, string>>;
type _ActionActive = Expect<Equal<typeof actionSlot.active, boolean>>;
type _Close = Expect<Equal<ReturnType<typeof root.close>, boolean>>;

root.openAndFocus();
root.close({ focusTrigger: true });
selection.preventDefault();

const fabProps: InstanceType<typeof FloatingActionButton>["$props"] = {
  ariaLabel: "Compose",
  extended: true,
  placement: "top-center",
};
const rootProps: InstanceType<typeof SpeedDialRoot>["$props"] = {
  closeOnSelect: false,
  direction: "left",
  openOnHover: true,
  onSelect: (event: SpeedDialSelectEvent) => event.value,
};
const actionProps: InstanceType<typeof SpeedDialAction>["$props"] = {
  label: "Copy",
  value: "copy",
};
const triggerProps: InstanceType<typeof SpeedDialTrigger>["$props"] = { ariaLabel: "Share" };
const contentProps: InstanceType<typeof SpeedDialContent>["$props"] = {
  ariaLabel: "Share actions",
};

// @ts-expect-error actions require an accessible label.
const missingLabel: InstanceType<typeof SpeedDialAction>["$props"] = { value: "copy" };

// @ts-expect-error direction is a closed union.
const badDirection: InstanceType<typeof SpeedDialRoot>["$props"] = { direction: "diagonal" };

// @ts-expect-error placement is a closed union.
const badPlacement: InstanceType<typeof FloatingActionButton>["$props"] = { placement: "center" };

void SpeedDial;
void actionProps;
void badDirection;
void badPlacement;
void contentProps;
void fabProps;
void missingLabel;
void rootProps;
void triggerProps;
