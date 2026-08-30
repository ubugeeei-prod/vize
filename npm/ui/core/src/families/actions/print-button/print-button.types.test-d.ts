/** Compile-only assertions for the public PrintButton contract. */

import type { Component } from "vue";

import { PrintButton } from "./print-button.ts";
import type {
  PrintButtonAction,
  PrintButtonCssCustomProperty,
  PrintButtonDataAttribute,
  PrintButtonDataName,
  PrintButtonElement,
  PrintButtonEmits,
  PrintButtonExpose,
  PrintButtonPart,
  PrintButtonProps,
  PrintButtonSlotState,
  PrintButtonSlots,
  PrintButtonState,
  PrintButtonType,
} from "./print-button.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: PrintButtonExpose;
declare const slotState: PrintButtonSlotState;

type ComponentPrintButtonProps = InstanceType<typeof PrintButton>["$props"];

type _TypeIsLiteral = Expect<Equal<PrintButtonType, "button" | "reset" | "submit">>;
type _StateIsLiteral = Expect<Equal<PrintButtonState, "idle" | "printing" | "printed" | "error">>;
type _PartNamesAreClosed = Expect<Equal<PrintButtonPart, "label" | "root">>;
type _DataNamesAreClosed = Expect<
  Equal<PrintButtonDataName, "print-button" | "print-button-label">
>;
type _DataAttributesAreClosed = Expect<
  Equal<PrintButtonDataAttribute, "data-disabled" | "data-printing" | "data-state" | "data-vize-ui">
>;
type _CssCustomPropertiesAreClosed = Expect<Equal<PrintButtonCssCustomProperty, never>>;
type _PropsKeysAreClosed = Expect<
  Equal<
    keyof PrintButtonProps,
    | "action"
    | "ariaDescribedby"
    | "ariaLabel"
    | "ariaLabelledby"
    | "as"
    | "disabled"
    | "errorLabel"
    | "idleLabel"
    | "native"
    | "printedLabel"
    | "printingLabel"
    | "type"
  >
>;
type _EmitsAreClosed = Expect<Equal<keyof PrintButtonEmits, "print" | "error">>;
type _SlotReceivesState = Expect<
  Equal<Parameters<PrintButtonSlots["default"]>[0], PrintButtonSlotState>
>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slotState,
    {
      readonly disabled: boolean;
      readonly printing: boolean;
      readonly unavailable: boolean;
      readonly state: PrintButtonState;
      readonly label: string;
    }
  >
>;
type _ExposeStateIsLiteral = Expect<Equal<typeof exposed.state, PrintButtonState>>;
type _ExposeElementIsNullable = Expect<Equal<typeof exposed.element, PrintButtonElement | null>>;
type _ActionIsNarrow = Expect<
  Equal<PrintButtonAction, (nativeEvent: MouseEvent) => void | Promise<void>>
>;
type _PropsFeedComponentProps = Expect<
  PrintButtonProps extends InstanceType<typeof PrintButton>["$props"] ? true : false
>;
type _PropsAllowNoAction = Expect<Equal<{} extends PrintButtonProps ? true : false, true>>;

const action: PrintButtonAction = async (event) => {
  const nativeEvent: MouseEvent = event;
  void nativeEvent;
};
const props = {
  action,
  ariaDescribedby: "print-help",
  ariaLabel: "Print invoice",
  ariaLabelledby: "print-label",
  as: componentTarget,
  disabled: false,
  errorLabel: "Print failed",
  idleLabel: "Print",
  native: false,
  printedLabel: "Printed invoice",
  printingLabel: "Printing invoice",
  type: "button",
} satisfies PrintButtonProps;
const componentProps: ComponentPrintButtonProps = {
  ...props,
  onPrint: (event: MouseEvent) => {
    void event;
  },
  onError: (error: unknown, event: MouseEvent) => {
    void error;
    void event;
  },
};

const element: PrintButtonExpose["element"] = exposed.element;
const state: PrintButtonState = exposed.state;
const label: string = exposed.label;
const unavailable: boolean = exposed.unavailable;

exposed.focus();

// @ts-expect-error native button type is limited to platform submit modes.
const badType: PrintButtonProps = { type: "menu" };

// @ts-expect-error state is a closed lifecycle contract.
const badState: PrintButtonState = "loading";

// @ts-expect-error action receives exactly the triggering MouseEvent.
const badAction: PrintButtonAction = (event: KeyboardEvent) => {
  void event;
};

void PrintButton;
void badAction;
void badState;
void badType;
void componentProps;
void element;
void label;
void props;
void state;
void unavailable;
