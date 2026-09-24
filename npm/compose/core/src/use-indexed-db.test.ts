import assert from "node:assert/strict";
import { setTimeout as delay } from "node:timers/promises";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { createIndexedDBKeyval, useIndexedDB } from "./use-indexed-db.ts";
import type {
  IndexedDBDatabaseLike,
  IndexedDBFactoryLike,
  IndexedDBFailure,
  IndexedDBObjectStoreLike,
  IndexedDBRequestLike,
  IndexedDBTransactionLike,
} from "./use-indexed-db.ts";

class FakeRequest<Result> extends EventTarget implements IndexedDBRequestLike<Result> {
  result: Result;
  error: unknown = null;

  constructor(initial: Result) {
    super();
    this.result = initial;
  }

  succeed(result: Result): void {
    this.result = result;
    this.dispatchEvent(new Event("success"));
  }

  fail(error: unknown): void {
    this.error = error;
    this.dispatchEvent(new Event("error"));
  }
}

function later<Result>(result: Result, failure?: unknown): FakeRequest<Result> {
  const request = new FakeRequest(result);
  queueMicrotask(() => (failure === undefined ? request.succeed(result) : request.fail(failure)));
  return request;
}

class FakeTransaction extends EventTarget implements IndexedDBTransactionLike {
  error: unknown = null;
  readonly #entries: Map<IDBValidKey, unknown>;
  readonly #failure: unknown;

  constructor(entries: Map<IDBValidKey, unknown>, failure: unknown) {
    super();
    this.#entries = entries;
    this.#failure = failure;
    setTimeout(() => {
      if (failure === undefined) {
        this.dispatchEvent(new Event("complete"));
      } else {
        this.error = failure;
        this.dispatchEvent(new Event("abort"));
      }
    }, 0);
  }

  readonly objectStore = (_name: string): IndexedDBObjectStoreLike => {
    const entries = this.#entries;
    const failure = this.#failure;
    return {
      get: (key) => later(structuredClone(entries.get(key)), failure),
      put: (value, key) => {
        if (failure === undefined) entries.set(key, structuredClone(value));
        return later(key, failure);
      },
      delete: (key) => {
        entries.delete(key);
        return later(undefined, failure);
      },
      getAllKeys: () => later([...entries.keys()], failure),
      clear: () => {
        entries.clear();
        return later(undefined, failure);
      },
    };
  };
}

class FakeDatabase implements IndexedDBDatabaseLike {
  version = 0;
  closed = false;
  failure: unknown = undefined;
  readonly stores = new Map<string, Map<IDBValidKey, unknown>>();
  readonly objectStoreNames = { contains: (name: string) => this.stores.has(name) };

  readonly createObjectStore = (name: string): void => {
    this.stores.set(name, new Map());
  };

  readonly transaction = (store: string): IndexedDBTransactionLike => {
    const entries = this.stores.get(store);
    if (entries === undefined) throw new Error(`NotFoundError: ${store}`);
    return new FakeTransaction(entries, this.failure);
  };

  readonly close = (): void => {
    this.closed = true;
  };
}

class FakeFactory implements IndexedDBFactoryLike {
  readonly databases = new Map<string, FakeDatabase>();
  readonly connections: FakeDatabase[] = [];
  opens = 0;

  readonly open = (name: string, version?: number): IndexedDBRequestLike<IndexedDBDatabaseLike> => {
    this.opens += 1;
    const existing = this.databases.get(name);
    const database = existing ?? new FakeDatabase();
    this.databases.set(name, database);
    const request = new FakeRequest<IndexedDBDatabaseLike>(database);
    setTimeout(() => {
      const target = version ?? Math.max(database.version, 1);
      if (target > database.version) {
        database.version = target;
        request.dispatchEvent(new Event("upgradeneeded"));
      }
      database.closed = false;
      this.connections.push(database);
      request.succeed(database);
    }, 0);
    return request;
  };
}

const settle = () => delay(10);

void test("keyval stores, lists, deletes, and clears structured values", async () => {
  const factory = new FakeFactory();
  const keyval = createIndexedDBKeyval({ factory, database: "db", store: "items" });

  await keyval.set("a", { when: new Date(0), tags: new Set(["x"]) });
  await keyval.set("b", 2);
  const stored = await keyval.get("a");
  assert.ok(typeof stored === "object" && stored !== null && "tags" in stored);
  assert.ok(stored.tags instanceof Set);
  assert.deepEqual(await keyval.keys(), ["a", "b"]);
  await keyval.delete("a");
  assert.equal(await keyval.get("a"), undefined);
  await keyval.clear();
  assert.deepEqual(await keyval.keys(), []);
  assert.equal(factory.opens, 1, "the connection is opened once");
  keyval.close();
  await settle();
  assert.equal(factory.databases.get("db")?.closed, true);
});

