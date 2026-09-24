/** Compile-only assertions for the public Resizable contract. */

import type {
  ResizableEdge,
  ResizableHandleExpose,
  ResizablePhysicalEdge,
  ResizableResizeEvent,
  ResizableRootExpose,
  ResizableSize,
  ResizableState,
} from "./resizable.ts";
import {
  Resizable,
  ResizableHandle,
  ResizableRoot,
  constrainResizableSize,
  resolveResizableEdge,
} from "./resizable.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: ResizableRootExpose;
declare const handle: ResizableHandleExpose;
declare const event: ResizableResizeEvent;

type _Edge = Expect<Equal<ResizableEdge, ResizablePhysicalEdge | "end" | "start">>;
type _State = Expect<Equal<ResizableState, "idle" | "resizing">>;
type _Size = Expect<Equal<typeof root.size, ResizableSize>>;
type _EventEdge = Expect<Equal<typeof event.edge, ResizablePhysicalEdge>>;
type _Source = Expect<Equal<typeof event.source, "keyboard" | "pointer">>;
type _HandleElement = Expect<Equal<typeof handle.element, HTMLDivElement | null>>;
type _Resolve = Expect<Equal<ReturnType<typeof resolveResizableEdge>, ResizablePhysicalEdge>>;
type _Constrain = Expect<Equal<ReturnType<typeof constrainResizableSize>, ResizableSize>>;

root.setSize({ height: 100, width: 200 });
handle.focus();

const rootProps: InstanceType<typeof ResizableRoot>["$props"] = {
  defaultSize: { height: 100, width: 200 },
  dir: "rtl",
  largeStep: 40,
  lockAspectRatio: 16 / 9,
  maxWidth: 800,
  step: 5,
  "onUpdate:size": (value: ResizableSize) => value,
};
const handleProps: InstanceType<typeof ResizableHandle>["$props"] = { edge: "start" };

// @ts-expect-error edges are a closed union.
const badEdge: InstanceType<typeof ResizableHandle>["$props"] = { edge: "center" };

// @ts-expect-error sizes need both dimensions.
const badSize: InstanceType<typeof ResizableRoot>["$props"] = { size: { width: 10 } };

// @ts-expect-error lockAspectRatio is a boolean or ratio number.
const badLock: InstanceType<typeof ResizableRoot>["$props"] = { lockAspectRatio: "16:9" };

void Resizable;
void badEdge;
void badLock;
void badSize;
void handleProps;
void rootProps;
