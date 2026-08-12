import { createMove } from "./move.ts";
import type { MoveController, MoveEvent, MoveOptions, MoveProps } from "./move.ts";

export const moveEventNames: Readonly<Record<keyof MoveProps, string>> = {
  onDragstart: "dragstart",
  onKeydown: "keydown",
  onMousedown: "mousedown",
  onPointercancel: "pointercancel",
  onPointerdown: "pointerdown",
  onTouchcancel: "touchcancel",
  onTouchend: "touchend",
  onTouchmove: "touchmove",
  onTouchstart: "touchstart",
};

export interface MoveHarness {
  readonly controller: MoveController;
  readonly events: MoveEvent[];
  readonly host: HTMLDivElement;
  readonly unmount: () => void;
}

/** Capture listener failures in both browsers and propagating test DOMs. */
export function dispatchReportingError(dispatch: () => void): unknown {
  let reportedError: unknown;
  const captureReportedError = (event: ErrorEvent) => {
    reportedError = event.error;
    event.preventDefault();
  };
  window.addEventListener("error", captureReportedError);
  try {
    dispatch();
  } catch (error) {
    reportedError ??= error;
  } finally {
    window.removeEventListener("error", captureReportedError);
  }
  return reportedError;
}

export function mountMove(options: MoveOptions = {}): MoveHarness {
  const host = document.createElement("div");
  host.tabIndex = 0;
  document.body.append(host);
  const events: MoveEvent[] = [];
  const controller = createMove({
    ...options,
    onMoveStart: (event) => {
      events.push(event);
      options.onMoveStart?.(event);
    },
    onMove: (event) => {
      events.push(event);
      options.onMove?.(event);
    },
    onMoveEnd: (event) => {
      events.push(event);
      options.onMoveEnd?.(event);
    },
  });
  for (const [property, type] of Object.entries(moveEventNames) as Array<
    [keyof MoveProps, string]
  >) {
    host.addEventListener(type, controller.moveProps[property] as EventListener);
  }
  return {
    controller,
    events,
    host,
    unmount: () => {
      try {
        controller.dispose();
      } finally {
        host.remove();
      }
    },
  };
}

export function pointer(
  type: string,
  x: number,
  y: number,
  values: Partial<PointerEventInit> = {},
): PointerEvent {
  const event = new PointerEvent(type, {
    bubbles: true,
    button: 0,
    clientX: x,
    clientY: y,
    isPrimary: true,
    pointerId: 7,
    pointerType: "mouse",
    ...values,
  });
  Object.defineProperties(event, {
    pageX: { value: x },
    pageY: { value: y },
  });
  return event;
}

export function mouse(type: string, x: number, y: number, values: MouseEventInit = {}): MouseEvent {
  const event = new MouseEvent(type, {
    bubbles: true,
    button: 0,
    clientX: x,
    clientY: y,
    ...values,
  });
  Object.defineProperties(event, {
    pageX: { value: x },
    pageY: { value: y },
  });
  return event;
}

export function touchEvent(
  type: string,
  values: Array<{ clientX: number; clientY: number; identifier: number }>,
  timeStamp?: number,
): TouchEvent {
  const event = new Event(type, { bubbles: true, cancelable: true }) as TouchEvent;
  const touches = Object.assign(
    values.map((value) => ({ ...value, pageX: value.clientX, pageY: value.clientY })),
    { item: (index: number) => touches[index] ?? null },
  ) as unknown as TouchList;
  Object.defineProperty(event, "changedTouches", { value: touches });
  Object.defineProperty(event, "view", { value: null });
  if (timeStamp !== undefined) Object.defineProperty(event, "timeStamp", { value: timeStamp });
  return event;
}
