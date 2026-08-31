import assert from "node:assert/strict";

import { effectScope, ref } from "vue";
import { test } from "vite-plus/test";

import { createCollectionRegistry } from "../collection/collection.ts";
import { createCompositeNavigation, useCompositeNavigation } from "./composite-navigation.ts";
import type { CompositeNavigationChange } from "./composite-navigation.ts";
import { keyboard, mountComposite, setDynamicProps } from "./composite-navigation-test-utils.ts";

test("roving tabindex skips disabled items and publishes frozen navigation snapshots", () => {
  const changes: CompositeNavigationChange<string>[] = [];
  const harness = mountComposite({ onNavigate: (change) => changes.push(change) });
  setDynamicProps(harness);
  assert.equal(harness.elements.get("alpha")?.tabIndex, 0);
  assert.equal(harness.elements.get("bravo")?.tabIndex, -1);

  harness.elements.get("alpha")?.focus();
  const event = keyboard("ArrowDown", {}, harness.container);
  assert.equal(event.defaultPrevented, true);
  assert.equal(harness.controller.activeKey.value, "bravo");
  assert.equal(document.activeElement, harness.elements.get("bravo"));
  assert.deepEqual(
    changes.map(({ key, previousKey, intent }) => ({ key, previousKey, intent })),
    [
      { key: "alpha", previousKey: null, intent: "focus" },
      { key: "bravo", previousKey: "alpha", intent: "next" },
    ],
  );
  assert.ok(changes.every(Object.isFrozen));
  assert.equal(changes[1]?.originalEvent, event);
  assert.equal(changes[1]?.focusStrategy, "roving");

  setDynamicProps(harness);
  assert.equal(harness.elements.get("alpha")?.tabIndex, -1);
  assert.equal(harness.elements.get("bravo")?.tabIndex, 0);
  harness.unmount();
});

test("orientation and RTL direction remain reactive without rebuilding handlers", () => {
  const orientation = ref<"horizontal" | "vertical">("horizontal");
  const direction = ref<"ltr" | "rtl">("ltr");
  const harness = mountComposite({ direction, orientation });
  const handler = harness.controller.getContainerProps().onKeydown;
  harness.registry.setActiveKey("bravo");

  keyboard("ArrowRight", {}, harness.container);
  assert.equal(harness.controller.activeKey.value, "charlie");
  direction.value = "rtl";
  keyboard("ArrowRight", {}, harness.container);
  assert.equal(harness.controller.activeKey.value, "bravo");
  orientation.value = "vertical";
  assert.equal(keyboard("ArrowRight", {}, harness.container).defaultPrevented, false);
  keyboard("ArrowUp", {}, harness.container);
  assert.equal(harness.controller.activeKey.value, "alpha");
  assert.equal(harness.controller.getContainerProps().onKeydown, handler);
  harness.unmount();
});

test("Home, End, paging, looping, and imperative navigation share one state", () => {
  const loop = ref(false);
  const pageSize = ref(2);
  const harness = mountComposite({ loop, pageSize });
  harness.registry.setActiveKey("alpha");
  keyboard("PageDown", {}, harness.container);
  assert.equal(harness.controller.activeKey.value, "charlie");
  keyboard("PageUp", {}, harness.container);
  assert.equal(harness.controller.activeKey.value, "alpha");
  assert.equal(harness.controller.navigate("last"), "charlie");
  assert.equal(harness.controller.navigate("next"), "charlie");
  loop.value = true;
  assert.equal(harness.controller.navigate("next"), "alpha");
  keyboard("End", {}, harness.container);
  assert.equal(harness.controller.activeKey.value, "charlie");
  keyboard("Home", {}, harness.container);
  assert.equal(harness.controller.activeKey.value, "alpha");
  harness.unmount();
});

test("keyboard navigation ignores edits, shortcuts, composition, and handled events", () => {
  const harness = mountComposite({ orientation: "both" });
  harness.registry.setActiveKey("alpha");
  const input = document.createElement("input");
  harness.container.append(input);
  assert.equal(keyboard("ArrowDown", {}, input).defaultPrevented, false);
  assert.equal(keyboard("ArrowDown", { ctrlKey: true }, harness.container).defaultPrevented, false);
  assert.equal(keyboard("ArrowDown", { metaKey: true }, harness.container).defaultPrevented, false);
  const composing = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "ArrowDown",
  });
  Object.defineProperty(composing, "isComposing", { value: true });
  harness.container.dispatchEvent(composing);
  const handled = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "ArrowDown",
  });
  handled.preventDefault();
  harness.container.dispatchEvent(handled);
  assert.equal(harness.controller.activeKey.value, "alpha");
  harness.unmount();
});

