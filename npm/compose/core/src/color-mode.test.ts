import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick } from "vue";

import { useColorMode, useDark } from "./color-mode.ts";
import type { ColorModeStorage } from "./color-mode.ts";
import type { MediaQueryHost } from "./media-query.ts";
import { asElement, FakeDocument } from "./testing/fake-dom.ts";

class MemoryStorage implements ColorModeStorage {
  readonly values = new Map<string, string>();
  getItem(key: string): string | null {
    return this.values.get(key) ?? null;
  }
  setItem(key: string, value: string): void {
    this.values.set(key, value);
  }
}

function systemHost(dark: boolean): MediaQueryHost & { set: (next: boolean) => void } {
  const list = Object.assign(new EventTarget(), { matches: dark, media: "" });
  return {
    matchMedia: () => list as unknown as MediaQueryList,
    set: (next) => {
      list.matches = next;
      list.dispatchEvent(new Event("change"));
    },
  };
}

void test("reads storage synchronously, applies classes, and persists changes", async () => {
  const element = new FakeDocument().createElement();
  const storage = new MemoryStorage();
  storage.setItem("vize-color-mode", "dark");
  const host = systemHost(false);
  const scope = effectScope();
  const color = scope.run(() =>
    useColorMode({ target: asElement(element), storage, host, modes: { sepia: "theme-sepia" } }),
  );
  assert.ok(color);
  assert.equal(color.mode.value, "dark");
  assert.equal(color.stored.value, "dark");
  assert.ok(element.classList.contains("dark"));

  color.mode.value = "sepia";
  await nextTick();
  assert.deepEqual([...element.classList.tokens], ["theme-sepia"]);
  assert.equal(storage.getItem("vize-color-mode"), "sepia");

  color.mode.value = "auto";
  await nextTick();
  assert.equal(color.state.value, "light");
  host.set(true);
  await nextTick();
  assert.deepEqual([color.system.value, color.state.value], ["dark", "dark"]);
  assert.ok(element.classList.contains("dark"));
  scope.stop();
});

void test("attribute mode, invalid stored values, and SSR fallbacks", () => {
  const element = new FakeDocument().createElement();
  const storage = new MemoryStorage();
  storage.setItem("theme", "neon");
  const scope = effectScope();
  const color = scope.run(() =>
    useColorMode({
      target: asElement(element),
      storage,
      storageKey: "theme",
      attribute: "data-theme",
      initialValue: "light",
      host: () => undefined,
      ssrSystem: "dark",
    }),
  );
  assert.equal(color?.mode.value, "light");
  assert.equal(element.getAttribute("data-theme"), "light");
  color!.mode.value = "auto";
  assert.equal(color?.state.value, "dark");
  scope.stop();

  const server = useColorMode({ storage: null, target: null, host: () => undefined });
  assert.deepEqual([server.mode.value, server.state.value], ["auto", "light"]);
});

void test("useDark stores auto when matching the system", async () => {
  const element = new FakeDocument().createElement();
  const storage = new MemoryStorage();
  const host = systemHost(true);
  const scope = effectScope();
  const dark = scope.run(() => useDark({ target: asElement(element), storage, host }));
  assert.equal(dark?.value, true);
  dark!.value = false;
  await nextTick();
  assert.equal(storage.getItem("vize-color-mode"), "light");
  assert.equal(element.classList.contains("dark"), false);
  dark!.value = true;
  assert.equal(storage.getItem("vize-color-mode"), "auto");
  scope.stop();
});
