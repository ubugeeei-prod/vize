import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { ref } from "vue";

import { createShortcutRegistry, parseShortcut } from "./shortcut.ts";
import type { ShortcutMatch, ShortcutRegistry } from "./shortcut.ts";

function keyboard(key: string, init: KeyboardEventInit = {}): KeyboardEvent {
  return new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...init });
}

function detached(options: Parameters<typeof createShortcutRegistry>[0] = {}): ShortcutRegistry {
  return createShortcutRegistry({ target: null, platform: "standard", ...options });
}

async function delay(milliseconds: number): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, milliseconds));
}

test("dispatches exact-modifier chords with an immutable match and cancels the native action", () => {
  const registry = detached();
  const matches: ShortcutMatch[] = [];
  registry.register({
    shortcut: "Mod+K",
    description: "Open palette",
    handler: (match) => matches.push(match),
  });

  const missed = keyboard("k");
  assert.equal(registry.input(missed), false);
  assert.equal(missed.defaultPrevented, false);
  const extra = keyboard("k", { ctrlKey: true, shiftKey: true });
  assert.equal(registry.input(extra), false);

  const hit = keyboard("K", { ctrlKey: true });
  assert.equal(registry.input(hit), true);
  assert.equal(hit.defaultPrevented, true);
  assert.equal(matches.length, 1);
  assert.ok(Object.isFrozen(matches[0]));
  assert.equal(matches[0]?.scope, "global");
  assert.equal(matches[0]?.description, "Open palette");
  assert.equal(matches[0]?.originalEvent, hit);
  registry.dispose();
});

test("resolves the Mod platform modifier per platform", () => {
  assert.deepEqual(parseShortcut("Mod+K", { platform: "apple" })[0], {
    key: "k",
    altKey: false,
    ctrlKey: false,
    metaKey: true,
    shiftKey: false,
  });
  const apple = createShortcutRegistry({ target: null, platform: "apple" });
  let dispatched = 0;
  apple.register({ shortcut: "Mod+K", handler: () => (dispatched += 1) });
  assert.equal(apple.input(keyboard("k", { ctrlKey: true })), false);
  assert.equal(apple.input(keyboard("k", { metaKey: true })), true);
  assert.equal(dispatched, 1);
  apple.dispose();
});

test("routes multi-chord sequences through reactive pending state", () => {
  const registry = detached();
  let dispatched = 0;
  registry.register({ shortcut: "G D", handler: () => (dispatched += 1) });

  const first = keyboard("g");
  assert.equal(registry.input(first), false);
  assert.equal(first.defaultPrevented, true);
  assert.equal(registry.pendingSequence.value.length, 1);
  assert.equal(registry.pendingSequence.value[0]?.key, "g");

  assert.equal(registry.input(keyboard("d")), true);
  assert.equal(dispatched, 1);
  assert.equal(registry.pendingSequence.value.length, 0);
  registry.dispose();
});

test("a non-continuing key resets the sequence and retries as a fresh start", () => {
  const registry = detached();
  const dispatched: string[] = [];
  registry.register({ shortcut: "G D", handler: () => dispatched.push("G D") });
  registry.register({ shortcut: "X", handler: () => dispatched.push("X") });

  assert.equal(registry.input(keyboard("g")), false);
  assert.equal(registry.input(keyboard("x")), true);
  assert.deepEqual(dispatched, ["X"]);
  assert.equal(registry.pendingSequence.value.length, 0);

  assert.equal(registry.input(keyboard("g")), false);
  assert.equal(registry.input(keyboard("g")), false);
  assert.equal(registry.pendingSequence.value.length, 1);
  assert.equal(registry.input(keyboard("d")), true);
  assert.deepEqual(dispatched, ["X", "G D"]);
  registry.dispose();
});

test("a pending sequence expires after the reactive timeout", async () => {
  const timeout = ref(20);
  const registry = detached({ sequenceTimeout: timeout });
  let dispatched = 0;
  registry.register({ shortcut: "G D", handler: () => (dispatched += 1) });

  registry.input(keyboard("g"));
  assert.equal(registry.pendingSequence.value.length, 1);
  await delay(40);
  assert.equal(registry.pendingSequence.value.length, 0);
  assert.equal(registry.input(keyboard("d")), false);
  assert.equal(dispatched, 0);
  registry.dispose();
});

test("the deepest active scope shadows global routing until released", () => {
  const registry = detached();
  const dispatched: string[] = [];
  registry.register({ shortcut: "Escape", handler: () => dispatched.push("global") });
  registry.register({
    shortcut: "Escape",
    scope: "dialog",
    handler: () => dispatched.push("dialog"),
  });
  registry.register({ shortcut: "Enter", scope: "dialog", handler: () => dispatched.push("only") });

  assert.equal(registry.input(keyboard("Enter")), false);
  registry.input(keyboard("Escape"));
  assert.deepEqual(dispatched, ["global"]);

  const release = registry.activateScope("dialog");
  assert.deepEqual(registry.activeScopes.value, ["dialog"]);
  registry.input(keyboard("Escape"));
  assert.equal(registry.input(keyboard("Enter")), true);
  assert.deepEqual(dispatched, ["global", "dialog", "only"]);

  release();
  release();
  assert.deepEqual(registry.activeScopes.value, []);
  registry.input(keyboard("Escape"));
  assert.deepEqual(dispatched, ["global", "dialog", "only", "global"]);
  registry.dispose();
});

