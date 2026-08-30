/** Compile-only assertions for the public error-summary contract. */

import type { ComputedRef, ShallowRef } from "vue";

import { createErrorSummary, type ErrorSummaryField } from "./error-summary.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const field: ErrorSummaryField = { id: "email", message: "Enter a valid address" };
export const labelledField: ErrorSummaryField = { id: "email", label: "Email", message: "Bad" };
// @ts-expect-error every field requires a message.
export const missingMessage: ErrorSummaryField = { id: "email" };

export const controller = createErrorSummary({ fields: [field], autoFocus: false });

type _FieldsAreReadonly = Expect<
  Equal<typeof controller.fields, Readonly<ShallowRef<readonly ErrorSummaryField[]>>>
>;
type _HasErrorsIsComputed = Expect<Equal<typeof controller.hasErrors, ComputedRef<boolean>>>;
type _FocusFieldReturnsControl = Expect<
  Equal<ReturnType<typeof controller.focusField>, HTMLElement | null>
>;

// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.fields.value = [];
// @ts-expect-error field lists are readonly.
controller.fields.value.push(field);
// @ts-expect-error autoFocus must resolve to a boolean.
createErrorSummary({ autoFocus: "yes" });
