/** Compile-only assertions for the public textarea contract. */

import type { TextareaAriaInvalid, TextareaExpose, TextareaWrap } from "./textarea.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const textarea: TextareaExpose;

type _ValueIsString = Expect<Equal<typeof textarea.value, string>>;
type _CompositionIsBoolean = Expect<Equal<typeof textarea.composing, boolean>>;
type _WrapIsLiteral = Expect<Equal<TextareaWrap, "hard" | "off" | "soft">>;
type _InvalidStateIsNative = Expect<Equal<TextareaAriaInvalid, boolean | "grammar" | "spelling">>;

textarea.setValue("Ada");
textarea.setSelectionRange(0, 1);
textarea.setSelectionRange(0, 1, "forward");
textarea.reset();
textarea.focus();
textarea.select();

// @ts-expect-error textarea values are strings.
textarea.setValue(1);

// @ts-expect-error native textarea wrap is a closed literal contract.
const badWrap: TextareaWrap = "balance";

void badWrap;
