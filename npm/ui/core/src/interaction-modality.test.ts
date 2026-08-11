import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createInteractionModalityTracker, isElementFocusVisible } from "./interaction-modality.ts";
import type { InteractionModalityChange } from "./interaction-modality.ts";

function dispatchKeyboard(
  target: Document,
  key: string,
  init: KeyboardEventInit = {},
): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { bubbles: true, key, ...init });
  target.dispatchEvent(event);
  return event;
}

function dispatchPointer(
  target: Document,
  pointerType: string,
  init: PointerEventInit = {},
): PointerEvent {
  const event = new PointerEvent("pointerdown", {
    bubbles: true,
    pointerType,
    ...init,
  });
  target.dispatchEvent(event);
  return event;
}

test("is inert without a document and remains manually controllable during SSR", () => {
  const changes: InteractionModalityChange[] = [];
  const tracker = createInteractionModalityTracker({
    document: null,
    onChange: (change) => changes.push(change),
  });

  assert.equal(tracker.document.value, null);
  assert.equal(tracker.modality.value, null);
  assert.equal(tracker.isFocusVisible.value, false);
  assert.equal(tracker.setModality("keyboard"), true);
  assert.equal(tracker.isFocusVisible.value, true);
  assert.equal(tracker.setModality("keyboard"), false);
  assert.equal(tracker.setModality(null), true);
  assert.deepEqual(
    changes.map(({ modality, previousModality, reason, document }) => ({
      modality,
      previousModality,
      reason,
      document,
    })),
    [
      { modality: "keyboard", previousModality: null, reason: "manual", document: null },
      { modality: null, previousModality: "keyboard", reason: "manual", document: null },
    ],
  );
  assert.ok(changes.every(Object.isFrozen));
  tracker.dispose();
});

test("classifies keyboard, pointer, touch, and virtual intent", () => {
  const changes: InteractionModalityChange[] = [];
  const tracker = createInteractionModalityTracker({
    document,
    onChange: (change) => changes.push(change),
  });

  const keyboardEvent = dispatchKeyboard(document, "Tab");
  assert.equal(tracker.modality.value, "keyboard");
  assert.equal(changes.at(-1)?.originalEvent, keyboardEvent);

  dispatchPointer(document, "pen");
  assert.equal(tracker.modality.value, "pointer");
  dispatchPointer(document, "touch");
  assert.equal(tracker.modality.value, "touch");
  dispatchPointer(document, "vendor-device");
  assert.equal(tracker.modality.value, "pointer");

  const virtualPointer = dispatchPointer(document, "", { pointerId: -1 });
  assert.equal(tracker.modality.value, "virtual");
  assert.equal(changes.at(-1)?.originalEvent, virtualPointer);
  tracker.dispose();
});

test("keeps keyboard-generated coordinate-free clicks keyboard-visible", () => {
  const tracker = createInteractionModalityTracker({ document });
  dispatchKeyboard(document, "Enter");
  document.dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 0 }));
  assert.equal(tracker.modality.value, "keyboard");

  dispatchPointer(document, "mouse");
  document.dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 1 }));
  assert.equal(tracker.modality.value, "pointer");
  document.dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 0 }));
  assert.equal(tracker.modality.value, "virtual");
  tracker.dispose();
});

test("ignores composition, modifier-only keys, and modified shortcuts", () => {
  const tracker = createInteractionModalityTracker({ document, initialModality: "pointer" });

  dispatchKeyboard(document, "Control");
  dispatchKeyboard(document, "k", { metaKey: true });
  dispatchKeyboard(document, "Dead", { isComposing: true });
  assert.equal(tracker.modality.value, "pointer");

  dispatchKeyboard(document, "Tab", { shiftKey: true });
  assert.equal(tracker.modality.value, "keyboard");
  tracker.dispose();
});

test("rejects invalid runtime documents and modalities", () => {
  assert.throws(
    () => createInteractionModalityTracker({ document: {} as Document }),
    /VIZE_UI_INTERACTION_MODALITY_DOCUMENT/,
  );
  assert.throws(
    () =>
      createInteractionModalityTracker({
        document: null,
        initialModality: "mouse" as "keyboard",
      }),
    /VIZE_UI_INTERACTION_MODALITY_VALUE/,
  );
  const tracker = createInteractionModalityTracker({ document: null });
  assert.throws(
    () => tracker.setModality("mouse" as "keyboard"),
    /VIZE_UI_INTERACTION_MODALITY_VALUE/,
  );
  tracker.dispose();
});

test("defers to native focus-visible semantics for document and shadow roots", () => {
  const button = document.createElement("button");
  const unfocused = document.createElement("button");
  const host = document.createElement("div");
  const shadow = host.attachShadow({ mode: "open" });
  const shadowButton = document.createElement("button");
  shadow.append(shadowButton);
  document.body.append(button, unfocused, host);

  button.focus();
  assert.equal(isElementFocusVisible(button, "pointer"), button.matches(":focus-visible"));
  assert.equal(isElementFocusVisible(unfocused, "keyboard"), false);
  shadowButton.focus();
  assert.equal(
    isElementFocusVisible(shadowButton, "pointer"),
    shadowButton.matches(":focus-visible"),
  );

  button.remove();
  unfocused.remove();
  host.remove();
});

test("falls back to modality only when focus-visible is unsupported", () => {
  const root = { activeElement: null as Element | null };
  const element = {
    getRootNode: () => root,
    matches: () => {
      throw new DOMException("unsupported selector", "SyntaxError");
    },
    ownerDocument: { activeElement: null },
  } as unknown as Element;
  root.activeElement = element;

  assert.equal(isElementFocusVisible(element, "keyboard"), true);
  assert.equal(isElementFocusVisible(element, "virtual"), true);
  assert.equal(isElementFocusVisible(element, "pointer"), false);
  assert.equal(isElementFocusVisible(element, "touch"), false);
  assert.equal(isElementFocusVisible(null, "keyboard"), false);
});
