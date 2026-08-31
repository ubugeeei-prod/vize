import assert from "node:assert/strict";

import { effectScope, ref } from "vue";
import { test } from "vite-plus/test";

import { createDismissableLayer, useDismissableLayer } from "./dismissable-layer.ts";
import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerFocusOutsideEvent,
  DismissableLayerInteractOutsideEvent,
  DismissableLayerOptions,
  DismissableLayerPointerDownOutsideEvent,
} from "./dismissable-layer.ts";

interface DismissableLayerHarness {
  readonly after: HTMLButtonElement;
  readonly before: HTMLButtonElement;
  readonly controller: ReturnType<typeof createDismissableLayer>;
  readonly dismisses: DismissableLayerDismissEvent[];
  readonly escapeEvents: DismissableLayerEscapeKeyDownEvent[];
  readonly focusEvents: DismissableLayerFocusOutsideEvent[];
  readonly inside: HTMLButtonElement;
  readonly interactions: DismissableLayerInteractOutsideEvent[];
  readonly pointerEvents: DismissableLayerPointerDownOutsideEvent[];
  readonly root: HTMLDivElement;
  readonly rootRef: ReturnType<typeof ref<Element | null>>;
  readonly unmount: () => void;
}

function mountDismissableLayer(
  options: Omit<DismissableLayerOptions, "root"> = {},
  activate = true,
): DismissableLayerHarness {
  const before = document.createElement("button");
  const root = document.createElement("div");
  const inside = document.createElement("button");
  const after = document.createElement("button");
  before.textContent = "before";
  inside.textContent = "inside";
  after.textContent = "after";
  root.append(inside);
  document.body.append(before, root, after);
  const rootRef = ref<Element | null>(root);
  const pointerEvents: DismissableLayerPointerDownOutsideEvent[] = [];
  const focusEvents: DismissableLayerFocusOutsideEvent[] = [];
  const interactions: DismissableLayerInteractOutsideEvent[] = [];
  const escapeEvents: DismissableLayerEscapeKeyDownEvent[] = [];
  const dismisses: DismissableLayerDismissEvent[] = [];
  const controller = createDismissableLayer({
    ...options,
    root: rootRef,
    onDismiss(event) {
      dismisses.push(event);
      options.onDismiss?.(event);
    },
    onEscapeKeyDown(event) {
      escapeEvents.push(event);
      options.onEscapeKeyDown?.(event);
    },
    onFocusOutside(event) {
      focusEvents.push(event);
      options.onFocusOutside?.(event);
    },
    onInteractOutside(event) {
      interactions.push(event);
      options.onInteractOutside?.(event);
    },
    onPointerDownOutside(event) {
      pointerEvents.push(event);
      options.onPointerDownOutside?.(event);
    },
  });
  if (activate) controller.activate();
  return {
    after,
    before,
    controller,
    dismisses,
    escapeEvents,
    focusEvents,
    inside,
    interactions,
    pointerEvents,
    root,
    rootRef,
    unmount: () => {
      controller.dispose();
      before.remove();
      root.remove();
      after.remove();
    },
  };
}

function dispatchPointerDown(target: Element): Event {
  const ViewPointer = target.ownerDocument.defaultView?.PointerEvent;
  const event = ViewPointer
    ? new ViewPointer("pointerdown", {
        bubbles: true,
        cancelable: true,
        clientX: 12,
        clientY: 34,
        composed: true,
        pointerType: "mouse",
      })
    : new MouseEvent("pointerdown", {
        bubbles: true,
        cancelable: true,
        clientX: 12,
        clientY: 34,
      });
  target.dispatchEvent(event);
  return event;
}

function dispatchFocusIn(target: Element, relatedTarget: Element | null = null): FocusEvent {
  const event = new FocusEvent("focusin", {
    bubbles: true,
    composed: true,
    relatedTarget,
  });
  target.dispatchEvent(event);
  return event;
}

