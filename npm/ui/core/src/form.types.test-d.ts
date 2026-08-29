/** Compile-only assertions for the public form foundation contract. */

import type { ComputedRef } from "vue";

import {
  normalizeStandardSchemaResult,
  useFormField,
  validateStandardSchema,
  type FormFieldController,
  type FormFieldError,
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

export const validated = validateStandardSchema(schema, { email: "me@example.com" });
type _ValidateKeepsOutput = Expect<
  Equal<typeof validated, Promise<FormValidationResult<SignInOutput>>>
>;

// @ts-expect-error schema input is enforced when validating.
void validateStandardSchema(schema, { email: 1 });

export const normalized = normalizeStandardSchemaResult<SignInOutput>({
  value: { email: "me@example.com", normalized: true },
});
type _NormalizeKeepsOutput = Expect<Equal<typeof normalized, FormValidationResult<SignInOutput>>>;

export const fieldError: FormFieldError = {
  message: "Enter an email",
  name: "email",
  path: ["email"],
};

export const field = useFormField({ errors: [fieldError], name: "email" });
type _FieldIsComputed = Expect<
  Equal<FormFieldController["errors"], ComputedRef<readonly FormFieldError[]>>
>;
type _InvalidIsComputed = Expect<Equal<typeof field.isInvalid, ComputedRef<boolean>>>;
type _MessageIsComputed = Expect<Equal<typeof field.errorMessage, ComputedRef<string | undefined>>>;

// @ts-expect-error field names must resolve to strings.
useFormField({ name: 1 });
// @ts-expect-error consumers cannot mutate computed errors.
field.errors.value = [];
// @ts-expect-error normalized field errors are readonly.
field.errors.value.push(fieldError);
