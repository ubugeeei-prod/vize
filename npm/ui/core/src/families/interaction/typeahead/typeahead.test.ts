import assert from "node:assert/strict";

import { effectScope, ref } from "vue";
import { test } from "vite-plus/test";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { createTypeahead, useTypeahead } from "./typeahead.ts";
import type { TypeaheadMatch } from "./typeahead.ts";

interface ItemValue {
  readonly label: string;
}

function createRegistry() {
  const registry = createCollectionRegistry<string, ItemValue>();
  for (const label of ["Alpha", "Beta", "Bravo", "New York", "👩‍💻 Developer"]) {
    registry.register({ key: label.toLowerCase(), value: { label }, textValue: label });
  }
  return registry;
}

function keyboard(key: string, init: KeyboardEventInit = {}): KeyboardEvent {
  return new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...init });
}

async function delay(milliseconds: number): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, milliseconds));
}

test("buffers locale-aware prefixes and publishes immutable match snapshots", () => {
  const registry = createRegistry();
  const matches: TypeaheadMatch<string>[] = [];
  const controller = createTypeahead({ registry, onMatch: (match) => matches.push(match) });
  registry.setActiveKey("beta");

  assert.equal(controller.input("b"), "bravo");
  assert.equal(controller.query.value, "b");
  assert.equal(matches[0]?.previousKey, "beta");
  assert.equal(matches[0]?.key, "bravo");
  assert.equal(matches[0]?.query, "b");
  assert.equal(matches[0]?.originalEvent, null);
  assert.ok(Object.isFrozen(matches[0]));

  controller.dispose();
  registry.dispose();
});

test("repeating one grapheme cycles matches while mixed input extends the query", () => {
  const registry = createRegistry();
  const controller = createTypeahead({ registry });
  registry.setActiveKey("beta");
  assert.equal(controller.input("b"), "bravo");
  assert.equal(controller.input("B"), "beta");
  assert.equal(controller.query.value, "B");

  controller.reset();
  assert.equal(controller.input("a"), "alpha");
  assert.equal(controller.input("l"), "alpha");
  assert.equal(controller.query.value, "al");
  controller.dispose();
  registry.dispose();
});

test("timeout starts a fresh query and reacts while a buffer is pending", async () => {
  const registry = createRegistry();
  const timeout = ref(100);
  const controller = createTypeahead({ registry, timeout });
  controller.input("a");
  timeout.value = 1;
  await delay(5);
  assert.equal(controller.query.value, "");

  controller.input("b");
  await delay(5);
  assert.equal(controller.query.value, "");
  controller.dispose();
  registry.dispose();
});

test("keyboard props consume graphemes but preserve activation Space and shortcuts", () => {
  const registry = createRegistry();
  const controller = createTypeahead({ registry });
  const composing = keyboard("a");
  Object.defineProperty(composing, "isComposing", { value: true });
  for (const event of [
    keyboard(" "),
    keyboard("Enter"),
    keyboard("Dead"),
    composing,
    keyboard("a", { metaKey: true }),
    keyboard("a", { ctrlKey: true }),
  ]) {
    controller.typeaheadProps.onKeydown(event);
    assert.equal(
      event.defaultPrevented,
      false,
      `unexpectedly consumed ${JSON.stringify({
        key: event.key,
        isComposing: event.isComposing,
        altKey: event.altKey,
        ctrlKey: event.ctrlKey,
        metaKey: event.metaKey,
        altGraph: event.getModifierState("AltGraph"),
      })}`,
    );
  }

  const international = keyboard("å", { altKey: true });
  controller.typeaheadProps.onKeydown(international);
  assert.equal(international.defaultPrevented, true);
  controller.reset();

  const first = keyboard("n");
  controller.typeaheadProps.onKeydown(first);
  assert.equal(first.defaultPrevented, true);
  const space = keyboard(" ");
  controller.typeaheadProps.onKeydown(space);
  assert.equal(space.defaultPrevented, true);
  assert.equal(controller.query.value, "n ");
  assert.equal(registry.activeKey.value, "new york");
  controller.dispose();
  registry.dispose();
});

test("allowSpace explicitly permits leading-space searches", () => {
  const registry = createRegistry();
  const controller = createTypeahead({ allowSpace: true, registry });
  const event = keyboard(" ");
  controller.typeaheadProps.onKeydown(event);
  assert.equal(event.defaultPrevented, true);
  assert.equal(controller.query.value, " ");
  controller.dispose();
  registry.dispose();
});

