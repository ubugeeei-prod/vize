/** Compile-only assertions for the public Field composition contract. */

import type { FieldErrorMessageExpose, FieldRootExpose, FieldRootSlotState } from "./field.ts";
import { Field, FieldDescription, FieldErrorMessage, FieldLabel, FieldRoot } from "./field.ts";
import type { FieldControlProps } from "./field-wiring.ts";
import type { FormFieldError } from "./form.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: FieldRootExpose;
declare const error: FieldErrorMessageExpose;

const errors: readonly FormFieldError[] = [
  { message: "Enter an email", name: "email", path: ["email"] },
];

const rootProps: InstanceType<typeof Field>["$props"] = {
  as: "section",
  errors,
  hasDescription: true,
  hasErrorMessage: true,
  id: "email",
  invalid: false,
  name: "email",
  "onInvalid-change": (invalid: boolean, nextErrors: readonly FormFieldError[]) => {
    void invalid;
    void nextErrors;
  },
};
const aliasProps: InstanceType<typeof FieldRoot>["$props"] = rootProps;
const labelProps: InstanceType<typeof FieldLabel>["$props"] = { as: "label" };
const descriptionProps: InstanceType<typeof FieldDescription>["$props"] = { as: "p" };
const errorMessageProps: InstanceType<typeof FieldErrorMessage>["$props"] = {
  as: "strong",
  forceMount: true,
};

type _SlotIncludesFieldProps = Expect<Equal<FieldRootSlotState["fieldProps"], FieldControlProps>>;
type _RootIdIsString = Expect<Equal<typeof root.id, string>>;
type _RootErrorsAreReadonly = Expect<Equal<typeof root.errors, readonly FormFieldError[]>>;
type _ErrorMessageIsOptional = Expect<Equal<typeof error.message, string | undefined>>;
type _ErrorInvalidIsBoolean = Expect<Equal<typeof error.invalid, boolean>>;

// @ts-expect-error invalid is boolean-only.
const invalidProps: InstanceType<typeof Field>["$props"] = { invalid: "true", name: "email" };

const badErrors: InstanceType<typeof Field>["$props"] = {
  errors: [
    // @ts-expect-error errors must be normalized FormFieldError objects.
    { message: "Bad" },
  ],
  name: "email",
};

void aliasProps;
void badErrors;
void descriptionProps;
void errorMessageProps;
void invalidProps;
void labelProps;
void rootProps;
