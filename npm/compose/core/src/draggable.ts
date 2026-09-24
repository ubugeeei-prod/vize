import { computed, readonly, ref, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

import { isElementNode, resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import type { Point } from "./mouse.ts";
import { toPointerKind } from "./pointer.ts";
import type { PointerKind } from "./pointer.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Axis constraint for {@link useDraggable}. */
export type DragAxis = "x" | "y" | "both";

/** Rectangle (in the same coordinate space as the position) that bounds dragging. */
export interface DragBounds {
  /** Minimum x. */
  readonly left: number;
  /** Minimum y. */
  readonly top: number;
  /** Maximum x of the dragged element's right edge. */
  readonly right: number;
  /** Maximum y of the dragged element's bottom edge. */
  readonly bottom: number;
}

/** Options for {@link useDraggable}. */
export interface UseDraggableOptions {
  /**
   * Element that starts the drag. Defaults to the dragged element itself.
   *
   * @default undefined (the target)
   */
  readonly handle?: MaybeElementTarget;

  /**
   * Restrict movement to one axis.
   *
   * @default "both"
   */
  readonly axis?: MaybeRefOrGetter<DragAxis>;

  /**
   * Keep the element inside these bounds, or inside a container element's
   * client rectangle.
   *
   * @default undefined (unbounded)
   */
  readonly bounds?: MaybeRefOrGetter<DragBounds | null | undefined> | MaybeElementTarget;

  /**
   * Position before the first drag and during server rendering.
   *
   * @default { x: 0, y: 0 }
   */
  readonly initialValue?: Point;

  /**
   * Device kinds that may start a drag.
   *
   * @default ["mouse", "touch", "pen"]
   */
  readonly pointerTypes?: readonly PointerKind[];

  /**
   * Only start when the pointer goes down on the handle itself, not a descendant.
   *
   * @default false
   */
  readonly exact?: boolean;

  /**
   * Call `preventDefault()` on pointer events while dragging.
   *
   * @default false
   */
  readonly preventDefault?: boolean;

  /**
   * Temporarily disable dragging.
   *
   * @default false
   */
  readonly disabled?: MaybeRefOrGetter<boolean>;

  /**
   * Called before a drag starts. Return `false` to cancel it.
   *
   * @default undefined
   */
  readonly onStart?: (position: Point, event: PointerEvent) => boolean | void;

  /**
   * Called after every position update.
   *
   * @default undefined
   */
  readonly onMove?: (position: Point, event: PointerEvent) => void;

  /**
   * Called when the drag ends.
   *
   * @default undefined
   */
  readonly onEnd?: (position: Point, event: PointerEvent) => void;
}

/** Reactive drag state returned by {@link useDraggable}. */
export interface DraggableControls {
  /** Horizontal position (the element's left edge in client coordinates). */
  readonly x: Ref<number>;
  /** Vertical position (the element's top edge in client coordinates). */
  readonly y: Ref<number>;
  /** Current position snapshot. */
  readonly position: ComputedRef<Point>;
  /** Whether a drag is in progress. */
  readonly isDragging: Readonly<Ref<boolean>>;
  /** `left`/`top` declaration for `position: fixed` or `absolute` placement. */
  readonly style: ComputedRef<string>;
  /** Remove listeners. Idempotent. */
  readonly stop: () => void;
}

/**
 * Make an element draggable with Pointer Events.
 *
 * `pointerdown` on the handle records the offset between the pointer and the
 * element; moves are tracked on the owner document (so fast drags that leave
 * the element keep working) and constrained by `axis` and `bounds`. `x`/`y`
 * are writable for programmatic placement. Server renders expose
 * `initialValue`; listeners are removed with the owning reactive scope.
 *
 * @param target Reactive element being dragged.
 * @param options Handle, axis, bounds, device filter, and lifecycle callbacks.
 * @default options {}
 * @returns Writable position, drag flag, inline style, and stop control.
 */
export function useDraggable(
  target: MaybeElementTarget,
  options: UseDraggableOptions = {},
): DraggableControls {
  const initial = options.initialValue ?? { x: 0, y: 0 };
  const x = ref(initial.x);
  const y = ref(initial.y);
  const isDragging = ref(false);
  const pointerTypes = options.pointerTypes ?? ["mouse", "touch", "pen"];
  const position = computed<Point>(() => ({ x: x.value, y: y.value }));
  let offset: Point | null = null;
  let activePointer: number | null = null;

  const constrain = (element: Element, next: Point): Point => {
    const bounds = readBounds(options.bounds);
    if (!bounds) return next;
    const rect = element.getBoundingClientRect();
    return {
      x: clamp(next.x, bounds.left, Math.max(bounds.left, bounds.right - rect.width)),
      y: clamp(next.y, bounds.top, Math.max(bounds.top, bounds.bottom - rect.height)),
    };
  };
  const accepts = (event: Event): event is PointerEvent => {
    if (!isPointerEvent(event)) return false;
    const kind = toPointerKind(event.pointerType);
    return kind !== null && pointerTypes.includes(kind);
  };
  const handleEvent = (event: PointerEvent): void => {
    if (options.preventDefault && event.cancelable) event.preventDefault();
  };

  const stopWatch = watch(
    () =>
      [
        resolveElement(target),
        options.handle === undefined ? undefined : resolveElement(options.handle),
      ] as const,
    ([element, handleOverride], _previous, onCleanup) => {
      const handle = handleOverride === undefined ? element : handleOverride;
      if (!element || !handle) return;
      const document = element.ownerDocument;

      const onMove = (event: Event): void => {
        if (!accepts(event) || event.pointerId !== activePointer || !offset) return;
        const axis = toValue(options.axis) ?? "both";
        const next = constrain(element, {
          x: axis === "y" ? x.value : event.clientX - offset.x,
          y: axis === "x" ? y.value : event.clientY - offset.y,
        });
        x.value = next.x;
        y.value = next.y;
        handleEvent(event);
        options.onMove?.(next, event);
      };
      const onUp = (event: Event): void => {
        if (!accepts(event) || event.pointerId !== activePointer) return;
        activePointer = null;
        offset = null;
        isDragging.value = false;
        document.removeEventListener("pointermove", onMove);
        document.removeEventListener("pointerup", onUp);
        document.removeEventListener("pointercancel", onUp);
        handleEvent(event);
        options.onEnd?.(position.value, event);
      };
      const onDown = (event: Event): void => {
        if (!accepts(event) || toValue(options.disabled) || activePointer !== null) return;
        if (event.pointerType === "mouse" && event.button !== 0) return;
        if (options.exact && event.target !== handle) return;
        const rect = element.getBoundingClientRect();
        const start = { x: rect.left, y: rect.top };
        if (options.onStart?.(start, event) === false) return;
        activePointer = event.pointerId;
        offset = { x: event.clientX - rect.left, y: event.clientY - rect.top };
        isDragging.value = true;
        document.addEventListener("pointermove", onMove);
        document.addEventListener("pointerup", onUp);
        document.addEventListener("pointercancel", onUp);
        handleEvent(event);
      };

      handle.addEventListener("pointerdown", onDown);
      onCleanup(() => {
        handle.removeEventListener("pointerdown", onDown);
        document.removeEventListener("pointermove", onMove);
        document.removeEventListener("pointerup", onUp);
        document.removeEventListener("pointercancel", onUp);
        activePointer = null;
        offset = null;
        isDragging.value = false;
      });
    },
    { immediate: true, flush: "post" },
  );
  const stop = (): void => stopWatch.stop();
  tryOnScopeDispose(stop);

  return {
    x,
    y,
    position,
    isDragging: readonly(isDragging),
    style: computed(() => `left:${x.value}px;top:${y.value}px;`),
    stop,
  };
}

function readBounds(source: UseDraggableOptions["bounds"]): DragBounds | null {
  if (source === undefined) return null;
  const value: unknown = toValue(source);
  if (isDragBounds(value)) return value;
  const root: unknown =
    typeof value === "object" && value !== null && "$el" in value ? value.$el : value;
  if (!isElementNode(root)) return null;
  const element = root;
  const rect = element.getBoundingClientRect();
  return { left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom };
}

function isDragBounds(value: unknown): value is DragBounds {
  return (
    typeof value === "object" &&
    value !== null &&
    !("nodeType" in value) &&
    !("$el" in value) &&
    "left" in value &&
    "right" in value
  );
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), maximum);
}

function isPointerEvent(event: Event): event is PointerEvent {
  return "pointerId" in event && "pointerType" in event;
}
