/** Compile-only assertions for the public FullscreenButton contract. */

import type { Component } from "vue";

import { FullscreenButton } from "./fullscreen-button.ts";
import type {
  FullscreenButtonController,
  FullscreenButtonCssCustomProperty,
  FullscreenButtonDataAttribute,
  FullscreenButtonDataName,
  FullscreenButtonElement,
  FullscreenButtonEmits,
  FullscreenButtonExpose,
  FullscreenButtonOperation,
  FullscreenButtonOperationType,
  FullscreenButtonPart,
  FullscreenButtonProps,
  FullscreenButtonSlotState,
  FullscreenButtonSlots,
  FullscreenButtonState,
  FullscreenButtonTarget,
  FullscreenButtonType,
} from "./fullscreen-button.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const elementTarget: Element;
declare const exposed: FullscreenButtonExpose;
declare const slotState: FullscreenButtonSlotState;

type ComponentFullscreenButtonProps = InstanceType<typeof FullscreenButton>["$props"];

type _TypeIsLiteral = Expect<Equal<FullscreenButtonType, "button" | "reset" | "submit">>;
type _StateIsLiteral = Expect<
  Equal<FullscreenButtonState, "idle" | "entering" | "active" | "exiting" | "error">
>;
type _OperationTypeIsLiteral = Expect<Equal<FullscreenButtonOperationType, "enter" | "exit">>;
type _PartNamesAreClosed = Expect<Equal<FullscreenButtonPart, "label" | "root">>;
type _DataNamesAreClosed = Expect<
  Equal<FullscreenButtonDataName, "fullscreen-button" | "fullscreen-button-label">
>;
type _DataAttributesAreClosed = Expect<
  Equal<
    FullscreenButtonDataAttribute,
    "data-active" | "data-disabled" | "data-pending" | "data-state" | "data-vize-ui"
  >
>;
type _CssCustomPropertiesAreClosed = Expect<Equal<FullscreenButtonCssCustomProperty, never>>;
type _PropsKeysAreClosed = Expect<
  Equal<
    keyof FullscreenButtonProps,
    | "ariaDescribedby"
    | "ariaLabel"
    | "ariaLabelledby"
    | "as"
    | "busyLabel"
    | "controller"
    | "disabled"
    | "enterLabel"
    | "errorLabel"
    | "exitLabel"
    | "native"
    | "target"
    | "type"
  >
>;
type _ControllerKeysAreClosed = Expect<
  Equal<
    keyof FullscreenButtonController,
    "exitFullscreen" | "getFullscreenElement" | "requestFullscreen"
  >
>;
type _OperationKeysAreClosed = Expect<
  Equal<keyof FullscreenButtonOperation, "controller" | "target" | "type">
>;
type _EmitsAreClosed = Expect<Equal<keyof FullscreenButtonEmits, "fullscreen" | "error">>;
type _SlotReceivesState = Expect<
  Equal<Parameters<FullscreenButtonSlots["default"]>[0], FullscreenButtonSlotState>
>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slotState,
    {
      readonly disabled: boolean;
      readonly active: boolean;
      readonly pending: boolean;
      readonly operation: FullscreenButtonOperationType | null;
      readonly unavailable: boolean;
      readonly state: FullscreenButtonState;
      readonly label: string;
    }
  >
>;
type _ExposeStateIsLiteral = Expect<Equal<typeof exposed.state, FullscreenButtonState>>;
type _ExposeElementIsNullable = Expect<
  Equal<typeof exposed.element, FullscreenButtonElement | null>
>;
type _TargetIsNarrow = Expect<
  Equal<
    FullscreenButtonTarget,
    Element | null | undefined | ((nativeEvent: MouseEvent) => Element | null | undefined)
  >
>;
type _PropsFeedComponentProps = Expect<
  FullscreenButtonProps extends InstanceType<typeof FullscreenButton>["$props"] ? true : false
>;
type _PropsAllowNoController = Expect<Equal<{} extends FullscreenButtonProps ? true : false, true>>;

const controller: FullscreenButtonController = {
  getFullscreenElement: () => null,
  requestFullscreen: async (target, event) => {
    const nextTarget: Element = target;
    const nativeEvent: MouseEvent = event;
    void nextTarget;
    void nativeEvent;
  },
  exitFullscreen: async (event) => {
    const nativeEvent: MouseEvent = event;
    void nativeEvent;
  },
};
const targetResolver: FullscreenButtonTarget = (event) => {
  const nativeEvent: MouseEvent = event;
  void nativeEvent;
  return elementTarget;
};
const props = {
  ariaDescribedby: "fullscreen-help",
  ariaLabel: "Toggle fullscreen",
  ariaLabelledby: "fullscreen-label",
  as: componentTarget,
  busyLabel: "Changing fullscreen",
  controller,
  disabled: false,
  enterLabel: "Enter fullscreen",
  errorLabel: "Fullscreen failed",
  exitLabel: "Exit fullscreen",
  native: false,
  target: targetResolver,
  type: "button",
} satisfies FullscreenButtonProps;
const componentProps: ComponentFullscreenButtonProps = {
  ...props,
  onFullscreen: (operation: FullscreenButtonOperation, event: MouseEvent) => {
    void operation;
    void event;
  },
  onError: (error: unknown, operation: FullscreenButtonOperation, event: MouseEvent) => {
    void error;
    void operation;
    void event;
  },
};

const element: FullscreenButtonExpose["element"] = exposed.element;
const state: FullscreenButtonState = exposed.state;
const label: string = exposed.label;
const active: boolean = exposed.active;
const pending: boolean = exposed.pending;
const operation: FullscreenButtonOperationType | null = exposed.operation;
const unavailable: boolean = exposed.unavailable;

exposed.focus();

// @ts-expect-error native button type is limited to platform submit modes.
const badType: FullscreenButtonProps = { type: "menu" };

// @ts-expect-error state is a closed lifecycle contract.
const badState: FullscreenButtonState = "loading";

// @ts-expect-error operation is a closed toggle contract.
const badOperationType: FullscreenButtonOperationType = "toggle";

// @ts-expect-error fullscreen targets must be Elements or event resolvers.
const badTarget: FullscreenButtonTarget = () => 1;

const badController: FullscreenButtonController = {
  // @ts-expect-error requestFullscreen receives the target and triggering MouseEvent.
  requestFullscreen: (event: KeyboardEvent) => {
    void event;
  },
  exitFullscreen: () => {},
};

void FullscreenButton;
void active;
void badController;
void badOperationType;
void badState;
void badTarget;
void badType;
void componentProps;
void element;
void label;
void operation;
void pending;
void props;
void state;
void unavailable;