void test("keyval creates a missing store by upgrading an existing database", async () => {
  const factory = new FakeFactory();
  await createIndexedDBKeyval({ factory, database: "db", store: "one" }).set("k", 1);
  const second = createIndexedDBKeyval({ factory, database: "db", store: "two" });
  await second.set("k", 2);
  assert.equal(factory.databases.get("db")?.version, 2);
  assert.equal(await second.get("k"), 2);
});

void test("keyval rejects with a tagged error without IndexedDB", async () => {
  const keyval = createIndexedDBKeyval({ factory: null });
  await assert.rejects(keyval.get("k"), /VIZE_COMPOSE_INDEXED_DB_UNAVAILABLE/);
});

void test("loads asynchronously and writes the default for absent keys", async () => {
  const factory = new FakeFactory();
  const draft = useIndexedDB("draft", { title: "" }, { factory });

  assert.equal(draft.supported.value, true);
  assert.equal(draft.ready.value, false);
  await settle();
  assert.equal(draft.ready.value, true);
  assert.deepEqual(factory.databases.get("vize-keyval")?.stores.get("keyval")?.get("draft"), {
    title: "",
  });
});

void test("reads existing values and persists nested mutations", async () => {
  const factory = new FakeFactory();
  await createIndexedDBKeyval({ factory }).set("list", ["a"]);
  const list = useIndexedDB("list", [] as string[], { factory });
  await settle();
  assert.deepEqual(list.state.value, ["a"]);

  list.state.value.push("b");
  await nextTick();
  await settle();
  assert.deepEqual(await createIndexedDBKeyval({ factory }).get("list"), ["a", "b"]);
});

void test("rejects values of the wrong kind without overwriting them", async () => {
  const factory = new FakeFactory();
  await createIndexedDBKeyval({ factory }).set("n", "text");
  const failures: IndexedDBFailure[] = [];
  const count = useIndexedDB("n", 0, { factory, onError: (failure) => failures.push(failure) });
  await settle();
  assert.equal(count.state.value, 0);
  assert.equal(count.error.value?.code, "invalid-value");
  assert.equal(failures.length, 1);
  await nextTick();
  await settle();
  assert.equal(await createIndexedDBKeyval({ factory }).get("n"), "text");
});

void test("reloads when the reactive key changes and discards stale loads", async () => {
  const factory = new FakeFactory();
  const seed = createIndexedDBKeyval({ factory });
  await seed.set("user:1", "alice");
  await seed.set("user:2", "bob");
  const id = ref(1);
  const name = useIndexedDB(() => `user:${String(id.value)}`, "", { factory });
  id.value = 2;
  await settle();
  assert.equal(name.state.value, "bob");
});

void test("remove deletes the key and restores the default", async () => {
  const factory = new FakeFactory();
  await createIndexedDBKeyval({ factory }).set("n", 5);
  const count = useIndexedDB("n", 1, { factory, writeDefaults: false });
  await settle();
  assert.equal(count.state.value, 5);
  await count.remove();
  await nextTick();
  await settle();
  assert.equal(count.state.value, 1);
  assert.equal(await createIndexedDBKeyval({ factory }).get("n"), undefined);
});

void test("reports write failures without rejecting", async () => {
  const factory = new FakeFactory();
  const count = useIndexedDB("n", 0, { factory });
  await settle();
  const database = factory.databases.get("vize-keyval");
  assert.ok(database);
  database.failure = new Error("QuotaExceededError");
  count.state.value = 2;
  await nextTick();
  await settle();
  assert.equal(count.error.value?.code, "write-failed");
});

void test("closes the connection with the scope", async () => {
  const factory = new FakeFactory();
  const scope = effectScope();
  scope.run(() => useIndexedDB("n", 0, { factory }));
  await settle();
  scope.stop();
  await settle();
  assert.equal(factory.databases.get("vize-keyval")?.closed, true);
});

void test("stays on the default without IndexedDB", () => {
  const count = useIndexedDB("n", 3, { factory: null });
  assert.equal(count.supported.value, false);
  assert.equal(count.ready.value, false);
  assert.equal(count.state.value, 3);
});

void test("server rendering opens nothing and renders the default", async () => {
  const state = await renderComposableOnServer(() => {
    const draft = useIndexedDB("draft", { title: "" });
    return { draft: draft.state, ready: draft.ready, supported: draft.supported };
  });
  assert.equal(state, '{"draft":{"title":""},"ready":false,"supported":false}');
});
