import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  DRAG_TRANSFER_TYPE,
  readClipboardTransfer,
  readDragTransfer,
  writeClipboardTransfer,
  writeDragTransfer,
} from "./drag-and-drop.ts";

test("data-transfer adapters round-trip typed payloads with plain text", () => {
  const dataTransfer = new DataTransfer();
  writeDragTransfer(dataTransfer, {
    kind: "task",
    data: { id: 7, title: "Ship" },
    plainText: "Ship",
  });
  assert.equal(dataTransfer.getData("text/plain"), "Ship");
  const restored = readDragTransfer<{ id: number; title: string }>(dataTransfer);
  assert.deepEqual(restored, { kind: "task", data: { id: 7, title: "Ship" }, plainText: "Ship" });
  assert.ok(Object.isFrozen(restored));
});

test("data-transfer adapters omit plain text unless provided", () => {
  const dataTransfer = new DataTransfer();
  writeDragTransfer(dataTransfer, { kind: "task", data: [1, 2] });
  assert.equal(dataTransfer.getData("text/plain"), "");
  assert.deepEqual(readDragTransfer(dataTransfer), { kind: "task", data: [1, 2] });
});

test("malformed or foreign transfers read as null instead of throwing", () => {
  const empty = new DataTransfer();
  assert.equal(readDragTransfer(empty), null);
  const foreign = new DataTransfer();
  foreign.setData("text/plain", "just text");
  assert.equal(readDragTransfer(foreign), null);
  const corrupt = new DataTransfer();
  corrupt.setData(DRAG_TRANSFER_TYPE, "{not json");
  assert.equal(readDragTransfer(corrupt), null);
  const wrongShape = new DataTransfer();
  wrongShape.setData(DRAG_TRANSFER_TYPE, JSON.stringify({ kind: 5, data: 1 }));
  assert.equal(readDragTransfer(wrongShape), null);
});

test("transfer writers validate the payload shape eagerly", () => {
  const dataTransfer = new DataTransfer();
  assert.throws(
    () => writeDragTransfer(dataTransfer, { kind: "", data: 1 }),
    /VIZE_UI_DRAG_AND_DROP_TRANSFER/,
  );
  assert.throws(
    () => writeDragTransfer(dataTransfer, { kind: "task", data: 1, plainText: 2 as never }),
    /VIZE_UI_DRAG_AND_DROP_TRANSFER/,
  );
});

function clipboardEvent(type: string, dataTransfer: DataTransfer | null): ClipboardEvent {
  const event = new Event(type, { bubbles: true, cancelable: true }) as ClipboardEvent;
  Object.defineProperty(event, "clipboardData", { value: dataTransfer });
  return event;
}

test("clipboard adapters bridge copy and paste through the same format", () => {
  const dataTransfer = new DataTransfer();
  const copy = clipboardEvent("copy", dataTransfer);
  assert.equal(
    writeClipboardTransfer(copy, { kind: "task", data: { id: 3 }, plainText: "Task 3" }),
    true,
  );
  assert.equal(copy.defaultPrevented, true, "the default copy must not overwrite the payload");
  const paste = clipboardEvent("paste", dataTransfer);
  assert.deepEqual(readClipboardTransfer<{ id: number }>(paste), {
    kind: "task",
    data: { id: 3 },
    plainText: "Task 3",
  });
});

test("clipboard adapters degrade to null without writable clipboard data", () => {
  const detached = clipboardEvent("copy", null);
  assert.equal(writeClipboardTransfer(detached, { kind: "task", data: 1 }), false);
  assert.equal(detached.defaultPrevented, false);
  assert.equal(readClipboardTransfer(clipboardEvent("paste", null)), null);
});
