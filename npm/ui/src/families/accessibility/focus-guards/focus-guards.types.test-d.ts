/** Compile-only assertions for the public focus-guard contract. */

import { ref } from "vue";
import type { ShallowRef } from "vue";

import {
  createFocusGuards,
  type FocusGuardDirection,
  type FocusGuardProps,
  type FocusGuardsController,
} from "./focus-guards.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const root = ref<Element | null>(null);
const controller: FocusGuardsController = createFocusGuards({
  root,
  branches: () => [],
  enabled: ref(true),
  onRedirect(event) {
    const direction: FocusGuardDirection = event.direction;
    void direction;
  },
});
const props: Readonly<FocusGuardProps> = controller.beforeProps;
void props;

type _ActiveIsReadonly = Expect<Equal<typeof controller.isActive, Readonly<ShallowRef<boolean>>>>;
type _GuardingIsReadonly = Expect<
  Equal<typeof controller.isGuarding, Readonly<ShallowRef<boolean>>>
>;

// @ts-expect-error branches retain DOM element type safety.
createFocusGuards({ root, branches: [window] });
// @ts-expect-error enablement must resolve to boolean.
createFocusGuards({ root, enabled: ref("yes") });
// @ts-expect-error redirect evidence is immutable.
controller.beforeProps.tabindex = 1;
