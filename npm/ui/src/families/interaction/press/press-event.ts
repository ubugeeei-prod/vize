import { toValue } from "vue";

import type {
  PressEvent,
  PressEventType,
  PressKeyboardBehavior,
  PressOptions,
  PressPointerType,
} from "./press-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_PRESS_OPTION";
const pointerTypes = new Set<PressPointerType>([
  "keyboard",
  "mouse",
  "pen",
  "pointer",
  "touch",
  "virtual",
]);
const keyboardBehaviors = new Set<PressKeyboardBehavior>(["button", "link", "none"]);

interface Point {
  readonly x: number;
  readonly y: number;
}

/** Resolve and validate a reactive boolean option for JavaScript consumers. */
export function readBooleanOption(
  value: PressOptions[keyof Pick<
    PressOptions,
    "allowTextSelectionOnPress" | "isDisabled" | "preventFocusOnPress" | "shouldCancelOnPointerExit"
  >],
  name: string,
): boolean {
  const resolved = toValue(value);
  if (resolved === undefined) return false;
  if (typeof resolved !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: ${name} must resolve to a boolean`);
  }
  return resolved;
}

/** Resolve the closed keyboard behavior union without accepting mistyped JS. */
export function readKeyboardBehavior(
  value: PressOptions["keyboardBehavior"],
): PressKeyboardBehavior {
  const resolved = toValue(value) ?? "button";
  if (!keyboardBehaviors.has(resolved)) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: keyboardBehavior must resolve to button, link, or none`,
    );
  }
  return resolved;
}

/** Resolve a cross-realm event currentTarget without instanceof assumptions. */
export function eventElement(event: Event): Element | null {
  const value = event.currentTarget;
  return value && (value as Node).nodeType === 1 ? (value as Element) : null;
}

/** Map Pointer Events' extensible device string to the stable public union. */
export function pointerTypeOf(event: PointerEvent): PressPointerType {
  if (event.pointerId === -1 && event.pointerType === "") return "virtual";
  if (event.pointerType === "mouse" || event.pointerType === "pen") return event.pointerType;
  if (event.pointerType === "touch") return "touch";
  return "pointer";
}

/** Only a primary contact and its primary button can activate a control. */
export function isPrimaryPointer(event: PointerEvent): boolean {
  return event.isPrimary !== false && event.button === 0;
}

function eventPoint(event: Event | null, touchIdentifier: number | null = null): Point | null {
  if (!event) return null;
  if ("clientX" in event && "clientY" in event) {
    return { x: Number(event.clientX), y: Number(event.clientY) };
  }
  if ("changedTouches" in event) {
    const touches = Array.from((event as TouchEvent).changedTouches);
    const touch =
      (touchIdentifier === null
        ? touches[0]
        : touches.find(({ identifier }) => identifier === touchIdentifier)) ?? null;
    if (touch) return { x: touch.clientX, y: touch.clientY };
  }
  return null;
}

function modifier(event: Event | null, key: "altKey" | "ctrlKey" | "metaKey" | "shiftKey") {
  return event && key in event ? Boolean((event as KeyboardEvent)[key]) : false;
}

/** Build a frozen snapshot so later browser mutation cannot change callbacks' data. */
export function createPressEvent(
  type: PressEventType,
  target: Element,
  pointerType: PressPointerType,
  originalEvent: Event | null,
  isCanceled = false,
  touchIdentifier: number | null = null,
): PressEvent {
  if (!pointerTypes.has(pointerType)) {
    throw new TypeError(`${invalidOptionDiagnostic}: invalid press pointer type`);
  }
  const point = eventPoint(originalEvent, touchIdentifier);
  return Object.freeze({
    type,
    pointerType,
    target,
    originalEvent,
    x: point?.x ?? null,
    y: point?.y ?? null,
    altKey: modifier(originalEvent, "altKey"),
    ctrlKey: modifier(originalEvent, "ctrlKey"),
    metaKey: modifier(originalEvent, "metaKey"),
    shiftKey: modifier(originalEvent, "shiftKey"),
    isCanceled,
  });
}

/** Native elements retain their own activation timing and default actions. */
export function keyboardActivation(
  target: Element,
  key: string,
  behavior: PressKeyboardBehavior,
): "custom" | "native" | null {
  if (behavior === "none") return null;
  const tag = target.localName;
  if (tag === "a" || tag === "area") {
    if (target.hasAttribute("href")) return key === "Enter" ? "native" : null;
  }
  if (tag === "button") return key === "Enter" || key === " " ? "native" : null;
  if (tag === "summary") return key === "Enter" || key === " " ? "native" : null;
  if (tag === "input") {
    const type = (target.getAttribute("type") ?? "text").toLowerCase();
    if (type === "checkbox" || type === "radio") return key === " " ? "native" : null;
    const buttonTypes = new Set(["button", "file", "image", "reset", "submit"]);
    return buttonTypes.has(type) && (key === "Enter" || key === " ") ? "native" : null;
  }
  if (behavior === "link") return key === "Enter" ? "custom" : null;
  return key === "Enter" || key === " " ? "custom" : null;
}

/** Determine pointer containment without assuming events originate in one realm. */
export function isEventInside(
  event: Event,
  target: Element,
  touchIdentifier: number | null = null,
): boolean {
  const point = eventPoint(event, touchIdentifier);
  if (point) {
    const hit = target.ownerDocument.elementFromPoint?.(point.x, point.y);
    if (hit) return hit === target || target.contains(hit);
  }
  return event.composedPath().includes(target);
}

/** Apply a transient selection guard and return an exact, idempotent restore. */
export function disableTextSelection(target: Element): () => void {
  if (!("style" in target)) return () => undefined;
  const style = (target as HTMLElement).style;
  if (!style || typeof style.setProperty !== "function") return () => undefined;
  const properties = ["user-select", "-webkit-user-select"] as const;
  const previous = properties.map((property) => ({
    property,
    value: style.getPropertyValue(property),
    priority: style.getPropertyPriority(property),
  }));
  for (const property of properties) style.setProperty(property, "none");
  let restored = false;
  return () => {
    if (restored) return;
    restored = true;
    for (const { property, value, priority } of previous) {
      if (value) style.setProperty(property, value, priority);
      else style.removeProperty(property);
    }
  };
}

/** Validate callback slots eagerly so setup failures never install listeners. */
export function validatePressOptions(options: PressOptions): void {
  for (const name of [
    "onPress",
    "onPressChange",
    "onPressEnd",
    "onPressStart",
    "onPressUp",
  ] as const) {
    const callback = options[name];
    if (callback !== undefined && typeof callback !== "function") {
      throw new TypeError(`${invalidOptionDiagnostic}: ${name} must be a function`);
    }
  }
}
