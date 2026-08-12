import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  createCollectionRegistry,
  extractCollectionTextValue,
  normalizeCollectionTextValue,
} from "./collection.ts";

interface ItemValue {
  readonly label: string;
}

function item(label: string): ItemValue {
  return Object.freeze({ label });
}

async function flushDomMutations(): Promise<void> {
  await new Promise<void>((resolve) => setTimeout(resolve, 0));
}

test("scopes mutation observation and preserves observers across text refreshes", async () => {
  const NativeMutationObserver = window.MutationObserver;
  let observerCount = 0;
  class CountingMutationObserver extends NativeMutationObserver {
    constructor(callback: MutationCallback) {
      super(callback);
      observerCount += 1;
    }
  }
  Object.defineProperty(window, "MutationObserver", {
    configurable: true,
    value: CountingMutationObserver,
  });

  const registry = createCollectionRegistry<string, ItemValue>();
  const collectionRoot = document.createElement("div");
  const connected = document.createElement("div");
  const disconnected = document.createElement("div");
  const unrelated = document.createElement("div");
  let textResolutionCount = 0;
  connected.textContent = "Connected";
  disconnected.textContent = "Disconnected";
  collectionRoot.append(connected);
  document.body.append(collectionRoot, unrelated);

  try {
    registry.register({
      key: "connected",
      value: item("Connected"),
      element: connected,
      textValue: () => {
        textResolutionCount += 1;
        return connected.textContent ?? "";
      },
    });
    registry.register({
      key: "disconnected",
      value: item("Disconnected"),
      element: disconnected,
    });
    assert.deepEqual(
      registry.items.value.map(({ textValue }) => textValue),
      ["Connected", "Disconnected"],
    );
    const resolutionsBeforeUnrelatedMutation = textResolutionCount;
    const observersBeforeTextMutation = observerCount;

    unrelated.textContent = "An unrelated document mutation";
    await flushDomMutations();
    void registry.items.value;
    assert.equal(textResolutionCount, resolutionsBeforeUnrelatedMutation);

    connected.textContent = "Updated";
    await flushDomMutations();
    assert.equal(registry.items.value[0]?.textValue, "Updated");
    assert.equal(observerCount, observersBeforeTextMutation);
  } finally {
    registry.dispose();
    collectionRoot.remove();
    unrelated.remove();
    Object.defineProperty(window, "MutationObserver", {
      configurable: true,
      value: NativeMutationObserver,
    });
  }
});

test("reorders items when a fully detached collection mounts", async () => {
  const registry = createCollectionRegistry<string, ItemValue>();
  const first = document.createElement("div");
  const second = document.createElement("div");

  try {
    registry.register({ key: "first", value: item("First"), element: first });
    registry.register({ key: "second", value: item("Second"), element: second });
    assert.deepEqual(
      registry.items.value.map(({ key }) => key),
      ["first", "second"],
    );

    document.body.append(second, first);
    await flushDomMutations();
    assert.deepEqual(
      registry.items.value.map(({ key }) => key),
      ["second", "first"],
    );
  } finally {
    registry.dispose();
    first.remove();
    second.remove();
  }
});

test("keeps observing an item that is detached after registration", async () => {
  const registry = createCollectionRegistry<string, ItemValue>();
  const element = document.createElement("div");
  element.textContent = "Attached";
  document.body.append(element);

  try {
    registry.register({
      key: "item",
      value: item("Item"),
      element,
      textValue: () => element.textContent ?? "",
    });
    assert.equal(registry.items.value[0]?.textValue, "Attached");

    element.remove();
    await flushDomMutations();
    void registry.items.value;

    element.textContent = "Updated while detached";
    await flushDomMutations();
    assert.equal(registry.items.value[0]?.textValue, "Updated while detached");
  } finally {
    registry.dispose();
    element.remove();
  }
});

test("refreshes snapshots synchronously and rejects refresh after disposal", () => {
  const registry = createCollectionRegistry<string, ItemValue>();
  const element = document.createElement("div");
  element.textContent = "Before";
  registry.register({ key: "item", value: item("Item"), element });

  assert.equal(registry.items.value[0]?.textValue, "Before");
  element.textContent = "After";
  assert.equal(registry.items.value[0]?.textValue, "Before");

  registry.refresh();
  assert.equal(registry.items.value[0]?.textValue, "After");
  registry.dispose();
  assert.throws(() => registry.refresh(), /VIZE_UI_COLLECTION_DISPOSED/);
});

test("extracts normalized accessible text across inline and shadow DOM content", () => {
  const label = document.createElement("span");
  label.id = "collection-label";
  label.hidden = true;
  label.textContent = "  Café\nau lait ";
  const labelled = document.createElement("div");
  labelled.setAttribute("aria-labelledby", label.id);
  labelled.textContent = "Ignored content";
  document.body.append(label, labelled);

  assert.equal(extractCollectionTextValue(labelled), "Café au lait");

  const content = document.createElement("div");
  content.append("Visible ");
  const hidden = document.createElement("span");
  hidden.setAttribute("aria-hidden", "true");
  hidden.textContent = "decorative";
  const image = document.createElement("img");
  image.alt = "avatar";
  content.append(hidden, image);
  assert.equal(extractCollectionTextValue(content), "Visible avatar");

  const input = document.createElement("input");
  input.type = "submit";
  input.value = "Send form";
  assert.equal(extractCollectionTextValue(input), "Send form");

  const inline = document.createElement("div");
  const strong = document.createElement("strong");
  strong.textContent = "Al";
  inline.append(strong, "pha");
  assert.equal(extractCollectionTextValue(inline), "Alpha");

  const shadowHost = document.createElement("div");
  const shadowRoot = shadowHost.attachShadow({ mode: "open" });
  const shadowLabel = document.createElement("span");
  shadowLabel.id = "shadow-label";
  shadowLabel.textContent = "Shadow label";
  const shadowItem = document.createElement("div");
  shadowItem.setAttribute("aria-labelledby", shadowLabel.id);
  shadowItem.textContent = "Ignored shadow content";
  shadowRoot.append(shadowLabel, shadowItem);
  document.body.append(shadowHost);
  assert.equal(extractCollectionTextValue(shadowItem), "Shadow label");

  assert.equal(normalizeCollectionTextValue(" e\u0301\tclair "), "é clair");

  label.remove();
  labelled.remove();
  shadowHost.remove();
});
