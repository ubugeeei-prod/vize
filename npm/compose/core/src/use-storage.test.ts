import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import {
  inferStorageSerializerKind,
  storageSerializers,
  useSessionStorage,
  useStorage,
} from "./use-storage.ts";
import type { StorageFailure, StorageLike } from "./use-storage.ts";

class MemoryStorage implements StorageLike {
  readonly items = new Map<string, string>();
  writes = 0;
  failWrites = false;

  getItem(key: string): string | null {
    return this.items.get(key) ?? null;
  }

  setItem(key: string, value: string): void {
    if (this.failWrites) throw new Error("QuotaExceededError");
    this.writes += 1;
    this.items.set(key, value);
  }

  removeItem(key: string): void {
    this.items.delete(key);
  }
}

class FakeStorageEvent extends Event {
  readonly key: string | null;
  readonly newValue: string | null;
  readonly storageArea: StorageLike;

  constructor(storageArea: StorageLike, key: string | null, newValue: string | null) {
    super("storage");
    this.storageArea = storageArea;
    this.key = key;
    this.newValue = newValue;
  }
}

void test("infers the serializer from the default value", () => {
  assert.equal(inferStorageSerializerKind(1), "number");
  assert.equal(inferStorageSerializerKind(true), "boolean");
  assert.equal(inferStorageSerializerKind("x"), "string");
  assert.equal(inferStorageSerializerKind(1n), "bigint");
  assert.equal(inferStorageSerializerKind(new Map()), "map");
  assert.equal(inferStorageSerializerKind(new Set()), "set");
  assert.equal(inferStorageSerializerKind(new Date(0)), "date");
  assert.equal(inferStorageSerializerKind({ a: 1 }), "object");
  assert.equal(inferStorageSerializerKind(null), "object");
  assert.equal(inferStorageSerializerKind([]), "object");
});

void test("built-in serializers round-trip their value types", () => {
  const date = new Date("2026-01-02T03:04:05.000Z");
  assert.equal(
    storageSerializers.date.read(storageSerializers.date.write(date)).getTime(),
    date.getTime(),
  );
  assert.deepEqual(
    [...storageSerializers.map.read(storageSerializers.map.write(new Map([["a", 1]])))],
    [["a", 1]],
  );
  assert.deepEqual(
    [...storageSerializers.set.read(storageSerializers.set.write(new Set([1, 2])))],
    [1, 2],
  );
  assert.equal(storageSerializers.bigint.read(storageSerializers.bigint.write(9n)), 9n);
  assert.equal(storageSerializers.boolean.read("true"), true);
  assert.equal(storageSerializers.number.read("1.5"), 1.5);
});

void test("reads existing data and writes the default for absent keys", () => {
  const storage = new MemoryStorage();
  storage.items.set("count", "4");
  const count = useStorage("count", 0, { storage, eventTarget: null });
  const fresh = useStorage("fresh", { theme: "light" }, { storage, eventTarget: null });

  assert.equal(count.state.value, 4);
  assert.equal(count.supported.value, true);
  assert.deepEqual(fresh.state.value, { theme: "light" });
  assert.equal(storage.items.get("fresh"), '{"theme":"light"}');
});

void test("does not write defaults when writeDefaults is false", () => {
  const storage = new MemoryStorage();
  useStorage("x", 1, { storage, eventTarget: null, writeDefaults: false });
  assert.equal(storage.items.has("x"), false);
});

void test("persists changes, including nested mutations", async () => {
  const storage = new MemoryStorage();
  const scope = effectScope();
  const controls = scope.run(() =>
    useStorage("prefs", { tags: ["a"] }, { storage, eventTarget: null }),
  );
  assert.ok(controls);

  controls.state.value.tags.push("b");
  await nextTick();
  assert.equal(storage.items.get("prefs"), '{"tags":["a","b"]}');

  controls.state.value = { tags: [] };
  await nextTick();
  assert.equal(storage.items.get("prefs"), '{"tags":[]}');

  scope.stop();
  controls.state.value = { tags: ["late"] };
  await nextTick();
  assert.equal(storage.items.get("prefs"), '{"tags":[]}');
});

void test("round-trips maps, sets, and dates through inferred serializers", async () => {
  const storage = new MemoryStorage();
  const map = useStorage("map", new Map<string, number>(), { storage, eventTarget: null });
  map.state.value.set("a", 1);
  await nextTick();
  assert.equal(storage.items.get("map"), '[["a",1]]');

  const reread = useStorage("map", new Map<string, number>(), { storage, eventTarget: null });
  assert.equal(reread.state.value.get("a"), 1);

  storage.items.set("when", "2026-09-25T00:00:00.000Z");
  const when = useStorage("when", new Date(0), { storage, eventTarget: null });
  assert.equal(when.state.value.toISOString(), "2026-09-25T00:00:00.000Z");
});

