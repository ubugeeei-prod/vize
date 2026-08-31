import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { formatShortcut, getShortcutKeycaps, parseShortcut } from "./shortcut.ts";

test("apple symbol keycaps follow the platform modifier order without separators", () => {
  assert.equal(formatShortcut("Mod+Shift+K", { platform: "apple" }), "⇧⌘K");
  assert.equal(formatShortcut("Ctrl+Alt+Shift+Mod+2", { platform: "apple" }), "⌃⌥⇧⌘2");
  assert.deepEqual(getShortcutKeycaps("Mod+Shift+K", { platform: "apple" }), [["⇧", "⌘", "K"]]);
});

test("standard platforms format spelled-out keycaps joined with plus", () => {
  assert.equal(formatShortcut("Mod+Shift+K", { platform: "standard" }), "Ctrl+Shift+K");
  assert.equal(formatShortcut("Alt+F4", { platform: "standard" }), "Alt+F4");
  assert.deepEqual(getShortcutKeycaps("Mod+Shift+K", { platform: "standard" }), [
    ["Ctrl", "Shift", "K"],
  ]);
});

test("named keys map to platform glyphs and stable shared keycaps", () => {
  assert.equal(formatShortcut("Mod+Enter", { platform: "apple" }), "⌘↩");
  assert.equal(formatShortcut("Mod+Enter", { platform: "standard" }), "Ctrl+Enter");
  assert.equal(formatShortcut("Escape", { platform: "apple" }), "⎋");
  assert.equal(formatShortcut("Escape", { platform: "standard" }), "Esc");
  assert.equal(formatShortcut("Mod+ArrowUp", { platform: "standard" }), "Ctrl+↑");
  assert.equal(formatShortcut("Shift+Space", { platform: "standard" }), "Shift+Space");
  assert.equal(formatShortcut("Ctrl++", { platform: "standard" }), "Ctrl++");
});

test("explicit style overrides the platform default in both directions", () => {
  assert.equal(formatShortcut("Mod+K", { platform: "apple", style: "text" }), "Command+K");
  assert.equal(formatShortcut("Alt+K", { platform: "apple", style: "text" }), "Option+K");
  assert.equal(formatShortcut("Mod+K", { platform: "standard", style: "symbol" }), "Ctrl+K");
});

test("sequence steps join with a single space and accept parsed sequences", () => {
  assert.equal(formatShortcut("G D", { platform: "standard" }), "G D");
  const sequence = parseShortcut("Mod+K Mod+S", { platform: "standard" });
  assert.equal(formatShortcut(sequence, { platform: "standard" }), "Ctrl+K Ctrl+S");
  assert.deepEqual(getShortcutKeycaps(sequence, { platform: "standard" }), [
    ["Ctrl", "K"],
    ["Ctrl", "S"],
  ]);
});

test("keycap structures are frozen and reject invalid styles", () => {
  const keycaps = getShortcutKeycaps("Mod+K", { platform: "apple" });
  assert.ok(Object.isFrozen(keycaps));
  assert.ok(Object.isFrozen(keycaps[0]));
  assert.throws(
    () => formatShortcut("Mod+K", { platform: "apple", style: "fancy" as never }),
    /VIZE_UI_SHORTCUT_FORMAT/,
  );
});
