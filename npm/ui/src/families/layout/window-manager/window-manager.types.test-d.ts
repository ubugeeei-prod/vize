/** Compile-only assertions for the window manager. */

import {
  bringWindowToFront,
  moveWindowRect,
  parseWindowLayout,
  resizeWindowRect,
  useWindowLayoutPersistence,
} from "./window-manager.ts";
import type {
  FloatingWindowSlotState,
  WindowEdge,
  WindowLayout,
  WindowMode,
  WindowPersistedLayout,
  WindowRect,
} from "./window-manager.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _ModesAreClosed = Expect<Equal<WindowMode, "normal" | "minimized" | "maximized">>;
type _EdgesAreClosed = Expect<Equal<WindowEdge, "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw">>;

const rect: WindowRect = { x: 0, y: 0, width: 10, height: 10 };
moveWindowRect(rect, 1, 2, [], null) satisfies WindowRect;
const limits = { minWidth: 1, minHeight: 1, maxWidth: 9, maxHeight: 9 };
resizeWindowRect(rect, "se", 1, 1, limits, null);
// @ts-expect-error edges are a closed union.
resizeWindowRect(rect, "center", 1, 1, limits, null);
bringWindowToFront(["a"], "a") satisfies readonly string[];
type _ParseMayFail = Expect<Equal<ReturnType<typeof parseWindowLayout>, WindowLayout | undefined>>;

declare const layout: WindowLayout;
// @ts-expect-error layouts are readonly.
layout.order = [];
// @ts-expect-error modes are closed.
const _badEntry: WindowLayout = { windows: { a: { ...rect, mode: "floating" } }, order: [] };

type _PersistenceReturnsARef = Expect<
  Equal<ReturnType<typeof useWindowLayoutPersistence>, WindowPersistedLayout>
>;
// @ts-expect-error a storage key is required.
useWindowLayoutPersistence({});

declare const slot: FloatingWindowSlotState;
slot.handleProps.onKeydown satisfies (event: KeyboardEvent) => void;
slot.toggleMaximize() satisfies void;
type _SlotStateMode = Expect<Equal<typeof slot.state.mode, WindowMode>>;
