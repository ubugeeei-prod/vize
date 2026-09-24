import { readonly, ref, shallowRef, toRaw, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Minimal structural view of an `IDBRequest`. */
export interface IndexedDBRequestLike<Result> extends Pick<EventTarget, "addEventListener"> {
  /** Result available after the `success` event. */
  readonly result: Result;

  /** Failure available after the `error` event. */
  readonly error: unknown;
}

/** Minimal structural view of an `IDBObjectStore` using out-of-line keys. */
export interface IndexedDBObjectStoreLike {
  /** Read the value stored under `key`. */
  readonly get: (key: IDBValidKey) => IndexedDBRequestLike<unknown>;

  /** Store `value` under `key`. */
  readonly put: (value: unknown, key: IDBValidKey) => IndexedDBRequestLike<IDBValidKey>;

  /** Delete `key`. */
  readonly delete: (key: IDBValidKey) => IndexedDBRequestLike<undefined>;

  /** List every key. */
  readonly getAllKeys: () => IndexedDBRequestLike<IDBValidKey[]>;

  /** Delete every entry. */
  readonly clear: () => IndexedDBRequestLike<undefined>;
}

/** Minimal structural view of an `IDBTransaction` (`complete`/`error`/`abort` events). */
export interface IndexedDBTransactionLike extends Pick<EventTarget, "addEventListener"> {
  /** Access an object store in the transaction scope. */
  readonly objectStore: (name: string) => IndexedDBObjectStoreLike;

  /** Failure available after `error`/`abort`. */
  readonly error: unknown;
}

/** Minimal structural view of an `IDBDatabase`. */
export interface IndexedDBDatabaseLike {
  /** Current schema version. */
  readonly version: number;

  /** Names of the existing object stores. */
  readonly objectStoreNames: { readonly contains: (name: string) => boolean };

  /** Create an object store; only valid during `upgradeneeded`. */
  readonly createObjectStore: (name: string) => unknown;

  /** Start a transaction on one store. */
  readonly transaction: (store: string, mode: "readonly" | "readwrite") => IndexedDBTransactionLike;

  /** Close the connection. */
  readonly close: () => void;
}

/** Minimal structural view of an `IDBFactory`; `window.indexedDB` satisfies it. */
export interface IndexedDBFactoryLike {
  /** Open (and possibly upgrade) a database; fires `upgradeneeded`, `success`, `error`. */
  readonly open: (name: string, version?: number) => IndexedDBRequestLike<IndexedDBDatabaseLike>;
}

/** Options for {@link createIndexedDBKeyval}. */
export interface IndexedDBKeyvalOptions {
  /**
   * Database name.
   *
   * @default "vize-keyval"
   */
  readonly database?: string;

  /**
   * Object store name; created on demand.
   *
   * @default "keyval"
   */
  readonly store?: string;

  /**
   * IndexedDB implementation.
   *
   * @default window.indexedDB when a browser window exists
   */
  readonly factory?: IndexedDBFactoryLike | null;
}

/** Promise-based key-value store returned by {@link createIndexedDBKeyval}. */
export interface IndexedDBKeyval {
  /** Read a structured-cloned value, or `undefined` when absent. */
  readonly get: (key: IDBValidKey) => Promise<unknown>;

  /** Store a structured-cloneable value. Resolves when the transaction commits. */
  readonly set: (key: IDBValidKey, value: unknown) => Promise<void>;

  /** Delete a key. Resolves when the transaction commits. */
  readonly delete: (key: IDBValidKey) => Promise<void>;

  /** List every key. */
  readonly keys: () => Promise<IDBValidKey[]>;

  /** Delete every entry. */
  readonly clear: () => Promise<void>;

  /** Close the connection; the next operation reopens it. */
  readonly close: () => void;
}

function browserFactory(): IndexedDBFactoryLike | undefined {
  try {
    return typeof window !== "undefined" ? (window.indexedDB ?? undefined) : undefined;
  } catch {
    // Accessing `indexedDB` throws in some privacy modes.
    return undefined;
  }
}

function request<Result>(pending: IndexedDBRequestLike<Result>): Promise<Result> {
  return new Promise((resolve, reject) => {
    pending.addEventListener("success", () => resolve(pending.result));
    pending.addEventListener("error", () => reject(pending.error));
  });
}

function committed(transaction: IndexedDBTransactionLike): Promise<void> {
  return new Promise((resolve, reject) => {
    transaction.addEventListener("complete", () => resolve());
    transaction.addEventListener("error", () => reject(transaction.error));
    transaction.addEventListener("abort", () => reject(transaction.error));
  });
}

function openDatabase(
  factory: IndexedDBFactoryLike,
  name: string,
  store: string,
  version?: number,
): Promise<IndexedDBDatabaseLike> {
  const opening = factory.open(name, version);
  opening.addEventListener("upgradeneeded", () => {
    const database = opening.result;
    if (!database.objectStoreNames.contains(store)) database.createObjectStore(store);
  });
  return request(opening).then((database) => {
    if (database.objectStoreNames.contains(store)) return database;
    // The database exists without our store: bump the version to create it.
    const next = database.version + 1;
    database.close();
    return openDatabase(factory, name, store, next);
  });
}

/**
 * Create a small promise-based key-value store on one IndexedDB object
 * store. The connection opens lazily on the first operation, the store is
 * created on demand, and writes resolve once their transaction commits.
 * Without IndexedDB (server rendering, privacy modes) every operation
 * rejects with a tagged `[VIZE_COMPOSE_INDEXED_DB_UNAVAILABLE]` error.
 *
 * @param options Database, store, and implementation.
 * @default options {}
 * @returns Key-value operations.
 */
export function createIndexedDBKeyval(options: IndexedDBKeyvalOptions = {}): IndexedDBKeyval {
  const name = options.database ?? "vize-keyval";
  const store = options.store ?? "keyval";
  let connection: Promise<IndexedDBDatabaseLike> | undefined;

  const database = (): Promise<IndexedDBDatabaseLike> => {
    if (connection !== undefined) return connection;
    const factory = options.factory === undefined ? browserFactory() : options.factory;
    if (!factory) {
      return Promise.reject(
        new Error("[VIZE_COMPOSE_INDEXED_DB_UNAVAILABLE] IndexedDB is not available"),
      );
    }
    const opened = openDatabase(factory, name, store);
    connection = opened;
    opened.catch(() => {
      if (connection === opened) connection = undefined;
    });
    return opened;
  };

  const read = async <Result>(
    operation: (objects: IndexedDBObjectStoreLike) => IndexedDBRequestLike<Result>,
  ): Promise<Result> => {
    const db = await database();
    return request(operation(db.transaction(store, "readonly").objectStore(store)));
  };

  const write = async (
    operation: (objects: IndexedDBObjectStoreLike) => unknown,
  ): Promise<void> => {
    const db = await database();
    const transaction = db.transaction(store, "readwrite");
    const done = committed(transaction);
    operation(transaction.objectStore(store));
    await done;
  };

  return {
    get: (key) => read((objects) => objects.get(key)),
    set: (key, value) => write((objects) => objects.put(value, key)),
    delete: (key) => write((objects) => objects.delete(key)),
    keys: () => read((objects) => objects.getAllKeys()),
    clear: () => write((objects) => objects.clear()),
    close: () => {
      const current = connection;
      connection = undefined;
      void current?.then(
        (db) => db.close(),
        () => undefined,
      );
    },
  };
}

/** Stable failure codes reported by {@link useIndexedDB}. */
export type IndexedDBErrorCode = "read-failed" | "invalid-value" | "write-failed";

/** Failure observed while synchronizing a value with IndexedDB. */
export interface IndexedDBFailure {
  /** Which synchronization step failed. */
  readonly code: IndexedDBErrorCode;

  /** Key involved in the failure. */
  readonly key: IDBValidKey;

  /** Exact rejection, or the rejected candidate for `"invalid-value"`. */
  readonly cause: unknown;
}

/** Options for {@link useIndexedDB}. */
export interface UseIndexedDBOptions<Value> {
  /**
   * Database name.
   *
   * @default "vize-keyval"
   */
  readonly database?: string;

  /**
   * Object store name; created on demand.
   *
   * @default "keyval"
   */
  readonly store?: string;

  /**
   * Reactive IndexedDB implementation. `null`/`undefined` keeps the default
   * value, which is also what happens during server rendering.
   *
   * @default window.indexedDB when a browser window exists
   */
  readonly factory?: MaybeRefOrGetter<IndexedDBFactoryLike | null | undefined>;

  /**
   * Validation hook applied to every loaded value.
   *
   * @default a structural check that the value has the default's kind
   */
  readonly validate?: (candidate: unknown) => candidate is Value;

  /**
   * Persist the default value when the key is absent.
   *
   * @default true
   */
  readonly writeDefaults?: boolean;

  /**
   * Watch nested mutations of object values.
   *
   * @default true
   */
  readonly deep?: boolean;

  /**
   * Write timing relative to component rendering.
   *
   * @default "pre"
   */
  readonly flush?: "pre" | "post" | "sync";

  /**
   * Observe failures. Failures never reject out of the composable.
   *
   * @default undefined
   */
  readonly onError?: (failure: IndexedDBFailure) => void;
}

/** Reactive state and controls returned by {@link useIndexedDB}. */
export interface IndexedDBControls<Value> {
  /** Writable value; assignments and nested mutations are persisted. */
  readonly state: Ref<Value>;

  /** Whether an IndexedDB implementation is attached. */
  readonly supported: Readonly<Ref<boolean>>;

  /** Whether the stored value for the current key has been loaded. */
  readonly ready: Readonly<Ref<boolean>>;

  /** Most recent failure, cleared by the next success. */
  readonly error: Readonly<ShallowRef<IndexedDBFailure | undefined>>;

  /** Reload the stored value. Never rejects. */
  readonly refresh: () => Promise<void>;

  /** Delete the key and restore the default value. Never rejects. */
  readonly remove: () => Promise<void>;
}

function sameKind<Value>(defaults: Value, candidate: unknown): candidate is Value {
  if (defaults === null || defaults === undefined) return true;
  if (defaults instanceof Map) return candidate instanceof Map;
  if (defaults instanceof Set) return candidate instanceof Set;
  if (defaults instanceof Date) return candidate instanceof Date;
  if (Array.isArray(defaults)) return Array.isArray(candidate);
  if (typeof defaults === "object") {
    return typeof candidate === "object" && candidate !== null && !Array.isArray(candidate);
  }
  return typeof candidate === typeof defaults;
}

/**
 * Synchronize a reactive value with one IndexedDB key.
 *
 * Values are stored by structured clone, so objects, arrays, `Map`, `Set`,
 * `Date`, typed arrays, and `Blob`s round-trip without a serializer. The
 * value is loaded asynchronously: `state` holds the default until `ready`
 * becomes `true`. Loaded values pass `validate` before they are accepted.
 * Assignments and nested mutations are written back; changing the reactive
 * `key` loads the new key (stale loads are discarded).
 *
 * Server rendering: without a browser `window` nothing is opened, `state`
 * is the default, and `ready` stays `false`, so server and client first
 * render identically. The connection closes and watchers stop with the
 * owning reactive scope. Failures never reject; they are exposed through
 * `error` and `onError`.
 *
 * @example
 * ```ts
 * const { state: draft, ready } = useIndexedDB("draft", { title: "", body: "" });
 * ```
 *
 * @typeParam Value Stored value type, inferred from `defaultValue`.
 * @param key Reactive IndexedDB key.
 * @param defaultValue Value used until loaded, and when absent or invalid.
 * @param options Database, store, implementation, and validation.
 * @default options {}
 * @returns The synchronized value and its controls.
 */
export function useIndexedDB<Value>(
  key: MaybeRefOrGetter<IDBValidKey>,
  defaultValue: Value,
  options: UseIndexedDBOptions<Value> = {},
): IndexedDBControls<Value> {
  // Stored values are plain structured-cloneable data, for which
  // `UnwrapRef<Value>` and `Value` coincide; `ref` keeps nested mutations
  // observable so they are persisted.
  const state = ref(defaultValue) as Ref<Value>;
  const supported = ref(false);
  const ready = ref(false);
  const error = shallowRef<IndexedDBFailure | undefined>(undefined);
  const validate =
    options.validate ??
    ((candidate: unknown): candidate is Value => sameKind(defaultValue, candidate));
  let keyval: IndexedDBKeyval | undefined;
  let generation = 0;
  let applying = false;
  let dirty = false;

  const fail = (code: IndexedDBErrorCode, cause: unknown): void => {
    const failure: IndexedDBFailure = { code, key: toValue(key), cause };
    error.value = failure;
    options.onError?.(failure);
  };

  const assign = (value: Value): void => {
    applying = true;
    try {
      state.value = value;
    } finally {
      applying = false;
    }
  };

  const persist = async (value: Value): Promise<void> => {
    if (keyval === undefined) return;
    try {
      await keyval.set(toValue(key), toRaw(value));
      error.value = undefined;
    } catch (cause) {
      fail("write-failed", cause);
    }
  };

  const refresh = async (): Promise<void> => {
    const current = ++generation;
    if (keyval === undefined) return;
    const store = keyval;
    let stored: unknown;
    try {
      stored = await store.get(toValue(key));
    } catch (cause) {
      if (current === generation) fail("read-failed", cause);
      return;
    }
    if (current !== generation) return;
    if (stored === undefined) {
      assign(defaultValue);
      ready.value = true;
      if (options.writeDefaults ?? true) await persist(defaultValue);
      return;
    }
    if (!validate(stored)) {
      fail("invalid-value", stored);
      assign(defaultValue);
    } else {
      error.value = undefined;
      assign(stored);
    }
    ready.value = true;
  };

  const remove = async (): Promise<void> => {
    generation += 1;
    assign(defaultValue);
    if (keyval === undefined) return;
    try {
      await keyval.delete(toValue(key));
      error.value = undefined;
    } catch (cause) {
      fail("write-failed", cause);
    }
  };

  let currentFactory: IndexedDBFactoryLike | undefined;
  const stopSource = watch(
    [
      () =>
        (options.factory === undefined ? browserFactory() : toValue(options.factory)) ?? undefined,
      () => toValue(key),
    ],
    ([factory]) => {
      ready.value = false;
      supported.value = factory !== undefined;
      if (factory !== currentFactory) {
        keyval?.close();
        currentFactory = factory;
        keyval =
          factory === undefined
            ? undefined
            : createIndexedDBKeyval({
                factory,
                ...(options.database === undefined ? {} : { database: options.database }),
                ...(options.store === undefined ? {} : { store: options.store }),
              });
      }
      if (keyval === undefined) {
        generation += 1;
        assign(defaultValue);
        return;
      }
      void refresh();
    },
    { immediate: true, flush: "sync" },
  );

  const stopDirty = watch(
    state,
    () => {
      if (!applying) dirty = true;
    },
    { deep: options.deep ?? true, flush: "sync" },
  );
  const stopWrite = watch(
    state,
    (value) => {
      if (!dirty) return;
      dirty = false;
      void persist(value);
    },
    { deep: options.deep ?? true, flush: options.flush ?? "pre" },
  );

  tryOnScopeDispose(() => {
    generation += 1;
    stopSource();
    stopDirty();
    stopWrite();
    keyval?.close();
    keyval = undefined;
  });

  return {
    state,
    supported: readonly(supported),
    ready: readonly(ready),
    error,
    refresh,
    remove,
  };
}
