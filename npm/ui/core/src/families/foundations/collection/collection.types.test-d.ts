/** Compile-only assertions for the public collection registry contract. */

import type { ComputedRef } from "vue";

import {
  createCollectionRegistry,
  type CollectionItem,
  type CollectionKey,
  type CollectionNavigationDirection,
} from "./collection.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface OptionData {
  readonly label: string;
  readonly payload: number;
}

export const stringKey: CollectionKey = "option-a";
export const numericKey: CollectionKey = 42;

// @ts-expect-error symbols cannot remain stable across SSR and hydration.
export const symbolKey: CollectionKey = Symbol("option");

// @ts-expect-error objects are outside the closed serializable key contract.
export const objectKey: CollectionKey = { id: "option" };

export const registry = createCollectionRegistry<"alpha" | "bravo", OptionData>();
export const alphaRegistration = registry.register({
  key: "alpha",
  value: { label: "Alpha", payload: 1 },
  textValue: () => "Alpha",
  disabled: () => false,
  element: () => null,
});

// @ts-expect-error keys remain inside the registry literal union.
registry.register({ key: "charlie", value: { label: "Charlie", payload: 3 } });

// @ts-expect-error consumer data is required for every item.
registry.register({ key: "bravo" });

registry.register({
  key: "bravo",
  // @ts-expect-error payload remains a number rather than widening to string.
  value: { label: "Bravo", payload: "2" },
});

createCollectionRegistry<string, OptionData>().register({
  key: "invalid-disabled",
  value: { label: "Invalid", payload: 0 },
  // @ts-expect-error disabled sources must resolve to booleans.
  disabled: () => "no",
});

type _ItemsRemainTyped = Expect<
  Equal<
    typeof registry.items,
    ComputedRef<readonly CollectionItem<"alpha" | "bravo", OptionData>[]>
  >
>;
type _ActiveKeyRemainsReadonly = Expect<
  Equal<typeof registry.activeKey, ComputedRef<"alpha" | "bravo" | null>>
>;

export const navigationDirection: CollectionNavigationDirection = "next";

// @ts-expect-error page navigation belongs to a higher-level composite policy.
export const unsupportedDirection: CollectionNavigationDirection = "page-down";

// @ts-expect-error consumers cannot replace the registry active-key ref.
registry.activeKey.value = "alpha";
