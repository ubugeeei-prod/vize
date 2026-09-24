/** Compile-only assertions for the public input-mask contract. */

import {
  MaskedInput,
  createInputMask,
  defineInputMaskTokens,
  useInputMask,
  type InputMask,
  type InputMaskController,
  type InputMaskDefaultTokenKey,
  type InputMaskResult,
  type InputMaskValueFormat,
  type MaskedInputEmits,
  type MaskedInputExpose,
  type MaskedInputProps,
  type MaskedInputState,
} from "./input-mask.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const controller: InputMaskController;
declare const control: MaskedInputExpose;

const hex = defineInputMaskTokens({
  H: { pattern: /[0-9a-f]/i, transform: (character: string) => character.toUpperCase() },
});
const mask: InputMask = createInputMask("#HHHHHH", { tokens: hex });

type _TokenKeysAreLiteral = Expect<Equal<keyof typeof hex, "H">>;
type _DefaultTokenKeys = Expect<Equal<InputMaskDefaultTokenKey, "9" | "a" | "*">>;
type _FormatIsClosed = Expect<Equal<InputMaskValueFormat, "masked" | "raw">>;
type _StateIsClosed = Expect<
  Equal<MaskedInputState, "complete" | "disabled" | "empty" | "incomplete" | "readonly">
>;
type _ResultShape = Expect<Equal<ReturnType<InputMask["conform"]>, InputMaskResult>>;
type _UpdatePayload = Expect<Equal<MaskedInputEmits["update:modelValue"], [value: string]>>;
type _CompletePayload = Expect<Equal<MaskedInputEmits["complete"], [result: InputMaskResult]>>;
type _ControllerInputMode = Expect<Equal<typeof controller.inputMode.value, "numeric" | "text">>;
type _ExposeRaw = Expect<Equal<typeof control.raw, string>>;

const props = {
  eager: true,
  lazy: false,
  mask: "(999) 999-9999",
  placeholderChar: "_",
  tokens: hex,
  valueFormat: "masked",
} satisfies MaskedInputProps;
const componentProps: InstanceType<typeof MaskedInput>["$props"] = {
  mask: "9999",
  modelValue: "12",
  onComplete: (result: InputMaskResult) => result.raw,
  "onUpdate:modelValue": (value: string) => value,
};

useInputMask({ mask: () => "99", valueFormat: "raw" });

// @ts-expect-error the mask is required.
const missingMask: InstanceType<typeof MaskedInput>["$props"] = {};

// @ts-expect-error value formats are closed.
const invalidFormat = { mask: "99", valueFormat: "digits" } satisfies MaskedInputProps;

// @ts-expect-error tokens need a RegExp pattern.
defineInputMaskTokens({ X: { pattern: "x" } });

void componentProps;
void invalidFormat;
void mask;
void missingMask;
void props;