test("pointer focus, reactive disablement, and integrated typeahead compose", () => {
  const disabled = ref(false);
  const changes: string[] = [];
  const harness = mountComposite({
    isDisabled: disabled,
    onNavigate: ({ intent }) => changes.push(intent),
    typeahead: { timeout: 1_000 },
  });
  harness.elements.get("bravo")?.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true }));
  assert.equal(harness.controller.activeKey.value, "bravo");
  assert.equal(changes.at(-1), "pointer");

  const typed = keyboard("c", {}, harness.container);
  assert.equal(typed.defaultPrevented, true);
  assert.equal(harness.controller.activeKey.value, "charlie");
  assert.equal(document.activeElement, harness.elements.get("charlie"));
  assert.equal(changes.at(-1), "typeahead");
  disabled.value = true;
  assert.equal(harness.controller.typeahead?.query.value, "");
  assert.equal(keyboard("Home", {}, harness.container).defaultPrevented, false);
  assert.equal(harness.controller.navigate("first"), null);
  assert.equal(harness.controller.activeKey.value, "charlie");
  harness.unmount();
});

test("typeahead does not publish callbacks for an unchanged active item", () => {
  const navigation: string[] = [];
  const matches: string[] = [];
  const harness = mountComposite({
    onNavigate: ({ intent }) => navigation.push(intent),
    typeahead: { onMatch: ({ key }) => matches.push(key) },
  });
  harness.registry.setActiveKey("alpha");
  keyboard("a", {}, harness.container);
  assert.deepEqual(matches, []);
  assert.deepEqual(navigation, []);
  harness.unmount();
});

test("DOM and callback failures aggregate after logical state commits", () => {
  const focusFailure = new Error("focus failed");
  const scrollFailure = new Error("scroll failed");
  const callbackFailure = new Error("callback failed");
  const harness = mountComposite({
    preventScroll: true,
    scrollIntoView: () => {
      throw scrollFailure;
    },
    onNavigate: () => {
      throw callbackFailure;
    },
  });
  harness.registry.setActiveKey("alpha");
  const bravo = harness.elements.get("bravo")!;
  bravo.focus = () => {
    throw focusFailure;
  };
  assert.throws(
    () => harness.controller.navigate("next"),
    (error) => {
      assert.equal((error as Error).name, "AggregateError");
      assert.deepEqual((error as AggregateError).errors, [
        focusFailure,
        scrollFailure,
        callbackFailure,
      ]);
      return true;
    },
  );
  assert.equal(harness.controller.activeKey.value, "bravo");
  harness.unmount();
});

test("validates options, missing keys, disposal, and Vue scope ownership", () => {
  const registry = createCollectionRegistry<string, string>();
  assert.throws(
    () => createCompositeNavigation({ registry, pageSize: 0 }),
    /VIZE_UI_COMPOSITE_NAVIGATION_OPTION.*pageSize/,
  );
  assert.throws(
    () =>
      createCompositeNavigation({
        registry,
        focusStrategy: "active-descendant",
      } as never),
    /active-descendant requires a getItemId/,
  );
  const controller = createCompositeNavigation({ registry });
  assert.throws(() => controller.getItemProps("missing"), /VIZE_UI_COMPOSITE_NAVIGATION_ITEM/);
  controller.dispose();
  controller.dispose();
  assert.throws(() => controller.navigate("first"), /VIZE_UI_COMPOSITE_NAVIGATION_DISPOSED/);
  assert.throws(() => useCompositeNavigation({ registry }), /VIZE_UI_COMPOSITE_NAVIGATION_SETUP/);

  const scope = effectScope();
  let scoped!: ReturnType<typeof useCompositeNavigation<string, string>>;
  scope.run(() => {
    scoped = useCompositeNavigation({ registry });
  });
  scope.stop();
  assert.throws(() => scoped.getContainerProps(), /VIZE_UI_COMPOSITE_NAVIGATION_DISPOSED/);
  registry.dispose();
});