test("reports same-scope conflicts and routes to the latest registration", () => {
  const registry = detached();
  const dispatched: string[] = [];
  registry.register({
    shortcut: "Ctrl+S",
    description: "Save",
    handler: () => dispatched.push("first"),
  });
  const releaseSecond = registry.register({
    shortcut: "Control+s",
    description: "Save As",
    handler: () => dispatched.push("second"),
  });

  const conflicts = registry.getConflicts();
  assert.equal(conflicts.length, 1);
  assert.equal(conflicts[0]?.scope, "global");
  assert.deepEqual(
    conflicts[0]?.bindings.map((binding) => binding.description),
    ["Save", "Save As"],
  );
  assert.ok(Object.isFrozen(conflicts[0]));

  registry.input(keyboard("s", { ctrlKey: true }));
  assert.deepEqual(dispatched, ["second"]);

  releaseSecond();
  assert.equal(registry.getConflicts().length, 0);
  assert.equal(registry.getBindings().length, 1);
  registry.input(keyboard("s", { ctrlKey: true }));
  assert.deepEqual(dispatched, ["second", "first"]);
  registry.dispose();
});

test("a false when gate skips the binding without consuming the event", () => {
  const registry = detached();
  const enabled = ref(false);
  let dispatched = 0;
  registry.register({ shortcut: "Mod+Z", when: enabled, handler: () => (dispatched += 1) });

  const skipped = keyboard("z", { ctrlKey: true });
  assert.equal(registry.input(skipped), false);
  assert.equal(skipped.defaultPrevented, false);
  enabled.value = true;
  assert.equal(registry.input(keyboard("z", { ctrlKey: true })), true);
  assert.equal(dispatched, 1);
  registry.dispose();
});

test("text-editing targets suppress bindings unless they opt in", () => {
  const registry = detached();
  const dispatched: string[] = [];
  registry.register({ shortcut: "K", handler: () => dispatched.push("plain") });
  registry.register({
    shortcut: "Mod+Enter",
    allowInEditable: true,
    handler: () => dispatched.push("editable"),
  });

  const input = document.createElement("input");
  document.body.append(input);
  const typed = keyboard("k");
  input.dispatchEvent(typed);
  assert.equal(registry.input(typed), false);
  assert.equal(typed.defaultPrevented, false);

  const submit = keyboard("Enter", { ctrlKey: true });
  input.dispatchEvent(submit);
  assert.equal(registry.input(submit), true);
  assert.deepEqual(dispatched, ["editable"]);

  const button = document.createElement("input");
  button.type = "checkbox";
  document.body.append(button);
  const onButton = keyboard("k");
  button.dispatchEvent(onButton);
  assert.equal(registry.input(onButton), true);
  assert.deepEqual(dispatched, ["editable", "plain"]);
  input.remove();
  button.remove();
  registry.dispose();
});

test("auto-repeat only re-dispatches chords that opt in", () => {
  const registry = detached();
  const dispatched: string[] = [];
  registry.register({
    shortcut: "ArrowDown",
    allowRepeat: true,
    handler: () => dispatched.push("down"),
  });
  registry.register({ shortcut: "Enter", handler: () => dispatched.push("enter") });

  assert.equal(registry.input(keyboard("ArrowDown", { repeat: true })), true);
  assert.equal(registry.input(keyboard("Enter", { repeat: true })), false);
  assert.equal(registry.input(keyboard("Enter")), true);
  assert.deepEqual(dispatched, ["down", "enter"]);
  registry.dispose();
});

test("modifier-only and composing input preserve pending state", () => {
  const registry = detached();
  let dispatched = 0;
  registry.register({ shortcut: "G D", handler: () => (dispatched += 1) });

  registry.input(keyboard("g"));
  assert.equal(registry.input(keyboard("Shift", { shiftKey: true })), false);
  assert.equal(registry.input(keyboard("d", { isComposing: true })), false);
  assert.equal(registry.pendingSequence.value.length, 1);
  assert.equal(registry.input(keyboard("d")), true);
  assert.equal(dispatched, 1);
  registry.dispose();
});

test("native listeners follow a reactive target across elements and shadow roots", () => {
  const first = document.createElement("section");
  const shadowHost = document.createElement("div");
  document.body.append(first, shadowHost);
  const shadowRoot = shadowHost.attachShadow({ mode: "open" });
  const target = ref<EventTarget | null>(first);
  const registry = createShortcutRegistry({ target, platform: "standard" });
  let dispatched = 0;
  registry.register({ shortcut: "Mod+B", handler: () => (dispatched += 1) });

  first.dispatchEvent(keyboard("b", { ctrlKey: true }));
  assert.equal(dispatched, 1);

  target.value = shadowRoot;
  first.dispatchEvent(keyboard("b", { ctrlKey: true }));
  assert.equal(dispatched, 1);
  shadowRoot.dispatchEvent(keyboard("b", { ctrlKey: true }));
  assert.equal(dispatched, 2);

  const detachExtra = registry.attach(first);
  first.dispatchEvent(keyboard("b", { ctrlKey: true }));
  assert.equal(dispatched, 3);
  detachExtra();

  target.value = null;
  shadowRoot.dispatchEvent(keyboard("b", { ctrlKey: true }));
  first.dispatchEvent(keyboard("b", { ctrlKey: true }));
  assert.equal(dispatched, 3);
  first.remove();
  shadowHost.remove();
  registry.dispose();
});

test("reactive disabled clears pending state synchronously and ignores input", () => {
  const isDisabled = ref(false);
  const registry = detached({ isDisabled });
  let dispatched = 0;
  registry.register({ shortcut: "G D", handler: () => (dispatched += 1) });

  registry.input(keyboard("g"));
  assert.equal(registry.pendingSequence.value.length, 1);
  isDisabled.value = true;
  assert.equal(registry.pendingSequence.value.length, 0);
  assert.equal(registry.input(keyboard("d")), false);
  isDisabled.value = false;
  registry.input(keyboard("g"));
  assert.equal(registry.input(keyboard("d")), true);
  assert.equal(dispatched, 1);
  registry.dispose();
});
