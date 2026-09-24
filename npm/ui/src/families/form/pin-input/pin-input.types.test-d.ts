/** Compile-only assertions for the public PinInput contract. */

import {
  PinInput,
  PinInputField,
  isPinCharacters,
  type PinInputCharacters,
  type PinInputExpose,
  type PinInputState,
  type PinInputType,
} from "./pin-input.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type RootProps<Length extends number> = Parameters<typeof PinInput<Length>>[0];

/** Infers the length literal exactly as a template usage would. */
declare function inferLength<Length extends number>(props: RootProps<Length>): Length;

declare const control: PinInputExpose;
declare const characters: readonly string[];

const inferredSix = inferLength({ length: 6 });
const inferredRuntime = inferLength({ length: characters.length });

type _InfersLiteralLength = Expect<Equal<typeof inferredSix, 6>>;
type _InfersRuntimeLength = Expect<Equal<typeof inferredRuntime, number>>;
type _FourTuple = Expect<Equal<PinInputCharacters<4>, readonly [string, string, string, string]>>;
type _RuntimeLengthIsArray = Expect<Equal<PinInputCharacters<number>, readonly string[]>>;
type _NegativeLengthIsArray = Expect<Equal<PinInputCharacters<-1>, readonly string[]>>;
type _TypeIsClosed = Expect<Equal<PinInputType, "alphanumeric" | "numeric">>;
type _StateIsClosed = Expect<
  Equal<PinInputState, "complete" | "disabled" | "empty" | "incomplete">
>;
type _ExposeValue = Expect<Equal<typeof control.value, string>>;

const sixDigits: RootProps<6> = {
  length: 6,
  onComplete: (
    value: string,
    digits: readonly [string, string, string, string, string, string],
  ) => {
    void value;
    void digits;
  },
};
const fieldProps: InstanceType<typeof PinInputField>["$props"] = { index: 0 };

if (isPinCharacters(characters, 3)) {
  const [first, second, third] = characters;
  type _Narrowed = Expect<Equal<typeof characters, readonly [string, string, string]>>;
  void first;
  void second;
  void third;
}

control.setValue("123456");
control.focus(2);

const fourDigits = (_: string, digits: readonly [string, string, string, string]) => digits;
// @ts-expect-error a six-field completion is not a four-tuple.
const wrongTuple: RootProps<6> = { length: 6, onComplete: fourDigits };

// @ts-expect-error length is required.
const missingLength: RootProps<4> = {};

// @ts-expect-error fields need an index.
const missingIndex: InstanceType<typeof PinInputField>["$props"] = {};

void fieldProps;
void missingIndex;
void missingLength;
void sixDigits;
void wrongTuple;
