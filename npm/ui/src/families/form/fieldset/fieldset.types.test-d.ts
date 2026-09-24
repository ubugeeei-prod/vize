/** Compile-only assertions for the public Fieldset contract. */

import {
  Fieldset,
  FieldsetDescription,
  FieldsetErrorMessage,
  FieldsetLegend,
  type FieldsetSlotState,
  type FieldsetState,
} from "./fieldset.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const slot: FieldsetSlotState;

type _StateIsClosed = Expect<Equal<FieldsetState, "disabled" | "invalid" | "valid">>;
type _SlotMessage = Expect<Equal<typeof slot.errorMessage, string | undefined>>;

const rootProps: InstanceType<typeof Fieldset>["$props"] = {
  name: "address",
  errors: [{ name: "address", message: "Required", path: ["address"] }],
  hasDescription: true,
  "onInvalid-change": (invalid: boolean) => invalid,
};
const errorProps: InstanceType<typeof FieldsetErrorMessage>["$props"] = { forceMount: true };
const descriptionProps: InstanceType<typeof FieldsetDescription>["$props"] = { as: "div" };
const legendProps: InstanceType<typeof FieldsetLegend>["$props"] = {};

// @ts-expect-error errors need the normalized FormFieldError shape.
const invalidErrors: InstanceType<typeof Fieldset>["$props"] = { errors: [{ message: "x" }] };

void descriptionProps;
void errorProps;
void invalidErrors;
void legendProps;
void rootProps;
