import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createLongPress } from "./long-press.ts";
import type { LongPressEvent } from "./long-press.ts";

function touchEvent(
  type: string,
  values: Array<{ clientX: number; clientY: number; identifier: number }>,
): TouchEvent {
  const event = new Event(type, { bubbles: true }) as TouchEvent;
  const touches = Object.assign(values, {
    item: (index: number) => touches[index] ?? null,
  }) as unknown as TouchList;
  Object.defineProperty(event, "changedTouches", { value: touches });
  Object.defineProperty(event, "view", { value: null });
  return event;
}

test("normalizes a legacy touch hold and owns only its initiating contact", async () => {
  const isolated = document.implementation.createHTMLDocument("legacy long press");
  const host = isolated.createElement("button");
  isolated.body.append(host);
  const events: LongPressEvent[] = [];
  const controller = createLongPress({
    threshold: 0,
    onLongPressStart: (event) => events.push(event),
    onLongPress: (event) => events.push(event),
    onLongPressEnd: (event) => events.push(event),
  });
  host.addEventListener("touchstart", controller.longPressProps.onTouchstart);
  host.addEventListener("touchend", controller.longPressProps.onTouchend);
  host.addEventListener("touchcancel", controller.longPressProps.onTouchcancel);
  host.addEventListener("contextmenu", controller.longPressProps.onContextmenu);

  host.dispatchEvent(touchEvent("touchstart", [{ identifier: 31, clientX: 4, clientY: 8 }]));
  await new Promise<void>((resolve) => setTimeout(resolve, 5));
  isolated.body.dispatchEvent(
    touchEvent("touchend", [{ identifier: 99, clientX: 100, clientY: 200 }]),
  );
  assert.equal(controller.isLongPressed.value, true);
  isolated.body.dispatchEvent(
    touchEvent("touchend", [
      { identifier: 99, clientX: 100, clientY: 200 },
      { identifier: 31, clientX: 9, clientY: 12 },
    ]),
  );

  assert.deepEqual(
    events.map(({ type }) => type),
    ["longpressstart", "longpress", "longpressend"],
  );
  assert.ok(events.every(({ pointerType }) => pointerType === "touch"));
  assert.deepEqual([events[0]?.x, events[0]?.y], [4, 8]);
  assert.deepEqual([events[2]?.x, events[2]?.y], [9, 12]);
  controller.dispose();
});
