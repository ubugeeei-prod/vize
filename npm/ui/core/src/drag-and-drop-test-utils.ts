import { createDragAndDrop } from "./drag-and-drop.ts";
import type {
  DragAndDropController,
  DragAndDropOptions,
  DragLifecycleEvent,
  DragSourceOptions,
  DragSourceProps,
  DragSourceRegistration,
  DropTargetEvent,
  DropTargetOptions,
  DropTargetRegistration,
  DropTargetRect,
} from "./drag-and-drop.ts";

export const dragSourceEventNames: Readonly<Record<keyof DragSourceProps, string>> = {
  onDragstart: "dragstart",
  onFocusout: "focusout",
  onKeydown: "keydown",
  onMousedown: "mousedown",
  onPointerdown: "pointerdown",
  onTouchstart: "touchstart",
};

/** Attach one registration's handlers to a host like a template spread would. */
export function attachSourceProps(host: Element, sourceProps: Readonly<DragSourceProps>): void {
  for (const [property, type] of Object.entries(dragSourceEventNames) as Array<
    [keyof DragSourceProps, string]
  >) {
    host.addEventListener(type, sourceProps[property] as EventListener);
  }
}

/** Build an axis-aligned rectangle in the harness's client coordinates. */
export function rect(top: number, left: number, bottom: number, right: number): DropTargetRect {
  return { top, left, bottom, right };
}

export interface DragHarness {
  readonly controller: DragAndDropController;
  readonly events: DragLifecycleEvent[];
  readonly host: HTMLDivElement;
  readonly source: DragSourceRegistration;
  readonly unmount: () => void;
}

/** Mount one controller with a focusable source handle wired to the body. */
export function mountDragAndDrop(
  options: DragAndDropOptions = {},
  sourceOptions: Partial<DragSourceOptions> = {},
): DragHarness {
  const events: DragLifecycleEvent[] = [];
  const controller = createDragAndDrop({
    ...options,
    onDragStart: (event) => {
      events.push(event);
      options.onDragStart?.(event);
    },
    onDragMove: (event) => {
      events.push(event);
      options.onDragMove?.(event);
    },
    onDragEnd: (event) => {
      events.push(event);
      options.onDragEnd?.(event);
    },
  });
  const host = document.createElement("div");
  host.tabIndex = 0;
  document.body.append(host);
  const source = controller.registerSource({
    key: "card",
    element: () => host,
    payload: { kind: "card", data: { id: 1 } },
    ...sourceOptions,
  });
  attachSourceProps(host, source.sourceProps);
  return {
    controller,
    events,
    host,
    source,
    unmount: () => {
      try {
        controller.dispose();
      } finally {
        host.remove();
      }
    },
  };
}

export interface TargetHarness {
  readonly element: HTMLDivElement;
  readonly events: DropTargetEvent[];
  readonly registration: DropTargetRegistration;
}

/** Register one measurable target; pass a parent to model nested ownership. */
export function addTarget(
  controller: DragAndDropController,
  key: string,
  targetRect: DropTargetRect,
  options: Partial<DropTargetOptions> = {},
  parent: Element | null = null,
): TargetHarness {
  const element = document.createElement("div");
  (parent ?? document.body).append(element);
  const events: DropTargetEvent[] = [];
  const registration = controller.registerTarget({
    key,
    element: () => element,
    getRect: () => targetRect,
    onEnter: (event) => events.push(event),
    onLeave: (event) => events.push(event),
    onMove: (event) => events.push(event),
    onDrop: (event) => events.push(event),
    ...options,
  });
  return { element, events, registration };
}

/** Read the text most recently spoken through the owned live region. */
export function liveRegionText(): string | null {
  const element = document.querySelector('[data-vize-ui="drag-and-drop-live"]');
  return element === null ? null : (element.textContent ?? "").trimEnd();
}

/** Remove any live region left behind by a disposed-free test. */
export function removeLiveRegions(): void {
  for (const element of document.querySelectorAll('[data-vize-ui="drag-and-drop-live"]')) {
    element.remove();
  }
}

/** Keyboard event factory matching template `@keydown` semantics. */
export function keydown(key: string, values: KeyboardEventInit = {}): KeyboardEvent {
  return new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...values });
}
