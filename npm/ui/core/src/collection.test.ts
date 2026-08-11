import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, ref } from "vue";

import {
  createCollectionRegistry,
  type CollectionActiveChange,
  type CollectionKey,
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

test("resolves immutable snapshots from reactive item and DOM sources", async () => {
  const registry = createCollectionRegistry<string, ItemValue>();
  const element = ref<Element | null>(null);
  const textValue = ref<string | undefined>("  Alpha\nitem  ");
  const disabled = ref(false);
  const registration = registry.register({
    key: "alpha",
    value: item("Alpha"),
    element,
    textValue,
    disabled,
  });

  assert.equal(Object.isFrozen(registry), true);
  assert.equal(Object.isFrozen(registry.items.value), true);
  assert.equal(Object.isFrozen(registry.items.value[0]), true);
  assert.deepEqual(registry.items.value[0], {
    key: "alpha",
    value: { label: "Alpha" },
    element: null,
    textValue: "Alpha item",
    disabled: false,
    order: undefined,
  });

  const button = document.createElement("button");
  button.textContent = "DOM fallback";
  element.value = button;
  textValue.value = undefined;
  disabled.value = true;

  assert.equal(registry.items.value[0]?.element, button);
  assert.equal(registry.items.value[0]?.textValue, "DOM fallback");
  assert.equal(registry.items.value[0]?.disabled, true);

  button.textContent = "Observed DOM text";
  await flushDomMutations();
  assert.equal(registry.items.value[0]?.textValue, "Observed DOM text");

  assert.equal(registration.registered.value, true);
  assert.equal(registration.unregister(), true);
  assert.equal(registration.unregister(), false);
  assert.equal(registration.registered.value, false);
  registry.dispose();
});

test("observes connected DOM order and uses deterministic registration fallback", async () => {
  const registry = createCollectionRegistry<string, ItemValue>();
  const first = document.createElement("div");
  const second = document.createElement("div");
  const disconnected = document.createElement("div");
  document.body.append(second, first);

  registry.register({ key: "first", value: item("First"), element: first });
  registry.register({ key: "second", value: item("Second"), element: second });

  assert.deepEqual(
    registry.items.value.map(({ key }) => key),
    ["second", "first"],
  );

  document.body.append(second);
  await flushDomMutations();
  assert.deepEqual(
    registry.items.value.map(({ key }) => key),
    ["first", "second"],
  );

  registry.register({ key: "disconnected", value: item("Disconnected"), element: disconnected });
  assert.deepEqual(
    registry.items.value.map(({ key }) => key),
    ["first", "second", "disconnected"],
  );

  document.body.prepend(disconnected);
  await flushDomMutations();
  assert.deepEqual(
    registry.items.value.map(({ key }) => key),
    ["disconnected", "first", "second"],
  );

  first.remove();
  second.remove();
  disconnected.remove();
  registry.dispose();
});

test("requires complete, unique, safe explicit ordering and rolls back rejection", () => {
  const registry = createCollectionRegistry<string, ItemValue>();
  registry.register({ key: "late", value: item("Late"), order: 20 });
  registry.register({ key: "early", value: item("Early"), order: 10 });

  assert.deepEqual(
    registry.items.value.map(({ key }) => key),
    ["early", "late"],
  );
  assert.throws(
    () => registry.register({ key: "partial", value: item("Partial") }),
    /VIZE_UI_COLLECTION_ORDER_PARTIAL/,
  );
  assert.throws(
    () => registry.register({ key: "duplicate", value: item("Duplicate"), order: 10 }),
    /VIZE_UI_COLLECTION_ORDER_DUPLICATE/,
  );
  assert.throws(
    () => registry.register({ key: "fraction", value: item("Fraction"), order: 1.5 }),
    /VIZE_UI_COLLECTION_ORDER_VALUE/,
  );
  assert.deepEqual(
    registry.items.value.map(({ key }) => key),
    ["early", "late"],
  );
  registry.dispose();
});

test("rejects unstable and duplicate runtime keys", () => {
  const registry = createCollectionRegistry<CollectionKey, ItemValue>();
  registry.register({ key: "valid", value: item("Valid") });

  assert.throws(
    () => registry.register({ key: "valid", value: item("Duplicate") }),
    /VIZE_UI_COLLECTION_KEY_DUPLICATE/,
  );
  assert.throws(
    () => registry.register({ key: "", value: item("Empty") }),
    /VIZE_UI_COLLECTION_KEY_VALUE/,
  );
  assert.throws(
    () => registry.register({ key: "bad\u0000key", value: item("Control") }),
    /VIZE_UI_COLLECTION_KEY_VALUE/,
  );
  assert.throws(
    () => registry.register({ key: Number.NaN, value: item("NaN") }),
    /VIZE_UI_COLLECTION_KEY_VALUE/,
  );
  assert.throws(
    () => registry.register({ key: -0, value: item("Negative zero") }),
    /VIZE_UI_COLLECTION_KEY_VALUE/,
  );
  registry.dispose();
});

test("navigates under skip and focusable disabled policies", () => {
  const skipped = createCollectionRegistry<string, ItemValue>();
  skipped.register({ key: "alpha", value: item("Alpha") });
  skipped.register({ key: "blocked", value: item("Blocked"), disabled: true });
  skipped.register({ key: "charlie", value: item("Charlie") });

  assert.equal(skipped.moveActive("next"), "alpha");
  assert.equal(skipped.moveActive("next"), "charlie");
  assert.equal(skipped.moveActive("next"), "charlie");
  assert.equal(skipped.moveActive("next", { loop: true }), "alpha");
  assert.equal(skipped.getNavigationKey("previous", { fromKey: "blocked" }), "alpha");
  assert.equal(skipped.getNavigationKey("next", { fromKey: "blocked" }), "charlie");
  assert.throws(() => skipped.setActiveKey("blocked"), /VIZE_UI_COLLECTION_KEY_DISABLED/);
  assert.throws(() => skipped.setActiveKey("missing"), /VIZE_UI_COLLECTION_KEY_MISSING/);

  const focusable = createCollectionRegistry<string, ItemValue>({
    disabledBehavior: "focusable",
  });
  focusable.register({ key: "alpha", value: item("Alpha") });
  focusable.register({ key: "blocked", value: item("Blocked"), disabled: true });
  assert.equal(focusable.moveActive("first"), "alpha");
  assert.equal(focusable.moveActive("next"), "blocked");
  assert.equal(focusable.setActiveKey("blocked"), false);

  skipped.dispose();
  focusable.dispose();
});

test("recovers active focus to the next item, then the previous item", () => {
  const changes: CollectionActiveChange<string>[] = [];
  const registry = createCollectionRegistry<string, ItemValue>({
    onActiveKeyChange: (change) => changes.push(change),
  });
  const alpha = registry.register({ key: "alpha", value: item("Alpha") });
  const bravo = registry.register({ key: "bravo", value: item("Bravo") });
  const charlie = registry.register({ key: "charlie", value: item("Charlie") });

  assert.equal(registry.setActiveKey("bravo"), true);
  assert.equal(bravo.unregister(), true);
  assert.equal(registry.activeKey.value, "charlie");
  assert.equal(charlie.unregister(), true);
  assert.equal(registry.activeKey.value, "alpha");
  assert.equal(alpha.unregister(), true);
  assert.equal(registry.activeKey.value, null);
  assert.deepEqual(
    changes.map(({ key, previousKey, reason }) => ({ key, previousKey, reason })),
    [
      { key: "bravo", previousKey: null, reason: "programmatic" },
      { key: "charlie", previousKey: "bravo", reason: "item-removed" },
      { key: "alpha", previousKey: "charlie", reason: "item-removed" },
      { key: null, previousKey: "alpha", reason: "item-removed" },
    ],
  );
  assert.equal(changes.every(Object.isFrozen), true);
  registry.dispose();
});

test("recovers synchronously when an active item becomes disabled", () => {
  const disabled = ref(false);
  const changes: CollectionActiveChange<string>[] = [];
  const registry = createCollectionRegistry<string, ItemValue>({
    onActiveKeyChange: (change) => changes.push(change),
  });
  registry.register({ key: "alpha", value: item("Alpha") });
  registry.register({ key: "bravo", value: item("Bravo"), disabled });
  registry.register({ key: "charlie", value: item("Charlie") });
  registry.setActiveKey("bravo");

  disabled.value = true;

  assert.equal(registry.activeKey.value, "charlie");
  assert.equal(changes.at(-1)?.reason, "item-disabled");
  registry.dispose();
});

test("cycles locale-aware typeahead and supports exact matching", () => {
  const registry = createCollectionRegistry<string, ItemValue>({
    collator: new Intl.Collator("de", { sensitivity: "base", usage: "search" }),
  });
  registry.register({ key: "beta", value: item("Beta"), textValue: "Beta" });
  registry.register({ key: "bravo", value: item("Bravo"), textValue: "Bravo" });
  registry.register({ key: "street", value: item("Street"), textValue: "Straße" });
  registry.register({ key: "empty", value: item("Empty"), textValue: "" });
  registry.register({
    key: "blocked",
    value: item("Blocked"),
    textValue: "Berlin",
    disabled: true,
  });
  registry.setActiveKey("beta");

  assert.equal(registry.findByTextValue("br"), "bravo");
  assert.equal(registry.moveActiveByTextValue("b"), "bravo");
  assert.equal(registry.moveActiveByTextValue("b"), "beta");
  assert.equal(registry.findByTextValue("strasse"), "street");
  assert.equal(registry.findByTextValue("beta", { match: "exact", fromKey: null }), "beta");
  assert.equal(registry.findByTextValue("bet", { match: "exact" }), null);
  assert.equal(registry.findByTextValue("   "), null);
  registry.dispose();
});

test("binds registration and registry cleanup to their Vue effect scopes", () => {
  const registry = createCollectionRegistry<string, ItemValue>();
  const itemScope = effectScope();
  const registration = itemScope.run(() =>
    registry.register({ key: "scoped", value: item("Scoped") }),
  );
  assert.ok(registration);
  assert.equal(registry.items.value.length, 1);

  itemScope.stop();

  assert.equal(registration.registered.value, false);
  assert.equal(registry.items.value.length, 0);

  const changes: CollectionActiveChange<string>[] = [];
  const registryScope = effectScope();
  const scopedRegistry = registryScope.run(() =>
    createCollectionRegistry<string, ItemValue>({
      onActiveKeyChange: (change) => changes.push(change),
    }),
  );
  assert.ok(scopedRegistry);
  scopedRegistry.register({ key: "active", value: item("Active") });
  scopedRegistry.setActiveKey("active");

  registryScope.stop();

  assert.equal(scopedRegistry.items.value.length, 0);
  assert.equal(scopedRegistry.activeKey.value, null);
  assert.equal(changes.at(-1)?.reason, "registry-disposed");
  assert.equal(scopedRegistry.dispose(), false);
  assert.throws(
    () => scopedRegistry.register({ key: "late", value: item("Late") }),
    /VIZE_UI_COLLECTION_DISPOSED/,
  );
  assert.throws(() => scopedRegistry.setActiveKey(null), /VIZE_UI_COLLECTION_DISPOSED/);
  registry.dispose();
});
