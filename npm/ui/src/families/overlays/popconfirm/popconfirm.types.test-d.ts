/** Compile-only assertions for the public Popconfirm contract. */

import type {
  PopconfirmButtonExpose,
  PopconfirmCancelReason,
  PopconfirmConfirmHandler,
  PopconfirmContentExpose,
  PopconfirmInitialFocus,
  PopconfirmRootExpose,
  PopconfirmSlotState,
  PopconfirmState,
} from "./popconfirm.ts";
import {
  Popconfirm,
  PopconfirmCancel,
  PopconfirmConfirm,
  PopconfirmContent,
  PopconfirmRoot,
  PopconfirmTrigger,
} from "./popconfirm.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: PopconfirmRootExpose;
declare const content: PopconfirmContentExpose;
declare const action: PopconfirmButtonExpose;
declare const slot: PopconfirmSlotState;

type _State = Expect<Equal<PopconfirmState, "idle" | "pending">>;
type _Focus = Expect<Equal<PopconfirmInitialFocus, "cancel" | "confirm" | "none">>;
type _Reason = Expect<Equal<PopconfirmCancelReason, "cancel-button" | "dismiss" | "programmatic">>;
type _Handler = Expect<Equal<ReturnType<PopconfirmConfirmHandler>, void | PromiseLike<void>>>;
type _Confirm = Expect<Equal<ReturnType<typeof root.confirm>, Promise<boolean>>>;
type _Element = Expect<Equal<typeof content.element, HTMLDivElement | null>>;
type _Button = Expect<Equal<typeof action.element, HTMLButtonElement | null>>;
type _Pending = Expect<Equal<typeof slot.pending, boolean>>;

const rootProps: InstanceType<typeof PopconfirmRoot>["$props"] = {
  defaultOpen: false,
  onConfirm: async () => {
    await Promise.resolve();
  },
  "onUpdate:open": (value: boolean) => value,
};
const syncRoot: InstanceType<typeof PopconfirmRoot>["$props"] = { onConfirm: () => undefined };
const contentProps: InstanceType<typeof PopconfirmContent>["$props"] = {
  description: "Cannot be undone",
  initialFocus: "confirm",
  placement: "bottom-end",
  title: "Delete?",
};
const triggerProps: InstanceType<typeof PopconfirmTrigger>["$props"] = { ariaLabel: "Delete" };
const confirmProps: InstanceType<typeof PopconfirmConfirm>["$props"] = { disabled: false };
const cancelProps: InstanceType<typeof PopconfirmCancel>["$props"] = { disabled: false };

// @ts-expect-error confirmation handlers resolve to nothing.
const badHandler: InstanceType<typeof PopconfirmRoot>["$props"] = { onConfirm: () => 42 };

// @ts-expect-error initial focus is a closed union.
const badFocus: InstanceType<typeof PopconfirmContent>["$props"] = { initialFocus: "title" };

// @ts-expect-error state is a closed union.
const badState: PopconfirmState = "done";

void Popconfirm;
void badFocus;
void badHandler;
void badState;
void cancelProps;
void confirmProps;
void contentProps;
void rootProps;
void syncRoot;
void triggerProps;
