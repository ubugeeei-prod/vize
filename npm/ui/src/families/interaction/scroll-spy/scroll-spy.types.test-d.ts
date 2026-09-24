/** Compile-only assertions for the public scroll spy contract. */

import type { ShallowRef } from "vue";

import {
  createScrollSpy,
  useScrollSpy,
  type ScrollSpyChangeReason,
  type ScrollSpyController,
  type ScrollSpyOptions,
} from "./scroll-spy.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const controller: ScrollSpyController;

type _ReasonIsLiteral = Expect<Equal<ScrollSpyChangeReason, "navigation" | "scroll">>;
type _ActiveIsReadonly = Expect<
  Equal<typeof controller.activeId, Readonly<ShallowRef<string | null>>>
>;
type _CreateReturns = Expect<Equal<ReturnType<typeof createScrollSpy>, ScrollSpyController>>;
type _UseReturns = Expect<Equal<ReturnType<typeof useScrollSpy>, ScrollSpyController>>;

const options: ScrollSpyOptions = {
  ids: () => ["a", "b"],
  initialActiveId: "a",
  isDisabled: false,
  offset: 64,
  onActiveChange: (id, previous, reason) => [id, previous, reason],
  root: null,
};

// @ts-expect-error ids are strings.
const badIds: ScrollSpyOptions = { ids: [1, 2] };

// @ts-expect-error the active id is read-only.
controller.activeId.value = "a";

void badIds;
void options;
