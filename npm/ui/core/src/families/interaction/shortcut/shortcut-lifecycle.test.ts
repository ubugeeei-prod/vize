import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope } from "vue";

import { createShortcutRegistry, parseShortcut, useShortcutRegistry } from "./shortcut.ts";
import type { ShortcutRegistry } from "./shortcut.ts";

function keyboard(key: string, init: KeyboardEventInit = {}): KeyboardEvent {
  return new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...init });
}

function detached(options: Parameters<typeof createShortcutRegistry>[0] = {}): ShortcutRegistry {
  return createShortcutRegistry({ target: null, platform: "standard", ...options });
}

test("stable diagnostics reject malformed patterns, options, and input", () => {
  assert.throws(() => parseShortcut(""), /VIZE_UI_SHORTCUT_PATTERN/);
  assert.throws(() => parseShortcut("Mod+"), /VIZE_UI_SHORTCUT_PATTERN/);
  assert.throws(() => parseShortcut("Q+K"), /VIZE_UI_SHORTCUT_PATTERN/);
  assert.throws(() => parseShortcut("Ctrl+Ctrl+K"), /VIZE_UI_SHORTCUT_PATTERN/);
  assert.throws(
    () => parseShortcut("Mod+K", { platform: "windows" as never }),
    /VIZE_UI_SHORTCUT_PLATFORM/,
  );

  const registry = detached();
  assert.throws(
    () => registry.register({ shortcut: "K", handler: null as never }),
    /VIZE_UI_SHORTCUT_OPTION/,
  );
  assert.throws(
    () => registry.register({ shortcut: [], handler: () => undefined }),
    /VIZE_UI_SHORTCUT_PATTERN/,
  );
  assert.throws(() => registry.activateScope("global"), /VIZE_UI_SHORTCUT_OPTION/);
  assert.throws(() => registry.input(null as never), /VIZE_UI_SHORTCUT_INPUT/);
  assert.throws(() => registry.attach(null as never), /VIZE_UI_SHORTCUT_OPTION/);
  registry.dispose();
  assert.throws(
    () => createShortcutRegistry({ target: null, sequenceTimeout: -1 }),
    /VIZE_UI_SHORTCUT_OPTION/,
  );
});

test("dispose and Vue scope teardown release listeners and become terminal", () => {
  const host = document.createElement("div");
  document.body.append(host);
  let dispatched = 0;
  const scope = effectScope();
  const registry = scope.run(() => useShortcutRegistry({ target: host, platform: "standard" }))!;
  registry.register({ shortcut: "Mod+K", handler: () => (dispatched += 1) });
  host.dispatchEvent(keyboard("k", { ctrlKey: true }));
  assert.equal(dispatched, 1);

  scope.stop();
  host.dispatchEvent(keyboard("k", { ctrlKey: true }));
  assert.equal(dispatched, 1);
  assert.throws(
    () => registry.input(keyboard("k", { ctrlKey: true })),
    /VIZE_UI_SHORTCUT_DISPOSED/,
  );
  assert.throws(
    () => registry.register({ shortcut: "K", handler: () => undefined }),
    /VIZE_UI_SHORTCUT_DISPOSED/,
  );
  registry.dispose();
  assert.throws(() => useShortcutRegistry(), /VIZE_UI_SHORTCUT_SETUP/);
  host.remove();
});
