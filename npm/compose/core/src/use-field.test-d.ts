/** Compile-only assertions for the `use-field` type contracts. */

import type { Ref } from "vue";

import type { StandardSchemaV1 } from "./standard-schema.ts";
import { useField } from "./use-field.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const age = useField(0, { rules: (value) => (value < 0 ? "Negative" : undefined) });
type _ValueFromInitial = Expect<Equal<typeof age.value, Ref<number>>>;

const tags = useField(() => [] as string[]);
type _ValueFromFactory = Expect<Equal<typeof tags.value.value, string[]>>;

declare const numberSchema: StandardSchemaV1<number, number>;
useField(1, { rules: [numberSchema] });

declare const stringSchema: StandardSchemaV1<string, string>;
// @ts-expect-error schemas must accept the field value.
useField(1, { rules: [stringSchema] });
// @ts-expect-error rules receive the field's value type.
useField(1, { rules: (value: string) => value });
// @ts-expect-error the trigger is a closed union.
useField(1, { validateOn: "input" });
// @ts-expect-error errors are read-only.
age.errors.value = [];
