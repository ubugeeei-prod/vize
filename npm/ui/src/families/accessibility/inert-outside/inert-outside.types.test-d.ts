/** Compile-only assertions for the public outside-inerting contract. */

import { ref } from "vue";
import type { ShallowRef } from "vue";

import {
  createInertOutside,
  type InertOutsideController,
  type InertOutsideMode,
} from "./inert-outside.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const root = ref<Element | null>(null);
const mode = ref<InertOutsideMode>("both");
export const controller: InertOutsideController = createInertOutside({
  branches: () => [],
  enabled: ref(true),
  mode,
  root,
});

type _ActiveIsReadonly = Expect<Equal<typeof controller.isActive, Readonly<ShallowRef<boolean>>>>;
type _AffectedIsReadonly = Expect<
  Equal<typeof controller.affectedElements, Readonly<ShallowRef<readonly Element[]>>>
>;

// @ts-expect-error mode is a closed union.
createInertOutside({ mode: "automatic", root });
// @ts-expect-error branch entries retain Element safety.
createInertOutside({ branches: ["#portal"], root });
// @ts-expect-error enablement must resolve to boolean.
createInertOutside({ enabled: ref("yes"), root });