void test("rejects values of the wrong kind and falls back to the default", () => {
  const storage = new MemoryStorage();
  storage.items.set("count", "not-a-number");
  storage.items.set("json", "{broken");
  const failures: StorageFailure[] = [];

  const count = useStorage("count", 7, {
    storage,
    eventTarget: null,
    onError: (failure) => failures.push(failure),
  });
  const json = useStorage("json", { a: 1 }, { storage, eventTarget: null });

  assert.equal(count.state.value, 7);
  assert.equal(count.error.value?.code, "invalid-value");
  assert.deepEqual(
    failures.map((failure) => failure.code),
    ["invalid-value"],
  );
  assert.deepEqual(json.state.value, { a: 1 });
  assert.equal(json.error.value?.code, "read-failed");
  assert.equal(storage.items.get("count"), "not-a-number", "rejected data is left untouched");
});

void test("runs the schema validation hook on every decoded value", () => {
  type Theme = "light" | "dark";
  const isTheme = (candidate: unknown): candidate is Theme =>
    candidate === "light" || candidate === "dark";
  const storage = new MemoryStorage();
  storage.items.set("theme", "sepia");

  const theme = useStorage<Theme>("theme", "light", {
    storage,
    eventTarget: null,
    validate: isTheme,
  });
  assert.equal(theme.state.value, "light");
  assert.equal(theme.error.value?.code, "invalid-value");

  storage.items.set("theme", "dark");
  theme.refresh();
  assert.equal(theme.state.value, "dark");
  assert.equal(theme.error.value, undefined);
});

void test("merges newly added default keys when requested", () => {
  const storage = new MemoryStorage();
  storage.items.set("settings", '{"a":2}');
  const shallow = useStorage(
    "settings",
    { a: 1, b: true },
    { storage, eventTarget: null, mergeDefaults: true },
  );
  assert.deepEqual(shallow.state.value, { a: 2, b: true });

  const custom = useStorage(
    "settings",
    { a: 1, b: true },
    {
      storage,
      eventTarget: null,
      mergeDefaults: (stored, defaults) => ({ ...stored, b: !defaults.b }),
    },
  );
  assert.deepEqual(custom.state.value, { a: 2, b: false });
});

void test("reports write failures without throwing", async () => {
  const storage = new MemoryStorage();
  const counter = useStorage("n", 0, { storage, eventTarget: null });
  storage.failWrites = true;

  counter.state.value = 5;
  await nextTick();
  assert.equal(counter.error.value?.code, "write-failed");
  assert.equal(counter.state.value, 5);
});

void test("follows storage events from other documents", () => {
  const storage = new MemoryStorage();
  const other = new MemoryStorage();
  const events = new EventTarget();
  const counter = useStorage("n", 0, { storage, eventTarget: events });

  events.dispatchEvent(new FakeStorageEvent(storage, "n", "9"));
  assert.equal(counter.state.value, 9);

  events.dispatchEvent(new FakeStorageEvent(other, "n", "3"));
  events.dispatchEvent(new FakeStorageEvent(storage, "other", "3"));
  assert.equal(counter.state.value, 9);

  events.dispatchEvent(new FakeStorageEvent(storage, null, null));
  assert.equal(counter.state.value, 0);
});

void test("synchronizes instances in the same document without write loops", async () => {
  const storage = new MemoryStorage();
  const events = new EventTarget();
  const first = useStorage("shared", "a", { storage, eventTarget: events });
  const second = useStorage("shared", "a", { storage, eventTarget: events });
  const writesBefore = storage.writes;

  first.state.value = "b";
  await nextTick();
  assert.equal(second.state.value, "b");
  await nextTick();
  assert.equal(storage.writes, writesBefore + 1);

  second.remove();
  assert.equal(storage.items.has("shared"), false);
  assert.equal(first.state.value, "a");
  await nextTick();
  assert.equal(storage.items.has("shared"), false, "removal is not undone by a write");
});

void test("re-reads when the reactive key changes", async () => {
  const storage = new MemoryStorage();
  storage.items.set("user:1", '"alice"');
  storage.items.set("user:2", '"bob"');
  const id = ref(1);
  const name = useStorage(() => `user:${String(id.value)}`, "", {
    storage,
    eventTarget: null,
    serializer: {
      read: (raw) => String(JSON.parse(raw)),
      write: (value) => JSON.stringify(value),
    },
  });

  assert.equal(name.state.value, "alice");
  id.value = 2;
  await nextTick();
  assert.equal(name.state.value, "bob");
});

void test("defers the first read past the first flush for hydration", async () => {
  const storage = new MemoryStorage();
  storage.items.set("n", "3");
  const counter = useStorage("n", 0, { storage, eventTarget: null, initialRead: "post-flush" });

  assert.equal(counter.state.value, 0);
  await nextTick();
  assert.equal(counter.state.value, 3);
});

void test("stays on the default without a backend", () => {
  const counter = useStorage("n", 1, { storage: null, eventTarget: null });
  const session = useSessionStorage("n", 1);

  assert.equal(counter.supported.value, false);
  assert.equal(counter.state.value, 1);
  assert.equal(session.supported.value, false);
});

void test("server rendering uses defaults and never touches server storage", async () => {
  const state = await renderComposableOnServer(() => {
    const local = useStorage("n", 2);
    const session = useSessionStorage("s", { a: 1 });
    return { local: local.state, session: session.state, supported: local.supported };
  });

  assert.equal(state, '{"local":2,"session":{"a":1},"supported":false}');
});
