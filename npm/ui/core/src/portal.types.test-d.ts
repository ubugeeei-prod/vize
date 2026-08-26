/** Compile-only assertions for the public portal contract. */

import type { ShallowRef } from "vue";

import { registerPortalLayer, topPortalLayer, usePortalStack } from "./portal.ts";
import type { PortalStackEntry } from "./portal.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const stack = usePortalStack();
type _StackIsReadonly = Expect<
  Equal<typeof stack, Readonly<ShallowRef<readonly PortalStackEntry[]>>>
>;

export const top: PortalStackEntry | null = topPortalLayer();

declare const element: HTMLElement;
export const release: () => void = registerPortalLayer({ depth: 0, element });

declare const entry: PortalStackEntry;
// @ts-expect-error stack entries are immutable.
entry.depth = 2;
// @ts-expect-error a stack entry requires its portalled element.
registerPortalLayer({ depth: 0 });
