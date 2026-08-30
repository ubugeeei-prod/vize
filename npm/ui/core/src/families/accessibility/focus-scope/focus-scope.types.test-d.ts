/** Compile-only assertions for the public focus scope contract. */

import { ref } from "vue";
import type { ShallowRef } from "vue";

import {
  createFocusScope,
  type FocusScopeAutoFocusEvent,
  type FocusScopeController,
  type FocusScopeOptions,
} from "./focus-scope.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const root = ref<Element | null>(null);
const contain = ref(true);
export const options = {
  autoFocus: true,
  contain,
  onMountAutoFocus(event: FocusScopeAutoFocusEvent) {
    if (event.target === null) event.preventDefault();
  },
  restoreFocus: true,
  root,
} satisfies FocusScopeOptions;
export const controller: FocusScopeController = createFocusScope(options);

type _ActiveIsReadonly = Expect<Equal<typeof controller.isActive, Readonly<ShallowRef<boolean>>>>;
type _MovementReturnIsExact = Expect<
  Equal<ReturnType<typeof controller.focusNext>, HTMLElement | null>
>;

controller.focusFirst({ includeProgrammatic: true, preventScroll: false, wrap: true });
// @ts-expect-error contain is reactive but must resolve to a boolean.
createFocusScope({ contain: ref("yes"), root });
// @ts-expect-error roots must resolve to Elements rather than selector strings.
createFocusScope({ root: "#dialog" });
// @ts-expect-error movement option names are closed and type safe.
controller.focusNext({ loop: true });
