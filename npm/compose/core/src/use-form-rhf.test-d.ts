/** Compile-only assertions for the React Hook Form-class controls on `useForm`. */

import type { ComputedRef } from "vue";

import type { DirtyFields } from "./form-path-types.ts";
import { useForm } from "./use-form.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Values {
  name: string;
  address: { city: string };
  tags: string[];
}

declare const initial: Values;
const form = useForm({
  initialValues: initial,
  criteriaMode: "first",
  disabled: () => false,
});

const watched: ComputedRef<string> = form.watch("name");
void watched;
const dirty: ComputedRef<DirtyFields<Values>> = form.dirtyFields;
void dirty;
type _DirtyName = Expect<Equal<DirtyFields<Values>["name"], true | undefined>>;

form.setError("name", "Required");
form.setError("", ["Form"]);
form.reset(initial, { keepDirtyValues: true, keepIsSubmitSuccessful: true });

// @ts-expect-error criteriaMode accepts only "all" or "first".
useForm({ initialValues: initial, criteriaMode: "each" });
// @ts-expect-error watch rejects unknown paths.
form.watch("address.town");
