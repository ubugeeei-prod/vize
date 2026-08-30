/** Compile-only assertions for the public typeahead contract. */

import { ref } from "vue";
import type { HTMLAttributes, ShallowRef } from "vue";

import { createCollectionRegistry } from "../../../collection.ts";
import { createTypeahead, type TypeaheadMatch, type TypeaheadProps } from "./typeahead.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const registry = createCollectionRegistry<"alpha" | "bravo", { label: string }>();
registry.register({ key: "alpha", value: { label: "Alpha" }, textValue: "Alpha" });
const timeout = ref(500);
export const controller = createTypeahead({
  registry,
  timeout,
  onMatch(match: TypeaheadMatch<"alpha" | "bravo">) {
    const key: "alpha" | "bravo" = match.key;
    void key;
  },
});

type _QueryIsReadonly = Expect<Equal<typeof controller.query, Readonly<ShallowRef<string>>>>;
type _PropsAreExact = Expect<Equal<typeof controller.typeaheadProps, Readonly<TypeaheadProps>>>;
type _InputKeyIsExact = Expect<
  Equal<ReturnType<typeof controller.input>, "alpha" | "bravo" | null>
>;

export const vueAttributes: HTMLAttributes = controller.typeaheadProps;
// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.query.value = "a";
// @ts-expect-error timeout must resolve to a number.
createTypeahead({ registry, timeout: "500" });
// @ts-expect-error manual input accepts a string grapheme.
controller.input(1);
