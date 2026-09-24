import { readonly, ref, shallowReactive, shallowReadonly, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Pointer device kind reported by Pointer Events. */
export type PointerKind = "mouse" | "touch" | "pen";

/** Snapshot of the latest pointer event. */
export interface PointerState {
  /** Horizontal client coordinate. */
  readonly x: number;
  /** Vertical client coordinate. */
  readonly y: number;
  /** Normalized pressure in `[0, 1]`. */
  readonly pressure: number;
  /** Identifier of the pointer that produced the latest event. */
  readonly pointerId: number;
  /** Pen tilt around the Y axis in degrees. */
  readonly tiltX: number;
  /** Pen tilt around the X axis in degrees. */
  readonly tiltY: number;
  /** Contact geometry width in CSS pixels. */
  readonly width: number;
  /** Contact geometry height in CSS pixels. */
  readonly height: number;
  /** Pen rotation in degrees. */
  readonly twist: number;
  /** Device kind; `null` before the first event. */
  readonly pointerType: PointerKind | null;
}

/** Options for {@link usePointer}. */
export interface UsePointerOptions {
  /**
   * Device kinds to track.
   *
   * @default ["mouse", "touch", "pen"]
   */
  readonly pointerTypes?: readonly PointerKind[];

  /**
   * Event target to listen on.
   *
   * @default globalThis.window when available
   */
  readonly target?: MaybeRefOrGetter<EventTarget | null | undefined>;

  /**
   * State exposed before any input and during server rendering.
   *
   * @default all numbers `0`, `pointerType: null`
   */
  readonly initialValue?: Partial<PointerState>;
}

/** Reactive pointer state returned by {@link usePointer}. */
export interface PointerControls {
  /** Latest pointer snapshot. */
  readonly state: Readonly<PointerState>;
  /** Whether the pointer is inside the target (between enter/move and leave). */
  readonly isInside: Readonly<Ref<boolean>>;
}

/**
 * Track the latest pointer (mouse, touch, or pen) state.
 *
 * Listens passively for `pointerdown`, `pointermove`, `pointerup`, and
 * `pointerleave` on the reactive target and filters by device kind.
 * Listeners are removed with the owning reactive scope; server renders expose
 * the initial snapshot.
 *
 * @param options Device filter, target, and initial snapshot.
 * @default options {}
 * @returns Reactive snapshot and inside flag.
 */
export function usePointer(options: UsePointerOptions = {}): PointerControls {
  const pointerTypes = options.pointerTypes ?? ["mouse", "touch", "pen"];
  const state = shallowReactive<{ -readonly [Key in keyof PointerState]: PointerState[Key] }>({
    x: 0,
    y: 0,
    pressure: 0,
    pointerId: 0,
    tiltX: 0,
    tiltY: 0,
    width: 0,
    height: 0,
    twist: 0,
    pointerType: null,
    ...options.initialValue,
  });
  const isInside = ref(false);

  const onPointer = (event: Event): void => {
    if (!isPointerEvent(event)) return;
    const kind = toPointerKind(event.pointerType);
    if (!kind || !pointerTypes.includes(kind)) return;
    isInside.value = true;
    state.x = event.clientX;
    state.y = event.clientY;
    state.pressure = event.pressure;
    state.pointerId = event.pointerId;
    state.tiltX = event.tiltX;
    state.tiltY = event.tiltY;
    state.width = event.width;
    state.height = event.height;
    state.twist = event.twist;
    state.pointerType = kind;
  };
  const onLeave = (): void => {
    isInside.value = false;
  };

  const stop = watch(
    () => (options.target === undefined ? browserWindow() : toValue(options.target)),
    (target, _previous, onCleanup) => {
      if (!target) return;
      const listenerOptions: AddEventListenerOptions = { passive: true };
      for (const type of ["pointerdown", "pointermove", "pointerup"] as const) {
        target.addEventListener(type, onPointer, listenerOptions);
      }
      target.addEventListener("pointerleave", onLeave, listenerOptions);
      onCleanup(() => {
        for (const type of ["pointerdown", "pointermove", "pointerup"] as const) {
          target.removeEventListener(type, onPointer);
        }
        target.removeEventListener("pointerleave", onLeave);
      });
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return { state: shallowReadonly(state), isInside: readonly(isInside) };
}

/**
 * Narrow a platform `pointerType` string to the closed {@link PointerKind} union.
 *
 * @param value Raw `PointerEvent.pointerType`.
 * @returns The matching kind, or `null` for unknown/future device kinds.
 */
export function toPointerKind(value: string): PointerKind | null {
  return value === "mouse" || value === "touch" || value === "pen" ? value : null;
}

function isPointerEvent(event: Event): event is PointerEvent {
  return "pointerId" in event && "pointerType" in event;
}

function browserWindow(): EventTarget | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
