import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef, toValue } from "vue";

import type {
  HoverController,
  HoverEvent,
  HoverEventType,
  HoverOptions,
  HoverPointerType,
  HoverProps,
} from "./hover-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_HOVER_OPTION";
const disposedDiagnostic = "VIZE_UI_HOVER_DISPOSED";
const setupDiagnostic = "VIZE_UI_HOVER_SETUP";
const pointerTypes = new Set<HoverPointerType>(["mouse", "pen"]);

interface ActiveHover {
  readonly pointerType: HoverPointerType;
  readonly releaseListeners: () => void;
  readonly target: Element;
  lastEvent: Event;
}

function readBoolean(value: HoverOptions["isDisabled"]): boolean {
  const resolved = toValue(value);
  if (resolved === undefined) return false;
  if (typeof resolved !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: isDisabled must resolve to a boolean`);
  }
  return resolved;
}

function readPointerType(value: HoverOptions["pointerType"]): HoverPointerType | null {
  const resolved = toValue(value) ?? null;
  if (resolved !== null && !pointerTypes.has(resolved)) {
    throw new TypeError(`${invalidOptionDiagnostic}: pointerType must resolve to mouse or pen`);
  }
  return resolved;
}

function eventElement(event: Event): Element | null {
  const value = event.currentTarget;
  return value && (value as Node).nodeType === 1 ? (value as Element) : null;
}

function modifier(event: Event | null, key: "altKey" | "ctrlKey" | "metaKey" | "shiftKey") {
  return event && key in event ? Boolean((event as MouseEvent)[key]) : false;
}

function createHoverEvent(
  type: HoverEventType,
  target: Element,
  pointerType: HoverPointerType,
  originalEvent: Event | null,
  isCanceled = false,
): HoverEvent {
  const hasPoint = originalEvent && "clientX" in originalEvent && "clientY" in originalEvent;
  return Object.freeze({
    type,
    pointerType,
    target,
    originalEvent,
    x: hasPoint ? Number((originalEvent as MouseEvent).clientX) : null,
    y: hasPoint ? Number((originalEvent as MouseEvent).clientY) : null,
    altKey: modifier(originalEvent, "altKey"),
    ctrlKey: modifier(originalEvent, "ctrlKey"),
    metaKey: modifier(originalEvent, "metaKey"),
    shiftKey: modifier(originalEvent, "shiftKey"),
    isCanceled,
  });
}

function notifyAll(notifications: readonly (() => void)[]): void {
  const errors: unknown[] = [];
  for (const notify of notifications) {
    try {
      notify();
    } catch (error) {
      errors.push(error);
    }
  }
  if (errors.length === 1) throw errors[0];
  if (errors.length > 1) throw new AggregateError(errors, "Hover callbacks failed");
}

function validateOptions(options: HoverOptions): void {
  for (const name of ["onHoverChange", "onHoverEnd", "onHoverStart"] as const) {
    const callback = options[name];
    if (callback !== undefined && typeof callback !== "function") {
      throw new TypeError(`${invalidOptionDiagnostic}: ${name} must be a function`);
    }
  }
  if (typeof options.pointerType !== "function") readPointerType(options.pointerType);
}

/** Create an SSR-safe mouse and pen hover normalizer for one host. */
export function createHover(options: HoverOptions = {}): HoverController {
  validateOptions(options);
  const hovered = shallowRef(false);
  let active: ActiveHover | null = null;
  let disposed = false;
  let transitionVersion = 0;
  let lastTouchTime = Number.NEGATIVE_INFINITY;

  const transition = (next: boolean, event: HoverEvent) => {
    if (hovered.value === next) return;
    hovered.value = next;
    const version = ++transitionVersion;
    const phase = next ? options.onHoverStart : options.onHoverEnd;
    notifyAll([
      () => phase?.(event),
      () => {
        if (transitionVersion === version) options.onHoverChange?.(next);
      },
    ]);
  };

  const end = (originalEvent: Event | null, isCanceled: boolean): boolean => {
    const current = active;
    if (!current) return false;
    current.releaseListeners();
    active = null;
    transition(
      false,
      createHoverEvent(
        "hoverend",
        current.target,
        current.pointerType,
        originalEvent ?? current.lastEvent,
        isCanceled,
      ),
    );
    return true;
  };

  const installListeners = (document: Document): (() => void) => {
    const removals: Array<() => void> = [];
    const listen = (
      owner: Document | Window,
      type: string,
      callback: EventListener,
      capture = true,
    ) => {
      owner.addEventListener(type, callback, capture);
      removals.push(() => owner.removeEventListener(type, callback, capture));
    };
    const revalidate = (event: Event) => {
      const current = active;
      if (!current) return;
      current.lastEvent = event;
      const filter = readPointerType(options.pointerType);
      if (readBoolean(options.isDisabled) || (filter && filter !== current.pointerType)) {
        end(event, true);
      }
    };
    try {
      listen(document, "pointermove", revalidate);
      listen(document, "mousemove", revalidate);
      listen(document, "pointerdown", ((event: PointerEvent) => {
        if (event.pointerType === "touch") end(event, true);
      }) as EventListener);
      listen(document, "touchstart", ((event: TouchEvent) => {
        lastTouchTime = event.timeStamp;
        end(event, true);
      }) as EventListener);
      listen(document, "visibilitychange", (() => {
        if (document.visibilityState === "hidden") end(null, true);
      }) as EventListener);
      if (document.defaultView) {
        listen(document.defaultView, "blur", (event) => end(event, true), false);
      }
    } catch (error) {
      for (const remove of removals.reverse()) remove();
      throw error;
    }
    return () => {
      for (const remove of removals.splice(0)) remove();
    };
  };

  const start = (event: Event, pointerType: HoverPointerType): void => {
    if (disposed || active || readBoolean(options.isDisabled)) return;
    const filter = readPointerType(options.pointerType);
    if (filter && filter !== pointerType) return;
    const target = eventElement(event);
    if (!target) return;
    const current: ActiveHover = {
      pointerType,
      releaseListeners: installListeners(target.ownerDocument),
      target,
      lastEvent: event,
    };
    active = current;
    transition(true, createHoverEvent("hoverstart", target, pointerType, event));
  };

  const leave = (event: MouseEvent | PointerEvent): void => {
    const current = active;
    if (!current || current.target !== eventElement(event)) return;
    const related = event.relatedTarget;
    if (related && (related as Node).nodeType && current.target.contains(related as Node)) return;
    current.lastEvent = event;
    end(event, false);
  };

  const hoverProps: Readonly<HoverProps> = Object.freeze({
    onMouseenter(event: MouseEvent) {
      if (event.view && "PointerEvent" in event.view) return;
      const elapsed = event.timeStamp - lastTouchTime;
      if (elapsed >= 0 && elapsed < 800) return;
      start(event, "mouse");
    },
    onMouseleave: leave,
    onMousemove(event: MouseEvent) {
      const current = active;
      if (!(event.view && "PointerEvent" in event.view) && current) current.lastEvent = event;
    },
    onPointercancel(event: PointerEvent) {
      if (active?.pointerType === event.pointerType) end(event, true);
    },
    onPointerenter(event: PointerEvent) {
      if (event.pointerType === "mouse" || event.pointerType === "pen") {
        start(event, event.pointerType);
      }
    },
    onPointerleave: leave,
    onPointermove(event: PointerEvent) {
      const current = active;
      if (current?.pointerType === event.pointerType) current.lastEvent = event;
    },
    onTouchstart(event: TouchEvent) {
      lastTouchTime = event.timeStamp;
      end(event, true);
    },
  });

  return Object.freeze({
    isHovered: shallowReadonly(hovered),
    hoverProps,
    cancel: () => {
      if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
      return end(null, true);
    },
    dispose: () => {
      if (disposed) return;
      active?.releaseListeners();
      active = null;
      hovered.value = false;
      transitionVersion++;
      disposed = true;
    },
  });
}

/** Create a hover normalizer disposed with the current Vue effect scope. */
export function useHover(options: HoverOptions = {}): HoverController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createHover(options);
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  HoverController,
  HoverEvent,
  HoverEventType,
  HoverOptions,
  HoverPointerType,
  HoverProps,
} from "./hover-types.ts";
