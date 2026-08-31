import type {
  LongPressController,
  LongPressEvent,
  LongPressOptions,
  LongPressProps,
} from "./long-press.ts";
import { createLongPress } from "./long-press.ts";
import type { PressEvent } from "../press/press.ts";

const eventNames: Readonly<Record<keyof Omit<LongPressProps, `aria-${string}`>, string>> = {
  onClick: "click",
  onContextmenu: "contextmenu",
  onDragstart: "dragstart",
  onKeydown: "keydown",
  onKeyup: "keyup",
  onMousedown: "mousedown",
  onMousemove: "mousemove",
  onMouseup: "mouseup",
  onPointercancel: "pointercancel",
  onPointerdown: "pointerdown",
  onPointermove: "pointermove",
  onPointerup: "pointerup",
  onTouchcancel: "touchcancel",
  onTouchend: "touchend",
  onTouchmove: "touchmove",
  onTouchstart: "touchstart",
};

export interface LongPressHarness {
  readonly controller: LongPressController;
  readonly events: Array<LongPressEvent | PressEvent>;
  readonly host: HTMLButtonElement;
  readonly unmount: () => void;
}

export function mountLongPress(options: LongPressOptions = {}): LongPressHarness {
  const host = document.createElement("button");
  document.body.append(host);
  const events: Array<LongPressEvent | PressEvent> = [];
  const controller = createLongPress({
    ...options,
    onLongPressStart: (event) => {
      events.push(event);
      options.onLongPressStart?.(event);
    },
    onLongPress: (event) => {
      events.push(event);
      options.onLongPress?.(event);
    },
    onLongPressEnd: (event) => {
      events.push(event);
      options.onLongPressEnd?.(event);
    },
    onPress: (event) => {
      events.push(event);
      options.onPress?.(event);
    },
  });
  for (const [property, type] of Object.entries(eventNames) as Array<
    [keyof typeof eventNames, string]
  >) {
    host.addEventListener(type, controller.longPressProps[property] as EventListener);
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
  pointerType = "mouse",
  values: Partial<PointerEventInit> = {},
): PointerEvent {
  return new PointerEvent(type, {
    bubbles: true,
    button: 0,
    clientX: 12,
    clientY: 18,
    isPrimary: true,
    pointerId: 19,
    pointerType,
    ...values,
  });
}

export async function elapseThreshold(): Promise<void> {
  await new Promise<void>((resolve) => setTimeout(resolve, 5));
}
