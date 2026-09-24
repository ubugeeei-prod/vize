import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { matchesAccept, useDropZone } from "./use-drop-zone.ts";
import type { DataTransferItemLike, DataTransferLike } from "./use-drop-zone.ts";

class FakeTransfer implements DataTransferLike {
  readonly items: DataTransferItemLike[];
  readonly files: File[];
  dropEffect = "copy";

  constructor(files: File[]) {
    this.files = files;
    this.items = files.map((file) => ({ kind: "file", type: file.type }));
  }
}

class FakeDragEvent extends Event {
  readonly dataTransfer: FakeTransfer | null;

  constructor(type: string, transfer: FakeTransfer | null) {
    super(type, { cancelable: true });
    this.dataTransfer = transfer;
  }
}

class Zone extends EventTarget {
  listeners = 0;

  override addEventListener(type: string, listener: EventListener): void {
    this.listeners += 1;
    super.addEventListener(type, listener);
  }

  override removeEventListener(type: string, listener: EventListener): void {
    this.listeners -= 1;
    super.removeEventListener(type, listener);
  }

  drag(type: string, transfer: FakeTransfer | null): FakeDragEvent {
    const event = new FakeDragEvent(type, transfer);
    this.dispatchEvent(event);
    return event;
  }
}

const png = new File(["x"], "photo.png", { type: "image/png" });
const pdf = new File(["x"], "doc.pdf", { type: "application/pdf" });

void test("matchesAccept supports MIME, wildcard, and extension patterns", () => {
  assert.equal(matchesAccept([], pdf), true);
  assert.equal(matchesAccept(["image/*"], png), true);
  assert.equal(matchesAccept(["image/*"], pdf), false);
  assert.equal(matchesAccept(["application/pdf"], pdf), true);
  assert.equal(matchesAccept([".PNG"], png), true);
  assert.equal(matchesAccept([".png"], pdf), false);
  assert.equal(matchesAccept([".png"], { name: "", type: "image/png" }), true);
  assert.equal(matchesAccept(["*"], pdf), true);
});

void test("tracks nested enter and leave events without flicker", () => {
  const zone = new Zone();
  const dropZone = useDropZone(zone);
  const transfer = new FakeTransfer([png]);

  const enter = zone.drag("dragenter", transfer);
  assert.equal(dropZone.isOverDropZone.value, true);
  assert.equal(dropZone.accepted.value, true);
  assert.equal(enter.defaultPrevented, true);
  zone.drag("dragenter", transfer);
  zone.drag("dragleave", transfer);
  assert.equal(dropZone.isOverDropZone.value, true, "leaving a child keeps the state");
  zone.drag("dragleave", transfer);
  assert.equal(dropZone.isOverDropZone.value, false);
});

void test("rejects drags with unaccepted types and shows no drop effect", () => {
  const zone = new Zone();
  const dropZone = useDropZone(zone, { accept: ["image/*"] });
  const transfer = new FakeTransfer([pdf]);

  const enter = zone.drag("dragenter", transfer);
  assert.equal(dropZone.accepted.value, false);
  assert.equal(enter.defaultPrevented, false);
  const over = zone.drag("dragover", transfer);
  assert.equal(over.defaultPrevented, false);
  assert.equal(transfer.dropEffect, "none");

  const drop = zone.drag("drop", transfer);
  assert.equal(drop.defaultPrevented, false);
  assert.equal(dropZone.files.value, null);
});

void test("filters dropped files by extension and multiplicity", () => {
  const zone = new Zone();
  const drops: (readonly File[] | null)[] = [];
  const multiple = ref(true);
  const dropZone = useDropZone(zone, {
    accept: [".png"],
    multiple,
    onDrop: (files) => drops.push(files),
  });

  zone.drag("dragenter", new FakeTransfer([png]));
  const drop = zone.drag("drop", new FakeTransfer([png]));
  assert.equal(drop.defaultPrevented, true);
  assert.deepEqual(dropZone.files.value, [png]);
  assert.equal(dropZone.isOverDropZone.value, false);

  zone.drag("drop", new FakeTransfer([png, pdf]));
  assert.equal(dropZone.files.value, null, "a partially rejected drop accepts nothing");

  multiple.value = false;
  zone.drag("drop", new FakeTransfer([png, png]));
  assert.deepEqual(drops, [[png], null, null]);
});

void test("supports predicate acceptance and unhandled prevention", () => {
  const zone = new Zone();
  const dropZone = useDropZone(zone, {
    accept: (types) => types.every((type) => type === "application/pdf"),
    preventDefaultForUnhandled: true,
  });

  const rejected = zone.drag("dragover", new FakeTransfer([png]));
  assert.equal(rejected.defaultPrevented, true, "prevents the browser from opening the file");
  zone.drag("drop", new FakeTransfer([pdf]));
  assert.deepEqual(dropZone.files.value, [pdf]);
});

void test("calls lifecycle callbacks and ignores events without data", () => {
  const zone = new Zone();
  const seen: string[] = [];
  const dropZone = useDropZone(zone, {
    onEnter: () => seen.push("enter"),
    onOver: () => seen.push("over"),
    onLeave: () => seen.push("leave"),
  });

  zone.drag("dragenter", null);
  assert.equal(dropZone.accepted.value, false);
  zone.drag("dragover", null);
  zone.drag("dragleave", null);
  assert.deepEqual(seen, ["enter", "over", "leave"]);
});

void test("rebinds reactive targets and removes listeners with the scope", async () => {
  const first = new Zone();
  const second = new Zone();
  const target = ref<Zone | null>(first);
  const scope = effectScope();
  scope.run(() => useDropZone(target));
  assert.equal(first.listeners, 4);

  target.value = second;
  await nextTick();
  assert.equal(first.listeners, 0);
  assert.equal(second.listeners, 4);
  scope.stop();
  assert.equal(second.listeners, 0);
});

void test("server rendering attaches nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const dropZone = useDropZone(() => null, { accept: ["image/*"] });
    return { over: dropZone.isOverDropZone, files: dropZone.files };
  });
  assert.equal(state, '{"over":false,"files":null}');
});
