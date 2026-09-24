/** Compile-only assertions for the useForm ↔ ui Field bridge. */

import type { ComputedRef } from "vue";

import { useForm } from "./use-form.ts";
import {
  useFormErrorSummaryFields,
  useFormFieldProps,
  type FormErrorSummaryItem,
  type FormFieldControlProps,
  type FormFieldErrorEntry,
  type FormFieldModelProps,
} from "./use-form-field-props.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

/** Mirrors `FieldControlProps` from `@vizejs/ui/field-wiring` (kept in sync structurally). */
interface UiFieldControlProps {
  readonly id: string;
  readonly "aria-labelledby": string;
  readonly "aria-describedby": string | undefined;
  readonly "aria-errormessage": string | undefined;
  readonly "aria-invalid": "true" | undefined;
}

/** Mirrors `ErrorSummaryField` from `@vizejs/ui/error-summary`. */
interface UiErrorSummaryField {
  readonly id: string;
  readonly message: string;
  readonly label?: string;
}

/** Mirrors `FormFieldError` from `@vizejs/ui/form`. */
interface UiFormFieldError {
  readonly name: string;
  readonly message: string;
  readonly path: readonly PropertyKey[];
}

const form = useForm({
  initialValues: { age: 0, address: { city: "" }, tags: [] as string[] },
});
const city = useFormFieldProps(form, "address.city");
const age = useFormFieldProps(form, "age");
const tag = useFormFieldProps(form, "tags.0");
const summary = useFormErrorSummaryFields(form, {
  labels: { "address.city": "City" },
  order: ["age"],
});
declare const entry: FormFieldErrorEntry;

type _ControlPropsMatchUi = Expect<Equal<FormFieldControlProps, UiFieldControlProps>>;
type _SummaryMatchesUi = Expect<Equal<FormErrorSummaryItem, UiErrorSummaryField>>;
const _entryIsUiError: UiFormFieldError = entry;
type _CityModel = Expect<Equal<typeof city.modelProps, ComputedRef<FormFieldModelProps<string>>>>;
type _AgeModel = Expect<Equal<typeof age.modelProps.value.modelValue, number>>;
type _TagModel = Expect<Equal<typeof tag.modelProps.value.modelValue, string>>;
type _PathIsLiteral = Expect<Equal<typeof city.field.path, "address.city">>;
type _Summary = Expect<Equal<typeof summary, ComputedRef<readonly FormErrorSummaryItem[]>>>;

age.modelProps.value["onUpdate:modelValue"](42);

// @ts-expect-error unknown paths are rejected.
useFormFieldProps(form, "address.street");

// @ts-expect-error the model setter is typed by the path value.
age.modelProps.value["onUpdate:modelValue"]("42");

// @ts-expect-error labels are keyed by typed paths.
useFormErrorSummaryFields(form, { labels: { "address.street": "Street" } });

// @ts-expect-error visibility is closed.
useFormFieldProps(form, "age", { showErrors: "dirty" });

void _entryIsUiError;
