import assert from "node:assert/strict";

import { ref } from "vue";
import { test } from "vite-plus/test";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { surfaceErrors } from "./spatial-navigation-internal.ts";
import { createSpatialNavigation } from "./spatial-navigation.ts";
import { keyboard, mountSpatial, rect } from "./spatial-navigation-test-utils.ts";

test("modified, composing, handled, and editable descendant keys remain untouched", () => {
  const harness = mountSpatial();
  harness.registry.setActiveKey("alpha");
  assert.equal(
    keyboard("ArrowRight", harness.container, { ctrlKey: true }).defaultPrevented,
    false,
  );
  assert.equal(keyboard("ArrowRight", harness.container, { altKey: true }).defaultPrevented, false);
  assert.equal(
    keyboard("ArrowRight", harness.container, { metaKey: true }).defaultPrevented,
    false,
  );
  assert.equal(
    keyboard("ArrowRight", harness.container, { shiftKey: true }).defaultPrevented,
    false,
  );
  const composing = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "ArrowRight",
  });
  Object.defineProperty(composing, "isComposing", { value: true });
  harness.container.dispatchEvent(composing);
  const handled = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "ArrowRight",
  });
  handled.preventDefault();
  harness.container.dispatchEvent(handled);
  const input = document.createElement("input");
  harness.container.append(input);
  assert.equal(keyboard("ArrowRight", input).defaultPrevented, false);
  assert.equal(harness.registry.activeKey.value, "alpha");
  harness.unmount();
});

test("editable descendants remain protected through shadow DOM retargeting", () => {
  const harness = mountSpatial();
  harness.registry.setActiveKey("alpha");
  const host = document.createElement("span");
  const shadow = host.attachShadow({ mode: "open" });
  const textarea = document.createElement("textarea");
  shadow.append(textarea);
  harness.container.append(host);
  const event = keyboard("ArrowRight", textarea, { composed: true });
  assert.equal(event.defaultPrevented, false);
  assert.equal(harness.registry.activeKey.value, "alpha");
  harness.unmount();
});

test("focus, reveal, and consumer failures aggregate after logical state commits", () => {
  const focusFailure = new Error("focus failed");
  const revealFailure = new Error("reveal failed");
  const callbackFailure = new Error("callback failed");
  const harness = mountSpatial({
    preventScroll: true,
    scrollIntoView: () => {
      throw revealFailure;
    },
    onNavigate: () => {
      throw callbackFailure;
    },
  });
  harness.registry.setActiveKey("alpha");
  harness.elements.get("bravo")!.focus = () => {
    throw focusFailure;
  };
  assert.throws(
    () => harness.controller.navigate("right"),
    (error) => {
      assert.equal((error as Error).name, "AggregateError");
      assert.deepEqual((error as AggregateError).errors, [
        focusFailure,
        revealFailure,
        callbackFailure,
      ]);
      return true;
    },
  );
  assert.equal(harness.registry.activeKey.value, "bravo");
  harness.unmount();
});

test("reactive transition validation fails without rolling back committed state", () => {
  const focusBehavior = ref<string>("focus");
  const harness = mountSpatial({ focusBehavior: focusBehavior as never });
  harness.registry.setActiveKey("alpha");
  focusBehavior.value = "invalid";
  assert.throws(
    () => harness.controller.navigate("right"),
    /VIZE_UI_SPATIAL_NAVIGATION_OPTION.*focusBehavior/,
  );
  assert.equal(harness.registry.activeKey.value, "bravo");
  harness.unmount();
});

test("missing candidate geometry is skipped but missing origin geometry is diagnostic", () => {
  const harness = mountSpatial({
    getRect: ({ key }) => (key === "bravo" ? null : harness.rects.get(key)),
  });
  harness.registry.setActiveKey("alpha");
  assert.equal(harness.controller.findTarget("right"), "delta");
  harness.registry.setActiveKey("bravo");
  assert.throws(() => harness.controller.findTarget("left"), /has no measurable rectangle/);
  harness.unmount();
});

test("focus option fallback and default logical reveal preserve browser compatibility", () => {
  const focusHarness = mountSpatial({ preventScroll: true });
  focusHarness.registry.setActiveKey("alpha");
  const focusCalls: unknown[] = [];
  const target = focusHarness.elements.get("bravo")!;
  target.focus = (options?: FocusOptions) => {
    focusCalls.push(options);
    if (options) throw new TypeError("legacy focus signature");
  };
  Object.assign(target, { scrollIntoView: () => undefined });
  focusHarness.controller.navigate("right");
  assert.deepEqual(focusCalls, [{ preventScroll: true }, undefined]);
  focusHarness.unmount();

  const logicalHarness = mountSpatial({ focusBehavior: "logical" });
  logicalHarness.registry.setActiveKey("alpha");
  const reveals: unknown[] = [];
  Object.assign(logicalHarness.elements.get("bravo")!, {
    scrollIntoView: (options: unknown) => reveals.push(options),
  });
  logicalHarness.controller.navigate("right");
  assert.deepEqual(reveals, [{ block: "nearest", inline: "nearest" }]);
  logicalHarness.unmount();
});

test("empty collections neither consume keys nor publish boundary callbacks", () => {
  const registry = createCollectionRegistry<string, string>();
  let boundaries = 0;
  const controller = createSpatialNavigation({ registry, onBoundary: () => boundaries++ });
  const host = document.createElement("div");
  host.addEventListener("keydown", controller.spatialNavigationProps.onKeydown);
  const event = keyboard("ArrowRight", host);
  assert.equal(event.defaultPrevented, false);
  assert.equal(boundaries, 0);
  controller.dispose();
  registry.dispose();
});

test("fallback aggregation retains all failures without native AggregateError", () => {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, "AggregateError");
  const failures = [new Error("first"), new Error("second")];
  Object.defineProperty(globalThis, "AggregateError", { configurable: true, value: undefined });
  try {
    assert.throws(
      () => surfaceErrors(failures, "failed"),
      (error) => {
        assert.equal((error as Error).name, "AggregateError");
        assert.deepEqual((error as Error & { errors: unknown[] }).errors, failures);
        return true;
      },
    );
  } finally {
    if (descriptor) Object.defineProperty(globalThis, "AggregateError", descriptor);
  }
});

test("zero-sized virtual rectangles remain valid geometry", () => {
  const registry = createCollectionRegistry<string, string>();
  registry.register({ key: "origin", value: "origin", textValue: "origin" });
  registry.register({ key: "target", value: "target", textValue: "target" });
  registry.setActiveKey("origin");
  const controller = createSpatialNavigation({
    registry,
    focusBehavior: "logical",
    getRect: ({ key }) => (key === "origin" ? rect(0, 0, 0, 0) : rect(1, 0, 0, 0)),
  });
  assert.equal(controller.findTarget("right"), "target");
  controller.dispose();
  registry.dispose();
});
