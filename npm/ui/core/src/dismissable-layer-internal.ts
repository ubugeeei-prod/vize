import { toValue } from "vue";

import type {
  DismissableLayerDismissEvent,
  DismissableLayerDismissReason,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerFocusOutsideEvent,
  DismissableLayerOptions,
  DismissableLayerPointerDownOutsideEvent,
  DismissableLayerPointerType,
} from "./dismissable-layer-types.ts";

const optionDiagnostic = "VIZE_UI_DISMISSABLE_LAYER_OPTION";
const rootDiagnostic = "VIZE_UI_DISMISSABLE_LAYER_ROOT";

interface Point {
  readonly x: number;
  readonly y: number;
}

/** Resolve an event target without assuming it comes from the current realm. */
export function eventElement(value: EventTarget | null): Element | null {
  const candidate = value as Partial<Element> | null;
  return candidate?.nodeType === 1 && typeof candidate.getRootNode === "function"
    ? (candidate as Element)
    : null;
}

export function readBoolean(source: unknown, name: string, fallback: boolean): boolean {
  const value = toValue(source);
  if (value === undefined) return fallback;
  if (typeof value !== "boolean") {
    throw new TypeError(`${optionDiagnostic}: ${name} must resolve to a boolean`);
  }
  return value;
}

export function readRoot(source: unknown): Element | null {
  const value = toValue(source);
  if (value === undefined || value === null) return null;
  const candidate = value as Partial<Element>;
  if (candidate.nodeType !== 1 || !candidate.ownerDocument) {
    throw new TypeError(`${rootDiagnostic}: root must resolve to an Element or null`);
  }
  return value as Element;
}

export function readBranches(source: unknown, document: Document | null): readonly Element[] {
  const value = toValue(source) ?? [];
  if (!Array.isArray(value)) {
    throw new TypeError(`${optionDiagnostic}: branches must resolve to an array of Elements`);
  }
  return validateBranches(value, document);
}

export function validateBranches(source: readonly unknown[], document: Document | null): Element[] {
  const unique = [...new Set(source)];
  for (const branch of unique) {
    const candidate = branch as Partial<Element>;
    if (candidate.nodeType !== 1 || !candidate.ownerDocument) {
      throw new TypeError(`${optionDiagnostic}: branches must contain only Elements`);
    }
    if (document && candidate.ownerDocument !== document) {
      throw new TypeError(`${optionDiagnostic}: branches must share the root Document`);
    }
  }
  return unique as Element[];
}

export function validateOptions(options: DismissableLayerOptions): void {
  if (!options || typeof options !== "object" || Array.isArray(options)) {
    throw new TypeError(`${optionDiagnostic}: options must be an object`);
  }
  const root = readRoot(options.root);
  readBranches(options.branches, root?.ownerDocument ?? null);
  readBoolean(options.enabled, "enabled", true);
  readBoolean(options.outsidePointerDown, "outsidePointerDown", true);
  readBoolean(options.outsideFocus, "outsideFocus", true);
  readBoolean(options.escapeKey, "escapeKey", true);
  for (const name of [
    "onPointerDownOutside",
    "onFocusOutside",
    "onInteractOutside",
    "onEscapeKeyDown",
    "onDismiss",
  ] as const) {
    if (options[name] !== undefined && typeof options[name] !== "function") {
      throw new TypeError(`${optionDiagnostic}: ${name} must be a function`);
    }
  }
}

function modifier(event: Event, key: "altKey" | "ctrlKey" | "metaKey" | "shiftKey"): boolean {
  return key in event ? Boolean((event as KeyboardEvent)[key]) : false;
}

function eventPoint(event: Event): Point | null {
  if ("clientX" in event && "clientY" in event) {
    return { x: Number(event.clientX), y: Number(event.clientY) };
  }
  if ("changedTouches" in event) {
    const touch = Array.from((event as TouchEvent).changedTouches)[0];
    if (touch) return { x: touch.clientX, y: touch.clientY };
  }
  return null;
}

function pointerTypeOf(event: PointerEvent | MouseEvent | TouchEvent): DismissableLayerPointerType {
  if ("changedTouches" in event) return "touch";
  if ("pointerType" in event) {
    if (event.pointerId === -1 && event.pointerType === "") return "virtual";
    if (event.pointerType === "mouse" || event.pointerType === "pen") return event.pointerType;
    if (event.pointerType === "touch") return "touch";
    return "pointer";
  }
  return "mouse";
}

function preventable() {
  let prevented = false;
  return {
    get defaultPrevented() {
      return prevented;
    },
    preventDefault: () => {
      prevented = true;
    },
  };
}

/** Build an immutable outside pointer event that is stable after native dispatch mutates. */
export function createPointerDownOutsideEvent(
  originalEvent: PointerEvent | MouseEvent | TouchEvent,
  target: Element,
): DismissableLayerPointerDownOutsideEvent {
  const point = eventPoint(originalEvent);
  const prevention = preventable();
  return Object.freeze({
    type: "pointer-down-outside" as const,
    reason: "pointer-down-outside" as const,
    target,
    originalEvent,
    pointerType: pointerTypeOf(originalEvent),
    x: point?.x ?? null,
    y: point?.y ?? null,
    altKey: modifier(originalEvent, "altKey"),
    ctrlKey: modifier(originalEvent, "ctrlKey"),
    metaKey: modifier(originalEvent, "metaKey"),
    shiftKey: modifier(originalEvent, "shiftKey"),
    get defaultPrevented() {
      return prevention.defaultPrevented;
    },
    preventDefault: prevention.preventDefault,
  });
}

/** Build an immutable outside focus event that preserves related-target evidence. */
export function createFocusOutsideEvent(
  originalEvent: FocusEvent,
  target: Element,
): DismissableLayerFocusOutsideEvent {
  const prevention = preventable();
  return Object.freeze({
    type: "focus-outside" as const,
    reason: "focus-outside" as const,
    target,
    relatedTarget: eventElement(originalEvent.relatedTarget),
    originalEvent,
    get defaultPrevented() {
      return prevention.defaultPrevented;
    },
    preventDefault: prevention.preventDefault,
  });
}

/** Build an immutable Escape event before dismissal is committed. */
export function createEscapeKeyDownEvent(
  originalEvent: KeyboardEvent,
  target: Element | null,
): DismissableLayerEscapeKeyDownEvent {
  const prevention = preventable();
  return Object.freeze({
    type: "escape-key" as const,
    reason: "escape-key" as const,
    target,
    originalEvent,
    altKey: modifier(originalEvent, "altKey"),
    ctrlKey: modifier(originalEvent, "ctrlKey"),
    metaKey: modifier(originalEvent, "metaKey"),
    shiftKey: modifier(originalEvent, "shiftKey"),
    get defaultPrevented() {
      return prevention.defaultPrevented;
    },
    preventDefault: prevention.preventDefault,
  });
}

/** Build the final immutable dismissal notification. */
export function createDismissEvent(
  reason: DismissableLayerDismissReason,
  originalEvent: Event,
  target: Element | null,
): DismissableLayerDismissEvent {
  return Object.freeze({
    type: "dismiss" as const,
    reason,
    target,
    originalEvent,
  });
}
