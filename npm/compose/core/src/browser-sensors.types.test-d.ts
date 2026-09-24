/** Compile-only assertions for the browser sensor composables. */

import type { ComputedRef, Ref, ShallowRef, WritableComputedRef } from "vue";
import { ref } from "vue";

import { useActiveElement } from "./active-element.js";
import { useElementBounding } from "./element-bounding.js";
import { resolveElement, resolveElements } from "./element-target.js";
import type { MaybeElementTarget } from "./element-target.js";
import { useFocus } from "./focus.js";
import { useElementVisibility } from "./intersection-observer.js";
import { useMagicKeys } from "./magic-keys.js";
import type { KeyCombo } from "./magic-keys.js";
import { useMouse } from "./mouse.js";
import type { MouseSourceType } from "./mouse.js";
import { usePinch } from "./pinch.js";
import { toPointerKind, usePointer } from "./pointer.js";
import type { PointerKind } from "./pointer.js";
import { PointerLockError, usePointerLock } from "./pointer-lock.js";
import { useElementSize } from "./resize-observer.js";
import { useScroll } from "./scroll.js";
import type { ScrollEdges } from "./scroll.js";
import { swipeDirection, useSwipe } from "./swipe.js";
import type { SwipeDirection } from "./swipe.js";
import { useTextSelection } from "./text-selection.js";
import { useWindowSize } from "./window-size.js";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const button: HTMLButtonElement;
declare const svg: SVGSVGElement;
declare const component: import("vue").ComponentPublicInstance;

// Element targets accept elements, component instances, refs, getters, and absence.
const _targets: readonly MaybeElementTarget[] = [
  button,
  svg,
  component,
  null,
  undefined,
  ref<HTMLElement | null>(null),
  () => button,
];
type _ResolveReturnsElementOrNull = Expect<
  Equal<ReturnType<typeof resolveElement>, Element | null>
>;
type _ResolveManyReturnsElements = Expect<Equal<ReturnType<typeof resolveElements>, Element[]>>;
// @ts-expect-error strings are not element targets.
resolveElement("#app");
// @ts-expect-error the window is not an element target.
resolveElement(window);

const size = useElementSize(button);
type _ElementSizeIsNumeric = Expect<Equal<typeof size.width, Readonly<Ref<number>>>>;
// @ts-expect-error element size refs are readonly.
size.width.value = 1;
// @ts-expect-error box options are the platform union.
useElementSize(button, { box: "margin-box" });

const bounding = useElementBounding(button);
type _BoundingFieldsAreReadonlyNumbers = Expect<
  Equal<typeof bounding.bottom, Readonly<Ref<number>>>
>;

const visibility = useElementVisibility(button, { once: true });
type _VisibilityIsBoolean = Expect<Equal<typeof visibility.isVisible, Readonly<Ref<boolean>>>>;

const windowSize = useWindowSize({ type: "outer" });
windowSize.width.value satisfies number;
// @ts-expect-error window size kinds are closed.
useWindowSize({ type: "visual" });

const scroll = useScroll(button);
type _ScrollXIsWritable = Expect<Equal<typeof scroll.x, WritableComputedRef<number>>>;
type _ArrivedStateIsReadonly = Expect<Equal<typeof scroll.arrivedState, Readonly<ScrollEdges>>>;
scroll.x.value = 10;
// @ts-expect-error arrived state is readonly.
scroll.arrivedState.top = false;
useScroll(window);

const mouse = useMouse({ type: (input) => ({ x: input.clientX, y: input.clientY }) });
type _MouseSourceIsClosed = Expect<
  Equal<typeof mouse.sourceType, Readonly<Ref<MouseSourceType | null>>>
>;
// @ts-expect-error coordinate spaces are closed.
useMouse({ type: "offset" });

const pointer = usePointer({ pointerTypes: ["pen"] });
type _PointerTypeIsClosed = Expect<Equal<typeof pointer.state.pointerType, PointerKind | null>>;
type _PointerKindNarrowing = Expect<Equal<ReturnType<typeof toPointerKind>, PointerKind | null>>;
// @ts-expect-error pointer kinds are closed.
usePointer({ pointerTypes: ["eye"] });

const lock = usePointerLock(button);
type _LockResolvesToElement = Expect<Equal<ReturnType<typeof lock.lock>, Promise<Element>>>;
type _LockElementIsShallow = Expect<
  Equal<typeof lock.element, Readonly<ShallowRef<Element | null>>>
>;
declare const lockError: PointerLockError;
type _LockErrorCodes = Expect<
  Equal<
    typeof lockError.code,
    | "VIZE_COMPOSE_POINTER_LOCK_UNSUPPORTED"
    | "VIZE_COMPOSE_POINTER_LOCK_NO_TARGET"
    | "VIZE_COMPOSE_POINTER_LOCK_FAILED"
  >
>;

const swipe = useSwipe(button, {
  onSwipeEnd: (_event, direction) => direction satisfies SwipeDirection,
});
type _SwipeDirectionIsClosed = Expect<Equal<typeof swipe.direction, ComputedRef<SwipeDirection>>>;
swipeDirection(1, 2, 3) satisfies SwipeDirection;

const pinch = usePinch(button);
pinch.scale.value satisfies number;
// @ts-expect-error pinch origin is readonly.
pinch.origin.x = 1;

const keys = useMagicKeys();
type _ComboRefs = Expect<Equal<ReturnType<typeof keys.isPressed>, ComputedRef<boolean>>>;
keys.isPressed("ctrl+shift+p");
keys.isPressed("meta_k");
// @ts-expect-error combos cannot end with a separator.
keys.isPressed("ctrl+");
// @ts-expect-error combos cannot start with a separator.
keys.isPressed("+s");
// @ts-expect-error combos cannot be empty.
keys.isPressed("");
type _NonLiteralCombosStayOpen = Expect<Equal<KeyCombo<string>, string>>;
const { ctrl_s: save } = keys.combos;
save satisfies ComputedRef<boolean> | undefined;
// @ts-expect-error the pressed set is readonly.
keys.current.add("a");

const focus = useFocus(button);
type _FocusIsWritable = Expect<Equal<typeof focus.focused, WritableComputedRef<boolean>>>;

const active = useActiveElement();
type _ActiveElementIsShallow = Expect<Equal<typeof active, Readonly<ShallowRef<Element | null>>>>;

const selection = useTextSelection();
type _SelectionTextIsString = Expect<Equal<typeof selection.text, ComputedRef<string>>>;
