/** Compile-only assertions for the `use-form` type contracts. */

import type { ComputedRef, WritableComputedRef } from "vue";

import type { StandardSchemaV1 } from "./standard-schema.ts";
import { useForm } from "./use-form.ts";
import type { FormArrayPath, FormPath, FormPathValue, FormResult } from "./use-form.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Values {
  name: string;
  birthday: Date;
  address: { city: string; geo: { lat: number } };
  tags: string[];
  people: { first: string; age: number }[];
}

type _Paths = Expect<
  Equal<
    FormPath<Values>,
    | "name"
    | "birthday"
    | "address"
    | "address.city"
    | "address.geo"
    | "address.geo.lat"
    | "tags"
    | `tags.${number}`
    | "people"
    | `people.${number}`
    | `people.${number}.first`
    | `people.${number}.age`
  >
>;
type _DeepValue = Expect<Equal<FormPathValue<Values, "address.geo.lat">, number>>;
type _ArrayItemValue = Expect<Equal<FormPathValue<Values, `people.${number}.first`>, string>>;
type _ArrayPaths = Expect<Equal<FormArrayPath<Values>, "tags" | "people">>;

declare const initial: Values;
const form = useForm({ initialValues: initial });
const city = form.field("address.city");
type _FieldValueIsTyped = Expect<Equal<typeof city.value, WritableComputedRef<string>>>;
type _FieldPathIsLiteral = Expect<Equal<typeof city.path, "address.city">>;
const firstAge = form.field("people.0.age");
type _IndexedFieldValue = Expect<Equal<typeof firstAge.value.value, number>>;

const people = form.fieldArray("people");
people.append({ first: "Ada", age: 36 });
type _EntryValue = Expect<
  Equal<(typeof people.entries.value)[number]["value"], { first: string; age: number }>
>;

form.setValue("birthday", new Date());
const lat: number = form.getValue("address.geo.lat");
void lat;

declare const schema: StandardSchemaV1<Values, { readonly id: string }>;
const parsed = useForm({ initialValues: initial, schema });
const result = parsed.validate();
type _SchemaOutputFlows = Expect<
  Equal<Awaited<typeof result>, FormResult<{ readonly id: string }>>
>;

useForm({
  initialValues: initial,
  validators: {
    "address.geo.lat": (value) => (value > 90 ? "Out of range" : undefined),
    name: async (value, { values }) => (values.tags.includes(value) ? ["Taken"] : undefined),
  },
});

const valid: ComputedRef<boolean> = form.valid;
void valid;

// @ts-expect-error unknown paths are rejected.
form.field("address.town");
// @ts-expect-error leaves such as Date are not traversed.
form.field("birthday.getTime");
// @ts-expect-error values must match the path type.
form.setValue("address.geo.lat", "north");
// @ts-expect-error only array paths have field-array controls.
form.fieldArray("name");
// @ts-expect-error field array items keep their shape.
people.append({ first: "Ada" });
// @ts-expect-error field validators receive the path's value type.
useForm({ initialValues: initial, validators: { name: (value: number) => String(value) } });
