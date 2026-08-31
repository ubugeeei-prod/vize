import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createCollectionRegistry } from "../collection/collection.ts";
import { createCompositeNavigation } from "./composite-navigation.ts";
import { keyboard, mountComposite, setDynamicProps } from "./composite-navigation-test-utils.ts";

test("active descendant keeps DOM focus on the container while moving logical focus", () => {
  const revealed: string[] = [];
  const harness = mountComposite({
    focusStrategy: "active-descendant",
    getItemId: ({ key }) => `stable-${key}`,
    scrollIntoView: ({ key }) => revealed.push(key),
  });
  setDynamicProps(harness);
  assert.equal(harness.container.tabIndex, 0);
  assert.equal(harness.container.getAttribute("aria-activedescendant"), "stable-alpha");
  assert.equal(harness.elements.get("alpha")?.hasAttribute("tabindex"), false);

  harness.container.focus();
  assert.equal(harness.controller.activeKey.value, "alpha");
  setDynamicProps(harness);
  const event = keyboard("ArrowDown", {}, harness.container);
  setDynamicProps(harness);
  assert.equal(event.defaultPrevented, true);
  assert.equal(harness.controller.activeKey.value, "bravo");
  assert.equal(document.activeElement, harness.container);
  assert.equal(harness.container.getAttribute("aria-activedescendant"), "stable-bravo");
  assert.deepEqual(revealed, ["alpha", "bravo"]);
  harness.unmount();
});

test("pointer activation updates active descendant without stealing container focus", () => {
  const harness = mountComposite({
    focusStrategy: "active-descendant",
    getItemId: ({ key }) => `item-${key}`,
  });
  setDynamicProps(harness);
  harness.container.focus();
  harness.elements
    .get("charlie")
    ?.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true }));
  setDynamicProps(harness);
  assert.equal(harness.controller.activeKey.value, "charlie");
  assert.equal(document.activeElement, harness.container);
  assert.equal(harness.container.getAttribute("aria-activedescendant"), "item-charlie");
  harness.unmount();
});

test("rejects invalid IDs and unrelated portal descendants after committing state", () => {
  const harness = mountComposite({
    focusStrategy: "active-descendant",
    getItemId: ({ key }) => `item-${key}`,
  });
  setDynamicProps(harness);
  harness.container.focus();
  const portal = document.createElement("div");
  document.body.append(portal);
  portal.append(harness.elements.get("bravo")!);
  assert.throws(
    () => harness.controller.navigate("next"),
    /VIZE_UI_COMPOSITE_NAVIGATION_RELATIONSHIP/,
  );
  assert.equal(harness.controller.activeKey.value, "bravo");

  harness.container.setAttribute("aria-owns", "item-bravo");
  assert.equal(harness.controller.navigate("previous"), "alpha");
  assert.equal(harness.controller.navigate("next"), "bravo");
  portal.remove();
  harness.unmount();

  const registry = createCollectionRegistry<string, string>();
  registry.register({ key: "alpha", value: "Alpha", textValue: "Alpha" });
  const invalid = createCompositeNavigation({
    registry,
    focusStrategy: "active-descendant",
    getItemId: () => "contains whitespace",
  });
  assert.throws(() => invalid.getContainerProps(), /VIZE_UI_COMPOSITE_NAVIGATION_ID/);
  invalid.dispose();
  registry.dispose();
});

test("combobox aria-controls establishes a valid active-descendant relationship", () => {
  const harness = mountComposite({
    focusStrategy: "active-descendant",
    getItemId: ({ key }) => `item-${key}`,
  });
  const listbox = document.createElement("div");
  listbox.id = "results";
  listbox.setAttribute("role", "listbox");
  for (const element of harness.elements.values()) listbox.append(element);
  document.body.append(listbox);
  harness.container.setAttribute("role", "combobox");
  harness.container.setAttribute("aria-controls", "results");
  harness.registry.setActiveKey("alpha");
  harness.container.dispatchEvent(new FocusEvent("focus"));
  assert.equal(harness.controller.navigate("next"), "bravo");
  listbox.remove();
  harness.unmount();
});

