/** Compile-only assertions for the public field-wiring contract. */

import type { ComputedRef } from "vue";

import type {
  FieldControlProps,
  FieldWiringController,
  FieldWiringOptions,
} from "./field-wiring.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const options: FieldWiringOptions = {
  hasDescription: true,
  hasErrorMessage: () => true,
  id: "billing-email",
  invalid: false,
};

// @ts-expect-error invalid must resolve to a boolean.
export const invalidOptions: FieldWiringOptions = { invalid: "yes" };

declare const controller: FieldWiringController;

type _FieldPropsAreComputed = Expect<
  Equal<typeof controller.fieldProps, ComputedRef<FieldControlProps>>
>;
type _AriaInvalidIsClosed = Expect<Equal<FieldControlProps["aria-invalid"], "true" | undefined>>;
type _IsInvalidIsComputed = Expect<Equal<typeof controller.isInvalid, ComputedRef<boolean>>>;

// @ts-expect-error consumers cannot mutate computed wiring.
controller.isInvalid.value = true;
// @ts-expect-error wiring props are readonly.
controller.fieldProps.value.id = "other";