function dispatchEscape(target: Element, options: { readonly composing?: boolean } = {}) {
  const event = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "Escape",
  });
  if (options.composing) Object.defineProperty(event, "isComposing", { value: true });
  target.dispatchEvent(event);
  return event;
}

async function flushMutationTurn(): Promise<void> {
  await new Promise<void>((resolve) => queueMicrotask(resolve));
  await new Promise<void>((resolve) => queueMicrotask(resolve));
}

test("outside pointer evidence is immutable, preventable, and routed before dismissal", () => {
  const harness = mountDismissableLayer({
    onPointerDownOutside(event) {
      event.preventDefault();
    },
  });
  dispatchPointerDown(harness.inside);
  assert.deepEqual(harness.pointerEvents, []);

  const nativeEvent = dispatchPointerDown(harness.after);
  assert.equal(harness.pointerEvents.length, 1);
  assert.equal(harness.interactions.length, 1);
  assert.equal(harness.dismisses.length, 0);
  const event = harness.pointerEvents[0]!;
  assert.ok(Object.isFrozen(event));
  assert.equal(event.defaultPrevented, true);
  assert.equal(event.target, harness.after);
  assert.equal(event.originalEvent, nativeEvent);
  assert.equal(event.pointerType, "mouse");
  assert.equal(event.x, 12);
  assert.equal(event.y, 34);
  harness.unmount();
});

test("outside focus and Escape request dismissal when not prevented", () => {
  const harness = mountDismissableLayer();
  dispatchFocusIn(harness.inside, harness.before);
  assert.equal(harness.dismisses.length, 0);

  const focusEvent = dispatchFocusIn(harness.after, harness.inside);
  assert.equal(harness.focusEvents.length, 1);
  assert.equal(harness.interactions.length, 1);
  assert.equal(harness.focusEvents[0]?.relatedTarget, harness.inside);
  assert.deepEqual(harness.dismisses.at(-1), {
    type: "dismiss",
    reason: "focus-outside",
    target: harness.after,
    originalEvent: focusEvent,
  });

  const escapeEvent = dispatchEscape(harness.inside);
  assert.equal(harness.escapeEvents.length, 1);
  assert.equal(harness.dismisses.at(-1)?.reason, "escape-key");
  assert.equal(harness.dismisses.at(-1)?.originalEvent, escapeEvent);

  dispatchEscape(harness.inside, { composing: true });
  assert.equal(harness.escapeEvents.length, 1);
  const prevented = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "Escape",
  });
  prevented.preventDefault();
  harness.inside.dispatchEvent(prevented);
  assert.equal(harness.escapeEvents.length, 1);
  harness.unmount();
});

test("the topmost connected layer alone receives outside interactions", () => {
  const parent = mountDismissableLayer();
  const child = mountDismissableLayer();
  assert.equal(parent.controller.isTopLayer.value, false);
  assert.equal(child.controller.isTopLayer.value, true);

  dispatchPointerDown(parent.inside);
  assert.equal(parent.dismisses.length, 0);
  assert.equal(child.dismisses.length, 1);
  assert.equal(child.dismisses[0]?.target, parent.inside);

  child.controller.deactivate();
  assert.equal(parent.controller.isTopLayer.value, true);
  dispatchPointerDown(parent.after);
  assert.equal(parent.dismisses.length, 1);
  child.unmount();
  parent.unmount();
});

test("reactive and imperative branches preserve portalled content", () => {
  const portal = document.createElement("div");
  const portalButton = document.createElement("button");
  portal.append(portalButton);
  const branch = document.createElement("div");
  const branchButton = document.createElement("button");
  branch.append(branchButton);
  document.body.append(portal, branch);
  const harness = mountDismissableLayer({ branches: [portal] });

  dispatchPointerDown(portalButton);
  assert.equal(harness.dismisses.length, 0);

  const release = harness.controller.registerBranch(branch);
  dispatchFocusIn(branchButton, harness.inside);
  assert.equal(harness.dismisses.length, 0);

  release();
  dispatchFocusIn(branchButton, harness.inside);
  assert.equal(harness.dismisses.length, 1);
  release();
  harness.unmount();
  portal.remove();
  branch.remove();
});