test("virtualized active descendants work without mounted elements through custom reveal", () => {
  const registry = createCollectionRegistry<string, string>();
  registry.register({ key: "alpha", value: "Alpha", textValue: "Alpha", element: null });
  registry.register({ key: "bravo", value: "Bravo", textValue: "Bravo", element: null });
  const revealed: Array<{ key: string; event: Event | null }> = [];
  const controller = createCompositeNavigation({
    registry,
    focusStrategy: "active-descendant",
    getItemId: ({ key }) => `virtual-${key}`,
    scrollIntoView: ({ key }, event) => revealed.push({ key, event }),
  });
  assert.equal(controller.getContainerProps()["aria-activedescendant"], "virtual-alpha");
  const event = new Event("programmatic");
  assert.equal(controller.navigate("first", event), "alpha");
  assert.deepEqual(revealed, [{ key: "alpha", event }]);
  controller.dispose();
  registry.dispose();
});

test("active-descendant initialization honors disabled state and empty collections", () => {
  const empty = createCollectionRegistry<string, string>();
  const controller = createCompositeNavigation({
    registry: empty,
    focusStrategy: "active-descendant",
    getItemId: ({ key }) => key,
  });
  assert.deepEqual(Object.keys(controller.getContainerProps()).sort(), [
    "onFocus",
    "onKeydown",
    "tabindex",
  ]);
  const host = document.createElement("div");
  document.body.append(host);
  host.addEventListener("focus", controller.getContainerProps().onFocus);
  host.focus();
  assert.equal(controller.activeKey.value, null);
  controller.dispose();
  empty.dispose();
  host.remove();
});

test("an applied ID must resolve back to the active registered element", () => {
  const harness = mountComposite({
    focusStrategy: "active-descendant",
    getItemId: ({ key }) => `stable-${key}`,
  });
  setDynamicProps(harness);
  harness.container.focus();
  harness.elements.get("bravo")!.id = "consumer-forgot-to-bind-the-id";
  assert.throws(
    () => harness.controller.navigate("next"),
    /VIZE_UI_COMPOSITE_NAVIGATION_ID_RELATIONSHIP/,
  );
  assert.equal(harness.controller.activeKey.value, "bravo");
  harness.unmount();
});

test("controlled popup relationships resolve inside the host shadow root", () => {
  const harness = mountComposite({
    focusStrategy: "active-descendant",
    getItemId: ({ key }) => `item-${key}`,
  });
  const shell = document.createElement("div");
  const shadow = shell.attachShadow({ mode: "open" });
  const popup = document.createElement("div");
  popup.id = "shadow-popup";
  for (const element of harness.elements.values()) popup.append(element);
  shadow.append(harness.container, popup);
  document.body.append(shell);
  harness.container.setAttribute("role", "combobox");
  harness.container.setAttribute("aria-controls", popup.id);
  setDynamicProps(harness);
  harness.container.focus();
  assert.equal(harness.controller.navigate("next"), "bravo");
  harness.unmount();
  shell.remove();
});

test("controlled popup relationships never resolve across the shadow boundary", () => {
  const harness = mountComposite({
    focusStrategy: "active-descendant",
    getItemId: ({ key }) => `item-${key}`,
  });
  const shell = document.createElement("div");
  const shadow = shell.attachShadow({ mode: "open" });
  const popup = document.createElement("div");
  popup.id = "document-popup";
  for (const element of harness.elements.values()) popup.append(element);
  shadow.append(harness.container);
  document.body.append(shell, popup);
  harness.container.setAttribute("role", "combobox");
  harness.container.setAttribute("aria-controls", popup.id);
  setDynamicProps(harness);
  keyboard("Shift", {}, harness.container);
  assert.throws(
    () => harness.controller.navigate("next"),
    /VIZE_UI_COMPOSITE_NAVIGATION_RELATIONSHIP/,
  );
  harness.unmount();
  shell.remove();
  popup.remove();
});
