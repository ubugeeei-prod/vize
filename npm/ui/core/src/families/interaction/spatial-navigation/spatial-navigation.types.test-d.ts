/** Compile-only assertions for the public spatial navigation contract. */

import { ref } from "vue";
import type { HTMLAttributes } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import {
  createSpatialNavigation,
  type SpatialNavigationChange,
  type SpatialNavigationProps,
} from "./spatial-navigation.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const registry = createCollectionRegistry<"alpha" | "bravo", { label: string }>();
registry.register({ key: "alpha", value: { label: "Alpha" }, textValue: "Alpha" });
const algorithm = ref<"grid" | "normal">("normal");
export const controller = createSpatialNavigation({
  registry,
  algorithm,
  getRect: ({ key }) => ({
    bottom: 10,
    height: 10,
    left: key === "alpha" ? 0 : 20,
    right: key === "alpha" ? 10 : 30,
    top: 0,
    width: 10,
  }),
  onNavigate(change: SpatialNavigationChange<"alpha" | "bravo">) {
    const key: "alpha" | "bravo" = change.key;
    void key;
  },
});

type _PropsAreExact = Expect<
  Equal<typeof controller.spatialNavigationProps, Readonly<SpatialNavigationProps>>
>;
type _FindKeyIsExact = Expect<
  Equal<ReturnType<typeof controller.findTarget>, "alpha" | "bravo" | null>
>;
type _NavigateKeyIsExact = Expect<
  Equal<ReturnType<typeof controller.navigate>, "alpha" | "bravo" | null>
>;

export const vueAttributes: HTMLAttributes = controller.spatialNavigationProps;
// @ts-expect-error scoring algorithms are a closed union.
createSpatialNavigation({ registry, algorithm: "nearest" });
// @ts-expect-error directions are physical and closed.
controller.navigate("inline-end");
// @ts-expect-error keys retain the registry's literal union.
controller.findTarget("right", "charlie");