test("reactive enablement, modality switches, and root migration recompute synchronously", () => {
  const enabled = ref(true);
  const outsidePointerDown = ref(false);
  const outsideFocus = ref(true);
  const escapeKey = ref(false);
  const harness = mountDismissableLayer({
    enabled,
    escapeKey,
    outsideFocus,
    outsidePointerDown,
  });

  dispatchPointerDown(harness.after);
  assert.equal(harness.dismisses.length, 0);
  outsidePointerDown.value = true;
  dispatchPointerDown(harness.after);
  assert.equal(harness.dismisses.length, 1);
  escapeKey.value = false;
  dispatchEscape(harness.inside);
  assert.equal(harness.dismisses.length, 1);
  escapeKey.value = true;
  dispatchEscape(harness.inside);
  assert.equal(harness.dismisses.length, 2);
  enabled.value = false;
  assert.equal(harness.controller.isTopLayer.value, false);
  dispatchFocusIn(harness.after, harness.inside);
  assert.equal(harness.dismisses.length, 2);

  const frame = document.createElement("iframe");
  document.body.append(frame);
  const frameDocument = frame.contentDocument;
  assert.ok(frameDocument);
  const frameRoot = frameDocument.createElement("div");
  const frameOutside = frameDocument.createElement("button");
  frameDocument.body.append(frameRoot, frameOutside);
  enabled.value = true;
  harness.rootRef.value = frameRoot;
  dispatchFocusIn(harness.after, harness.inside);
  assert.equal(harness.dismisses.length, 2);
  dispatchFocusIn(frameOutside, frameRoot);
  assert.equal(harness.dismisses.length, 3);

  harness.unmount();
  frame.remove();
});

test("disconnected top layers lose ownership and restore the parent stack owner", async () => {
  const parent = mountDismissableLayer();
  const child = mountDismissableLayer();
  assert.equal(child.controller.isTopLayer.value, true);
  child.root.remove();
  await flushMutationTurn();
  assert.equal(child.controller.isTopLayer.value, false);
  assert.equal(parent.controller.isTopLayer.value, true);
  dispatchPointerDown(parent.after);
  assert.equal(parent.dismisses.length, 1);
  child.unmount();
  parent.unmount();
});

test("runtime diagnostics, idempotence, and effect-scope disposal are explicit", () => {
  assert.throws(() => createDismissableLayer(null as never), /options must be an object/);
  assert.throws(
    () => createDismissableLayer({ root: "#layer" } as never),
    /VIZE_UI_DISMISSABLE_LAYER_ROOT/,
  );
  assert.throws(
    () => createDismissableLayer({ outsideFocus: "yes", root: null } as never),
    /VIZE_UI_DISMISSABLE_LAYER_OPTION.*outsideFocus/,
  );
  assert.throws(
    () => createDismissableLayer({ onDismiss: true, root: null } as never),
    /VIZE_UI_DISMISSABLE_LAYER_OPTION.*onDismiss/,
  );
  assert.throws(
    () => useDismissableLayer({ root: document.body }),
    /VIZE_UI_DISMISSABLE_LAYER_SETUP/,
  );

  const root = document.createElement("div");
  document.body.append(root);
  const scope = effectScope();
  const controller = scope.run(() => useDismissableLayer({ root }))!;
  assert.equal(controller.isActive.value, true);
  assert.equal(controller.isTopLayer.value, true);
  scope.stop();
  assert.equal(controller.isActive.value, false);
  controller.dispose();
  assert.throws(() => controller.refresh(), /VIZE_UI_DISMISSABLE_LAYER_DISPOSED/);
  root.remove();
});
