/** Compile-only assertions for the DOM helper composables. */

import type { ComputedRef, Ref, ShallowRef, WritableComputedRef } from "vue";
import { ref } from "vue";

import { useAnimate } from "./animate.js";
import { breakpointsTailwind, useBreakpoints } from "./breakpoints.js";
import { useColorMode, useDark } from "./color-mode.js";
import type { ColorModeValue } from "./color-mode.js";
import { useDraggable } from "./draggable.js";
import type { DragAxis } from "./draggable.js";
import { useElementRef } from "./element-ref.js";
import { onKeyStroke } from "./on-key-stroke.js";
import { onLongPress } from "./on-long-press.js";
import { useScrollLock } from "./scroll-lock.js";
import { TransitionPresets, useTransition } from "./transition.js";
import { useVirtualList } from "./virtual-list.js";
import type { VirtualListItem } from "./virtual-list.js";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const element: HTMLElement;

const drag = useDraggable(element, { axis: "x" });
type _DragXWritable = Expect<Equal<typeof drag.x, Ref<number>>>;
// @ts-expect-error axes are closed.
useDraggable(element, { axis: "z" });
const _axis: DragAxis = "both";

const virtual = useVirtualList([{ id: 1 }] as const, { itemSize: (_index, item) => item.id });
type _VirtualItemsKeepTheSourceType = Expect<
  Equal<typeof virtual.list, ComputedRef<readonly VirtualListItem<{ readonly id: 1 }>[]>>
>;
// @ts-expect-error item size functions receive typed items.
useVirtualList(["a"], { itemSize: (_index, item: number) => item });

useScrollLock(element) satisfies WritableComputedRef<boolean>;

const divRef = useElementRef((value): value is HTMLDivElement => value.tagName === "DIV");
type _GuardNarrows = Expect<
  Equal<typeof divRef.element, Readonly<ShallowRef<HTMLDivElement | null>>>
>;
const anyRef = useElementRef();
type _DefaultIsElement = Expect<Equal<typeof anyRef.element, Readonly<ShallowRef<Element | null>>>>;

useAnimate(element, [{ opacity: 0, transform: "none", offset: 0 }, { opacity: 1 }], 200);
useAnimate(element, { opacity: [0, 1], easing: "ease-out" }, { duration: 100, iterations: 2 });
// @ts-expect-error keyframe properties are checked against CSSStyleDeclaration.
useAnimate(element, [{ opacityy: 0 }], 200);

const single = useTransition(ref(1));
type _NumberStaysNumber = Expect<Equal<typeof single, Readonly<Ref<number>>>>;
const pair = useTransition(ref([0, 1] as const));
type _TupleShapeIsPreserved = Expect<Equal<typeof pair, Readonly<Ref<[number, number]>>>>;
useTransition(ref(0), { easing: "easeOutBack" });
useTransition(ref(0), { easing: TransitionPresets.easeInCubic });
// @ts-expect-error preset names are closed.
useTransition(ref(0), { easing: "bounce" });

const bp = useBreakpoints(breakpointsTailwind);
bp["2xl"].value satisfies boolean;
bp.greater("md").value satisfies boolean;
type _ActiveIsTyped = Expect<
  Equal<typeof bp.active, ComputedRef<"sm" | "md" | "lg" | "xl" | "2xl" | null>>
>;
// @ts-expect-error unknown breakpoint names are rejected.
bp.greater("xxl");
// @ts-expect-error breakpoint names cannot shadow helper methods.
useBreakpoints({ current: 100 });

const color = useColorMode({ modes: { sepia: "sepia" } });
type _ModesIncludeCustom = Expect<
  Equal<typeof color.mode, WritableComputedRef<ColorModeValue<"sepia">>>
>;
color.mode.value = "sepia";
// @ts-expect-error unregistered custom modes are rejected.
color.mode.value = "neon";
useDark() satisfies WritableComputedRef<boolean>;

onKeyStroke(["a", "b"], (event) => event.key satisfies string);
onLongPress(element, (event) => event.pointerId satisfies number, { delay: 300 });
