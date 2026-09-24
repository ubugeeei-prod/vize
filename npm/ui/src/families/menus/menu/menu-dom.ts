import type {
  Placement,
  PlacementAlign,
  PlacementSide,
} from "../../overlays/positioner/positioner.ts";
import type { MenuLevelContextValue, MenuPoint } from "./menu-context.ts";
import type { MenuDirection } from "./menu-types.ts";

/** Narrow an element to an `HTMLElement` of its own realm without casting. */
export function toHTMLElement(value: Element | null | undefined): HTMLElement | null {
  const View = value?.ownerDocument.defaultView;
  return value && View && value instanceof View.HTMLElement ? value : null;
}

/** Narrow an event target to an `Element` of its own realm without casting. */
export function eventTargetElement(target: EventTarget | null, scope: Element): Element | null {
  const View = scope.ownerDocument.defaultView;
  return View && target instanceof View.Element ? target : null;
}

/** Focus an element when it is focusable in its own realm. */
export function focusElement(value: Element | null | undefined): HTMLElement | null {
  const element = toHTMLElement(value);
  element?.focus({ preventScroll: true });
  return element;
}

/** Whether `target` lives inside the rendered content of `level` or any ancestor level. */
export function isInsideLevelChain(level: MenuLevelContextValue | null, target: Element): boolean {
  for (let current = level; current; current = current.parent) {
    if (current.contentElement.value?.contains(target)) return true;
  }
  return false;
}

/** Whether a pointer event came from a mouse-like device that hovers. */
export function isHoverPointer(event: PointerEvent): boolean {
  return event.pointerType === "mouse" || event.pointerType === "pen" || event.pointerType === "";
}

/** Viewport point of a pointer event. */
export function pointerPoint(event: PointerEvent | MouseEvent): MenuPoint {
  return { x: event.clientX, y: event.clientY };
}

/** Key that opens a submenu (or moves forward in a menubar) for a reading direction. */
export function forwardKey(dir: MenuDirection): "ArrowLeft" | "ArrowRight" {
  return dir === "rtl" ? "ArrowLeft" : "ArrowRight";
}

/** Key that closes a submenu (or moves backward in a menubar) for a reading direction. */
export function backwardKey(dir: MenuDirection): "ArrowLeft" | "ArrowRight" {
  return dir === "rtl" ? "ArrowRight" : "ArrowLeft";
}

const placements: readonly Placement[] = [
  "bottom",
  "bottom-center",
  "bottom-end",
  "bottom-start",
  "left",
  "left-center",
  "left-end",
  "left-start",
  "right",
  "right-center",
  "right-end",
  "right-start",
  "top",
  "top-center",
  "top-end",
  "top-start",
];

/** Resolve a positioner slot placement, falling back to the requested placement. */
export function resolvePlacement(value: unknown, fallback: Placement): Placement {
  return placements.find((placement) => placement === value) ?? fallback;
}

/** Side token of a placement. */
export function sideOf(value: Placement): PlacementSide {
  if (value.startsWith("top")) return "top";
  if (value.startsWith("left")) return "left";
  if (value.startsWith("right")) return "right";
  return "bottom";
}

/** Alignment token of a placement. */
export function alignOf(value: Placement): PlacementAlign {
  if (value.endsWith("-start")) return "start";
  if (value.endsWith("-end")) return "end";
  return "center";
}

/** Default submenu placement: beside the trigger, on the reading-direction side. */
export function submenuPlacement(dir: MenuDirection): Placement {
  return dir === "rtl" ? "left-start" : "right-start";
}
