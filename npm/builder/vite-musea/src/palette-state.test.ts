import assert from "node:assert/strict";
import test from "node:test";

import {
  clearSavedPaletteState,
  getPaletteStateStorageKey,
  normalizeSavedPaletteState,
  readSavedPaletteState,
  restorePaletteState,
  writeSavedPaletteState,
  type SavedPaletteState,
} from "../gallery/composables/paletteState.ts";

const controls = [
  {
    name: "tone",
    control: "select",
    default_value: "brand",
    required: false,
    options: [
      { label: "brand", value: "brand" },
      { label: "danger", value: "danger" },
    ],
  },
  {
    name: "size",
    control: "number",
    default_value: 4,
    required: false,
    options: [],
  },
];

void test("saved props editor state is restored against current palette controls", () => {
  const saved = normalizeSavedPaletteState({
    version: 1,
    values: {
      tone: "danger",
      size: 10,
      stale: "removed",
      "data-test": "card",
      toneCustom: "ignored",
    },
    customProps: [
      { name: "data-test", control: "text", default_value: "fallback" },
      { name: "tone", control: "text", default_value: "collision" },
      { name: "data-test", control: "text", default_value: "duplicate" },
    ],
    deletedPaletteProps: ["size", "missing"],
  });

  assert.ok(saved);

  const restored = restorePaletteState(controls, saved);

  assert.deepEqual(restored.values, {
    tone: "danger",
    size: 10,
    "data-test": "card",
  });
  assert.deepEqual(restored.customProps, [
    { name: "data-test", control: "text", default_value: "fallback" },
  ]);
  assert.deepEqual([...restored.deletedPaletteProps], ["size"]);
});

void test("saved props editor state round-trips through localStorage", () => {
  const storage = new MemoryStorage();
  const original = Object.getOwnPropertyDescriptor(globalThis, "localStorage");
  Object.defineProperty(globalThis, "localStorage", {
    value: storage,
    configurable: true,
  });

  const state: SavedPaletteState = {
    version: 1,
    values: { tone: "brand" },
    customProps: [],
    deletedPaletteProps: [],
  };

  try {
    writeSavedPaletteState("/src/Button.art.vue", state);

    assert.equal(storage.getItem(getPaletteStateStorageKey("/src/Button.art.vue")) !== null, true);
    assert.deepEqual(readSavedPaletteState("/src/Button.art.vue"), state);

    clearSavedPaletteState("/src/Button.art.vue");

    assert.equal(readSavedPaletteState("/src/Button.art.vue"), null);
  } finally {
    if (original) {
      Object.defineProperty(globalThis, "localStorage", original);
    } else {
      Reflect.deleteProperty(globalThis, "localStorage");
    }
  }
});

class MemoryStorage implements Storage {
  private readonly items = new Map<string, string>();

  get length(): number {
    return this.items.size;
  }

  clear(): void {
    this.items.clear();
  }

  getItem(key: string): string | null {
    return this.items.get(key) ?? null;
  }

  key(index: number): string | null {
    return [...this.items.keys()][index] ?? null;
  }

  removeItem(key: string): void {
    this.items.delete(key);
  }

  setItem(key: string, value: string): void {
    this.items.set(key, value);
  }
}
