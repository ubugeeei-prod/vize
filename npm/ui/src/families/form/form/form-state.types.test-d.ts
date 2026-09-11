/** Compile-only assertions for reactive form state. */

import type { ComputedRef, Ref } from "vue";

import {
  normalizeNativeConstraintErrors,
  useFormState,
  type FormFieldState,
  type FormStateController,
  type FormValidationResult,
  type StandardSchemaV1,
} from "./form.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface SignInInput {
  readonly email: string;
}

interface SignInOutput extends SignInInput {
  readonly normalized: true;
}

declare const schema: StandardSchemaV1<SignInInput, SignInOutput>;

export const form = useFormState({
  initialErrors: [{ message: "Bad", name: "email", path: ["email"] }],
  schema,
});
export const field = form.fieldState("email");
export const validation = form.validate({ email: "me@example.com" });
export const submission = form.submit({ email: "me@example.com" });
export const nativeErrors = normalizeNativeConstraintErrors(document.createElement("form"));

type _ControllerInfersSchema = Expect<
  Equal<typeof form, FormStateController<SignInInput, SignInOutput>>
>;
type _ErrorsAreReadonlyRef = Expect<
  Equal<FormStateController["errors"], Readonly<Ref<readonly import("./form.ts").FormFieldError[]>>>
>;
type _FieldStateIsComputed = Expect<Equal<typeof field, ComputedRef<FormFieldState>>>;
type _ValidationKeepsOutput = Expect<
  Equal<typeof validation, Promise<FormValidationResult<SignInOutput>>>
>;
type _SubmissionKeepsOutput = Expect<
  Equal<typeof submission, Promise<FormValidationResult<SignInOutput>>>
>;

form.markFieldDirty("email");
form.markFieldTouched("email", false);
form.markFieldVisited("email");
form.setServerErrors(nativeErrors);
form.reset({ dirtyFields: ["email"], touchedFields: [], visitedFields: [] });

// @ts-expect-error schema input is enforced when validating.
void form.validate({ email: 1 });
// @ts-expect-error schema input is enforced when submitting.
void form.submit({ email: 1 });
// @ts-expect-error field names must resolve to strings.
form.fieldState(1);
// @ts-expect-error reset field lists must be string arrays.
form.reset({ dirtyFields: [1] });
// @ts-expect-error the error ref is readonly to consumers.
form.errors.value = [];
// @ts-expect-error field state is readonly to consumers.
field.value.isDirty = false;
