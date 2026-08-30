import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, ref } from "vue";

import {
  createInteractionModalityTracker,
  useInteractionModality,
} from "./interaction-modality.ts";
import type { InteractionModalityChange } from "./interaction-modality.ts";

test("shares exactly one native listener set and state per document", () => {
  const isolated = document.implementation.createHTMLDocument("shared");
  const additions: string[] = [];
  const removals: string[] = [];
  const addEventListener = isolated.addEventListener.bind(isolated);
  const removeEventListener = isolated.removeEventListener.bind(isolated);
  isolated.addEventListener = ((
    type: string,
    listener: EventListenerOrEventListenerObject,
    options,
  ) => {
    additions.push(type);
    addEventListener(type, listener, options);
  }) as typeof isolated.addEventListener;
  isolated.removeEventListener = ((
    type: string,
    listener: EventListenerOrEventListenerObject,
    options,
  ) => {
    removals.push(type);
    removeEventListener(type, listener, options);
  }) as typeof isolated.removeEventListener;

  const first = createInteractionModalityTracker({ document: isolated });
  const second = createInteractionModalityTracker({ document: isolated });
  assert.deepEqual(additions.sort(), ["click", "keydown", "mousedown", "touchstart"]);

  isolated.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "Tab" }));
  assert.equal(first.modality.value, "keyboard");
  assert.equal(second.modality.value, "keyboard");
  assert.equal(first.setModality("touch"), true);
  assert.equal(second.modality.value, "touch");

  isolated.dispatchEvent(new Event("touchstart", { bubbles: true }));
  isolated.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
  assert.equal(first.modality.value, "touch");
  assert.equal(second.modality.value, "touch");

  first.dispose();
  assert.deepEqual(removals, []);
  second.dispose();
  assert.deepEqual(
    removals.sort((left, right) => left.localeCompare(right)),
    ["click", "keydown", "mousedown", "touchstart"],
  );
});

test("serializes reentrant updates so every peer reaches the same final state", () => {
  const isolated = document.implementation.createHTMLDocument("reentrant");
  const firstChanges: Array<string | null> = [];
  const secondChanges: Array<string | null> = [];
  let first!: ReturnType<typeof createInteractionModalityTracker>;
  first = createInteractionModalityTracker({
    document: isolated,
    onChange(change) {
      firstChanges.push(change.modality);
      if (change.modality === "keyboard") first.setModality("touch");
    },
  });
  const second = createInteractionModalityTracker({
    document: isolated,
    onChange: (change) => secondChanges.push(change.modality),
  });

  isolated.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "Tab" }));
  assert.equal(first.modality.value, "touch");
  assert.equal(second.modality.value, "touch");
  assert.deepEqual(firstChanges, ["keyboard", "touch"]);
  assert.deepEqual(secondChanges, ["keyboard", "touch"]);

  first.dispose();
  second.dispose();
});

test("updates every peer before surfacing a subscriber exception", () => {
  const isolated = document.implementation.createHTMLDocument("errors");
  const failure = new Error("consumer callback failed");
  const failing = createInteractionModalityTracker({
    document: isolated,
    onChange: () => {
      throw failure;
    },
  });
  const peer = createInteractionModalityTracker({ document: isolated });

  assert.throws(() => failing.setModality("keyboard"), failure);
  assert.equal(failing.modality.value, "keyboard");
  assert.equal(peer.modality.value, "keyboard");
  failing.dispose();
  peer.dispose();
});

test("rolls back a failed document adoption without leaking a subscription", () => {
  const isolated = document.implementation.createHTMLDocument("adoption-error");
  const removals: string[] = [];
  const removeEventListener = isolated.removeEventListener.bind(isolated);
  isolated.removeEventListener = ((
    type: string,
    listener: EventListenerOrEventListenerObject,
    options,
  ) => {
    removals.push(type);
    removeEventListener(type, listener, options);
  }) as typeof isolated.removeEventListener;
  const anchor = createInteractionModalityTracker({
    document: isolated,
    initialModality: "pointer",
  });
  const moving = createInteractionModalityTracker({
    document: null,
    onChange: () => {
      throw new Error("reject adoption");
    },
  });

  assert.throws(() => moving.attach(isolated), /reject adoption/);
  assert.equal(moving.document.value, null);
  anchor.dispose();
  assert.deepEqual(
    removals.sort((left, right) => left.localeCompare(right)),
    ["click", "keydown", "mousedown", "touchstart"],
  );
  moving.dispose();
});

test("isolates separate documents and adopts existing state when moving", () => {
  const left = document.implementation.createHTMLDocument("left");
  const right = document.implementation.createHTMLDocument("right");
  const anchor = createInteractionModalityTracker({ document: right, initialModality: "touch" });
  const currentDocument = ref<Document | null>(left);
  const changes: InteractionModalityChange[] = [];
  const moving = createInteractionModalityTracker({
    document: currentDocument,
    initialModality: "pointer",
    onChange: (change) => changes.push(change),
  });

  left.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "Tab" }));
  assert.equal(moving.modality.value, "keyboard");
  assert.equal(anchor.modality.value, "touch");

  currentDocument.value = right;
  assert.equal(moving.document.value, right);
  assert.equal(moving.modality.value, "touch");
  assert.equal(changes.at(-1)?.reason, "document");
  currentDocument.value = null;
  assert.equal(moving.document.value, null);
  assert.equal(moving.modality.value, "touch");

  moving.dispose();
  anchor.dispose();
});

test("supports explicit attach and detach without losing the last value", () => {
  const isolated = document.implementation.createHTMLDocument("attach");
  const tracker = createInteractionModalityTracker({ document: null });

  assert.equal(tracker.attach(isolated), true);
  assert.equal(tracker.attach(isolated), false);
  isolated.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
  assert.equal(tracker.modality.value, "pointer");
  assert.equal(tracker.detach(), true);
  assert.equal(tracker.detach(), false);
  assert.equal(tracker.modality.value, "pointer");
  tracker.dispose();
});

test("disposal is idempotent and rejects later mutation", () => {
  const tracker = createInteractionModalityTracker({ document: null });
  tracker.dispose();
  tracker.dispose();

  assert.throws(() => tracker.attach(document), /VIZE_UI_INTERACTION_MODALITY_DISPOSED/);
  assert.throws(() => tracker.detach(), /VIZE_UI_INTERACTION_MODALITY_DISPOSED/);
  assert.throws(() => tracker.setModality("keyboard"), /VIZE_UI_INTERACTION_MODALITY_DISPOSED/);
});

test("the composable requires and follows a Vue effect scope", () => {
  assert.throws(
    () => useInteractionModality({ document: null }),
    /VIZE_UI_INTERACTION_MODALITY_SETUP/,
  );

  const scope = effectScope();
  const tracker = scope.run(() => useInteractionModality({ document }))!;
  assert.equal(tracker.document.value, document);
  scope.stop();
  assert.equal(tracker.document.value, null);
  assert.throws(() => tracker.setModality("pointer"), /VIZE_UI_INTERACTION_MODALITY_DISPOSED/);
});
