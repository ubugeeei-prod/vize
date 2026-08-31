/** Compile-only assertions for the public composite navigation contract. */

import { ref } from "vue";
import type { ComputedRef, HTMLAttributes } from "vue";

import { createCollectionRegistry } from "../collection/collection.ts";
import {
  createCompositeNavigation,
  type CompositeContainerProps,
  type CompositeItemProps,
  type CompositeNavigationChange,
} from "./composite-navigation.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const registry = createCollectionRegistry<"alpha" | "bravo", { label: string }>();
registry.register({ key: "alpha", value: { label: "Alpha" }, textValue: "Alpha" });
const orientation = ref<"horizontal" | "vertical">("vertical");

export const roving = createCompositeNavigation({
  registry,
  orientation,
  getItemId: ({ key }) => `item-${key}`,
  onNavigate(change: CompositeNavigationChange<"alpha" | "bravo">) {
    const key: "alpha" | "bravo" = change.key;
    void key;
  },
});
export const activeDescendant = createCompositeNavigation({
  registry,
  focusStrategy: "active-descendant",
  getItemId: ({ key }) => `item-${key}`,
});

type _ActiveKeyIsExact = Expect<
  Equal<typeof roving.activeKey, ComputedRef<"alpha" | "bravo" | null>>
>;
type _ContainerPropsAreExact = Expect<
  Equal<ReturnType<typeof roving.getContainerProps>, Readonly<CompositeContainerProps>>
>;
type _ItemPropsAreExact = Expect<
  Equal<ReturnType<typeof roving.getItemProps>, Readonly<CompositeItemProps>>
>;
type _NavigateKeyIsExact = Expect<
  Equal<ReturnType<typeof roving.navigate>, "alpha" | "bravo" | null>
>;

export const containerAttributes: HTMLAttributes = roving.getContainerProps();
export const itemAttributes: HTMLAttributes = roving.getItemProps("alpha");
// @ts-expect-error active-descendant requires a stable item ID projection.
createCompositeNavigation({ registry, focusStrategy: "active-descendant" });
createCompositeNavigation({
  registry,
  focusStrategy: "active-descendant",
  getItemId: ({ key }) => key,
  // @ts-expect-error the active-descendant strategy does not accept preventScroll.
  preventScroll: true,
});
// @ts-expect-error navigation commands do not expose internal transition intents.
roving.navigate("typeahead");
// @ts-expect-error key inference rejects keys outside the registry union.
roving.getItemProps("charlie");