test("Unicode grapheme clusters remain atomic and multi-grapheme input is rejected", () => {
  const registry = createRegistry();
  const controller = createTypeahead({ registry });
  assert.equal(controller.input("👩‍💻"), "👩‍💻 developer");
  assert.equal(controller.query.value, "👩‍💻");
  assert.throws(() => controller.input("ab"), /VIZE_UI_TYPEAHEAD_INPUT/);
  assert.throws(() => controller.input(""), /VIZE_UI_TYPEAHEAD_INPUT/);
  controller.dispose();
  registry.dispose();
});

test("reactive disablement clears pending input and blocks new matches", () => {
  const registry = createRegistry();
  const disabled = ref(false);
  const controller = createTypeahead({ isDisabled: disabled, registry });
  controller.input("a");
  disabled.value = true;
  assert.equal(controller.query.value, "");
  assert.equal(controller.input("b"), null);
  assert.equal(registry.activeKey.value, "alpha");
  controller.dispose();
  registry.dispose();
});

test("invalid reactive sources clear pending ownership before diagnostics surface", () => {
  const registry = createRegistry();
  const disabled = ref<boolean | string>(false);
  const timeout = ref<number | string>(100);
  const controller = createTypeahead({
    isDisabled: disabled as never,
    registry,
    timeout: timeout as never,
  });
  controller.input("a");
  assert.throws(() => {
    disabled.value = "invalid";
  }, /VIZE_UI_TYPEAHEAD_OPTION.*isDisabled/);
  assert.equal(controller.query.value, "");

  disabled.value = false;
  controller.input("b");
  assert.throws(() => {
    timeout.value = "invalid";
  }, /VIZE_UI_TYPEAHEAD_OPTION.*timeout/);
  assert.equal(controller.query.value, "");
  controller.dispose();
  registry.dispose();
});

test("match callback failures leave collection and buffer state committed", () => {
  const registry = createRegistry();
  const controller = createTypeahead({
    registry,
    onMatch: () => {
      throw new Error("match failed");
    },
  });
  assert.throws(() => controller.input("a"), /match failed/);
  assert.equal(registry.activeKey.value, "alpha");
  assert.equal(controller.query.value, "a");
  controller.dispose();
  registry.dispose();
});

test("registry failures release the buffer and retain the original diagnostic", () => {
  const registry = createRegistry();
  const controller = createTypeahead({ registry });
  registry.dispose();
  assert.throws(() => controller.input("a"), /VIZE_UI_COLLECTION_DISPOSED/);
  assert.equal(controller.query.value, "");
  controller.dispose();
});

test("manual reset, disposal, and Vue scope ownership are terminal and leak-free", () => {
  const registry = createRegistry();
  const controller = createTypeahead({ registry });
  controller.input("a");
  assert.equal(controller.reset(), true);
  assert.equal(controller.reset(), false);
  controller.dispose();
  controller.dispose();
  assert.throws(() => controller.input("a"), /VIZE_UI_TYPEAHEAD_DISPOSED/);
  assert.throws(() => controller.reset(), /VIZE_UI_TYPEAHEAD_DISPOSED/);

  assert.throws(() => useTypeahead({ registry }), /VIZE_UI_TYPEAHEAD_SETUP/);
  const scope = effectScope();
  const scoped = scope.run(() => useTypeahead({ registry }))!;
  scope.stop();
  assert.throws(() => scoped.input("a"), /VIZE_UI_TYPEAHEAD_DISPOSED/);
  registry.dispose();
});

test("rejects malformed runtime options with stable diagnostics", () => {
  const registry = createRegistry();
  assert.throws(
    () => createTypeahead({ registry: {} as never }),
    /VIZE_UI_TYPEAHEAD_OPTION.*registry/,
  );
  assert.throws(
    () => createTypeahead({ registry, timeout: -1 }),
    /VIZE_UI_TYPEAHEAD_OPTION.*timeout/,
  );
  assert.throws(
    () => createTypeahead({ allowSpace: "yes" as never, registry }),
    /VIZE_UI_TYPEAHEAD_OPTION.*allowSpace/,
  );
  assert.throws(
    () => createTypeahead({ onMatch: "callback" as never, registry }),
    /VIZE_UI_TYPEAHEAD_OPTION.*onMatch/,
  );
  assert.throws(
    () => createTypeahead({ collator: {} as Intl.Collator, registry }),
    /VIZE_UI_TYPEAHEAD_OPTION.*collator/,
  );
  registry.dispose();
});
