/** Compile-only assertions for the public CopyButton contract. */

import type { Component } from "vue";

import { CopyButton } from "./copy-button.ts";
import type {
  CopyButtonCssCustomProperty,
  CopyButtonDataAttribute,
  CopyButtonDataName,
  CopyButtonElement,
  CopyButtonEmits,
  CopyButtonExpose,
  CopyButtonPart,
  CopyButtonProps,
  CopyButtonSlotState,
  CopyButtonSlots,
  CopyButtonState,
  CopyButtonType,
  CopyButtonWriter,
} from "./copy-button.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: CopyButtonExpose;
declare const slotState: CopyButtonSlotState;

type ComponentCopyButtonProps = InstanceType<typeof CopyButton>["$props"];

type _TypeIsLiteral = Expect<Equal<CopyButtonType, "button" | "reset" | "submit">>;
type _StateIsLiteral = Expect<Equal<CopyButtonState, "idle" | "copied" | "error">>;
type _PartNamesAreClosed = Expect<Equal<CopyButtonPart, "label" | "root">>;
type _DataNamesAreClosed = Expect<Equal<CopyButtonDataName, "copy-button" | "copy-button-label">>;
type _DataAttributesAreClosed = Expect<
  Equal<CopyButtonDataAttribute, "data-disabled" | "data-state" | "data-vize-ui" | "data-writing">
>;
type _CssCustomPropertiesAreClosed = Expect<Equal<CopyButtonCssCustomProperty, never>>;
type _PropsKeysAreClosed = Expect<
  Equal<
    keyof CopyButtonProps,
    | "ariaDescribedby"
    | "ariaLabel"
    | "ariaLabelledby"
    | "as"
    | "copiedLabel"
    | "disabled"
    | "errorLabel"
    | "idleLabel"
    | "native"
    | "type"
    | "value"
    | "writer"
  >
>;
type _EmitsAreClosed = Expect<Equal<keyof CopyButtonEmits, "copy" | "error">>;
type _SlotReceivesState = Expect<
  Equal<Parameters<CopyButtonSlots["default"]>[0], CopyButtonSlotState>
>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slotState,
    {
      readonly value: string;
      readonly disabled: boolean;
      readonly writing: boolean;
      readonly unavailable: boolean;
      readonly state: CopyButtonState;
      readonly label: string;
    }
  >
>;
type _ExposeStateIsLiteral = Expect<Equal<typeof exposed.state, CopyButtonState>>;
type _ExposeElementIsNullable = Expect<Equal<typeof exposed.element, CopyButtonElement | null>>;
type _WriterIsNarrow = Expect<Equal<CopyButtonWriter, (value: string) => void | Promise<void>>>;
type _PropsFeedComponentProps = Expect<
  CopyButtonProps extends InstanceType<typeof CopyButton>["$props"] ? true : false
>;
type _PropsRequireValue = Expect<Equal<{} extends CopyButtonProps ? true : false, false>>;

const writer: CopyButtonWriter = async (value) => {
  const copied: string = value;
  void copied;
};
const props = {
  ariaDescribedby: "copy-help",
  ariaLabel: "Copy invite",
  ariaLabelledby: "copy-label",
  as: componentTarget,
  copiedLabel: "Copied invite",
  disabled: false,
  errorLabel: "Copy failed",
  idleLabel: "Copy",
  native: false,
  type: "button",
  value: "https://vize.dev/invite",
  writer,
} satisfies CopyButtonProps;
const componentProps: ComponentCopyButtonProps = {
  ...props,
  onCopy: (value: string, event: MouseEvent) => {
    void value;
    void event;
  },
  onError: (error: unknown, value: string, event: MouseEvent) => {
    void error;
    void value;
    void event;
  },
};

const element: CopyButtonExpose["element"] = exposed.element;
const state: CopyButtonState = exposed.state;
const label: string = exposed.label;
const unavailable: boolean = exposed.unavailable;

exposed.focus();

// @ts-expect-error CopyButton requires a string value.
const missingValue: CopyButtonProps = {};

// @ts-expect-error copied values are always strings.
const badValue: CopyButtonProps = { value: 1 };

// @ts-expect-error native button type is limited to platform submit modes.
const badType: CopyButtonProps = { type: "menu", value: "copy" };

// @ts-expect-error state is a closed result contract.
const badState: CopyButtonState = "loading";

// @ts-expect-error writer receives exactly the string value.
const badWriter: CopyButtonWriter = (value: number) => {
  void value;
};

void CopyButton;
void badState;
void badType;
void badValue;
void badWriter;
void componentProps;
void element;
void label;
void missingValue;
void props;
void state;
void unavailable;
