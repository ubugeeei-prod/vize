import assert from "node:assert/strict";

import { ref } from "vue";
import { test } from "vite-plus/test";

import { createCollectionRegistry } from "../collection/collection.ts";
import { surfaceErrors } from "./composite-navigation-internal.ts";
import { createCompositeNavigation } from "./composite-navigation.ts";
import { keyboard, mountComposite } from "./composite-navigation-test-utils.ts";

test("reactive option diagnostics surface at the transition that reads them", () => {
  const loop = ref<boolean | string>(false);
  const pageSize = ref<number | string>(2);
  const disabled = ref<boolean | string>(false);
  const harness = mountComposite({
    isDisabled: disabled as never,
    loop: loop as never,
    pageSize: pageSize as never,
  });
  harness.registry.setActiveKey("alpha");
  loop.value = "invalid";
  assert.throws(
    () => harness.controller.navigate("next"),
    /VIZE_UI_COMPOSITE_NAVIGATION_OPTION.*loop/,
  );
  assert.equal(harness.controller.activeKey.value, "alpha");
  loop.value = false;
  pageSize.value = 0;
  assert.throws(
    () => harness.controller.navigate("page-next"),
    /VIZE_UI_COMPOSITE_NAVIGATION_OPTION.*pageSize/,
  );
  pageSize.value = 2;
  disabled.value = "invalid";
  assert.throws(
    () => harness.controller.navigate("next"),
    /VIZE_UI_COMPOSITE_NAVIGATION_OPTION.*isDisabled/,
  );
  harness.unmount();
});

test("reactive orientation and direction validate before consuming a key", () => {
  const orientation = ref<string>("vertical");
  const direction = ref<string>("ltr");
  const harness = mountComposite({
    direction: direction as never,
    orientation: orientation as never,
  });
  const handler = harness.controller.getContainerProps().onKeydown;
  orientation.value = "diagonal";
  assert.throws(() => handler(new KeyboardEvent("keydown", { key: "ArrowDown" })), /orientation/);
  orientation.value = "vertical";
  direction.value = "sideways";
  assert.throws(() => handler(new KeyboardEvent("keydown", { key: "ArrowDown" })), /direction/);
  harness.unmount();
});

test("runtime boundaries reject malformed options and imperative commands", () => {
  const registry = createCollectionRegistry<string, string>();
  assert.throws(() => createCompositeNavigation(null as never), /options must be an object/);
  assert.throws(
    () => createCompositeNavigation({ registry, typeahead: true } as never),
    /typeahead must be false or an options object/,
  );
  const controller = createCompositeNavigation({ registry });
  assert.throws(() => controller.navigate("sideways" as never), /navigation intent is invalid/);
  controller.dispose();
  registry.dispose();
});

test("preventScroll preserves focus options and invokes the nearest reveal fallback", () => {
  const harness = mountComposite({ preventScroll: true });
  harness.registry.setActiveKey("alpha");
  const item = harness.elements.get("bravo")!;
  const focusCalls: unknown[] = [];
  const revealCalls: unknown[] = [];
  item.focus = (options?: FocusOptions) => focusCalls.push(options);
  Object.assign(item, {
    scrollIntoView: (options?: ScrollIntoViewOptions) => revealCalls.push(options),
  });
  harness.controller.navigate("next");
  assert.deepEqual(focusCalls, [{ preventScroll: true }]);
  assert.deepEqual(revealCalls, [{ block: "nearest", inline: "nearest" }]);
  harness.unmount();
});

test("shadow-root editable descendants keep navigation keys for their editor", () => {
  const harness = mountComposite();
  harness.registry.setActiveKey("alpha");
  const host = document.createElement("span");
  const shadow = host.attachShadow({ mode: "open" });
  const input = document.createElement("input");
  shadow.append(input);
  harness.container.append(host);
  const event = keyboard("ArrowDown", { composed: true }, input);
  assert.equal(event.defaultPrevented, false);
  assert.equal(harness.controller.activeKey.value, "alpha");
  harness.unmount();
});

test("item handlers are stable, become inert after disposal, and leave registry ownership alone", () => {
  const harness = mountComposite();
  const first = harness.controller.getItemProps("alpha");
  const second = harness.controller.getItemProps("alpha");
  assert.equal(first.onFocus, second.onFocus);
  assert.equal(first.onPointerdown, second.onPointerdown);
  const containerProps = harness.controller.getContainerProps();
  harness.controller.dispose();
  first.onFocus(new FocusEvent("focus"));
  assert.equal(harness.registry.activeKey.value, null);
  assert.doesNotThrow(() => containerProps.onFocus(new FocusEvent("focus")));
  assert.doesNotThrow(() =>
    containerProps.onKeydown(new KeyboardEvent("keydown", { key: "Home" })),
  );
  assert.equal(harness.registry.activeKey.value, null);
  assert.doesNotThrow(() => harness.registry.setActiveKey("alpha"));
  harness.registry.dispose();
  harness.container.remove();
});

test("handler caching drops entries for keys the registry no longer holds", () => {
  const harness = mountComposite();
  const element = document.createElement("button");
  harness.container.append(element);
  const delta = harness.registry.register({ key: "delta", value: { label: "Delta" }, element });
  const cached = harness.controller.getItemProps("delta");
  delta.unregister();
  const other = document.createElement("button");
  harness.container.append(other);
  harness.registry.register({ key: "echo", value: { label: "Echo" }, element: other });
  harness.controller.getItemProps("echo");
  harness.registry.register({ key: "delta", value: { label: "Delta" }, element });
  assert.notEqual(harness.controller.getItemProps("delta").onFocus, cached.onFocus);
  harness.unmount();
});

test("non-text input descendants keep composite keyboard navigation", () => {
  const harness = mountComposite();
  harness.registry.setActiveKey("alpha");
  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  harness.container.append(checkbox);
  assert.equal(keyboard("ArrowDown", {}, checkbox).defaultPrevented, true);
  assert.equal(harness.controller.activeKey.value, "bravo");
  harness.unmount();
});

test("error aggregation retains every failure without native AggregateError", () => {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, "AggregateError");
  const first = new Error("first");
  const second = new Error("second");
  Object.defineProperty(globalThis, "AggregateError", { configurable: true, value: undefined });
  try {
    assert.throws(
      () => surfaceErrors([first, second], "failed"),
      (error) => {
        assert.equal((error as Error).name, "AggregateError");
        assert.deepEqual((error as Error & { errors: unknown[] }).errors, [first, second]);
        return true;
      },
    );
  } finally {
    if (descriptor) Object.defineProperty(globalThis, "AggregateError", descriptor);
  }
});
