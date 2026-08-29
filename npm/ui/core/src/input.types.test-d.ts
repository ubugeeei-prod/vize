/** Compile-only assertions for the public input contract. */

import type {
  InputAriaInvalid,
  InputEnterKeyHint,
  InputExpose,
  InputInputMode,
  InputType,
} from "./input.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const input: InputExpose;

type _ValueIsString = Expect<Equal<typeof input.value, string>>;
type _CompositionIsBoolean = Expect<Equal<typeof input.composing, boolean>>;
type _TypeIsTextLike = Expect<
  Equal<InputType, "email" | "password" | "search" | "tel" | "text" | "url">
>;
type _InputModeIsLiteral = Expect<
  Equal<
    InputInputMode,
    "decimal" | "email" | "none" | "numeric" | "search" | "tel" | "text" | "url"
  >
>;
type _EnterKeyHintIsLiteral = Expect<
  Equal<InputEnterKeyHint, "done" | "enter" | "go" | "next" | "previous" | "search" | "send">
>;
type _InvalidStateIsNative = Expect<Equal<InputAriaInvalid, boolean | "grammar" | "spelling">>;

input.setValue("Ada");
input.reset();
input.focus();
input.select();

// @ts-expect-error input values are strings.
input.setValue(1);

// @ts-expect-error number inputs are a separate value/parser family.
const numberType: InputType = "number";

void numberType;
