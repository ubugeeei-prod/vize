import { readonly, ref, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Minimal synchronous key-value store compatible with the Web Storage API. */
export interface StorageLike {
  /** Read the raw string stored for `key`, or `null` when absent. */
  readonly getItem: (key: string) => string | null;

  /** Store `value` under `key`. May throw (for example on quota exhaustion). */
  readonly setItem: (key: string, value: string) => void;

  /** Remove `key`. Removing an absent key is a no-op. */
  readonly removeItem: (key: string) => void;
}

/** Converts one value type to and from its stored string representation. */
export interface StorageSerializer<Value> {
  /** Decode a stored string. Throwing marks the stored value as unreadable. */
  readonly read: (raw: string) => Value;

  /** Encode a value for storage. */
  readonly write: (value: Value) => string;
}

/** Built-in serializers selected by {@link inferStorageSerializerKind}. */
export type StorageSerializerKind =
  | "bigint"
  | "boolean"
  | "date"
  | "map"
  | "number"
  | "object"
  | "set"
  | "string";

/** Stable machine-readable failure codes reported by {@link useStorage}. */
export type StorageErrorCode = "read-failed" | "invalid-value" | "write-failed" | "remove-failed";

/** Failure observed while synchronizing a value with storage. */
export interface StorageFailure {
  /** Which synchronization step failed. */
  readonly code: StorageErrorCode;

  /** Storage key involved in the failure. */
  readonly key: string;

  /** Exact thrown value, or the rejected candidate for `"invalid-value"`. */
  readonly cause: unknown;
}

/**
 * Validate a decoded candidate before it replaces the reactive value.
 *
 * Return `true` to accept the candidate. A type guard narrows `unknown` to
 * the value type; plug schema libraries in here (for example
 * `(candidate) => schema.safeParse(candidate).success`).
 */
export type StorageValidator<Value> = (candidate: unknown) => candidate is Value;

/** Options for {@link useStorage}. */
export interface UseStorageOptions<Value> {
  /**
   * Reactive storage backend. `null`/`undefined` keeps the composable on its
   * default value, which is also what happens during server rendering.
   *
   * @default window.localStorage when a browser window exists
   */
  readonly storage?: MaybeRefOrGetter<StorageLike | null | undefined>;

  /**
   * Event target receiving cross-document `storage` events and same-document
   * synchronization events between instances sharing a key.
   *
   * @default window when a browser window exists
   */
  readonly eventTarget?: MaybeRefOrGetter<EventTarget | null | undefined>;

  /**
   * Explicit serializer. When omitted the serializer is inferred from the
   * default value (see {@link inferStorageSerializerKind}).
   *
   * @default inferred from the default value
   */
  readonly serializer?: StorageSerializer<Value>;

  /**
   * Schema validation hook applied to every decoded value. Rejected values
   * fall back to the default and report an `"invalid-value"` failure.
   *
   * @default a structural check that the stored value has the default's kind
   */
  readonly validate?: StorageValidator<Value>;

  /**
   * Merge a stored object with the default so newly added default keys are
   * present. `true` performs a shallow merge; a function merges explicitly.
   *
   * @default false
   */
  readonly mergeDefaults?: boolean | ((stored: Value, defaults: Value) => Value);

  /**
   * Persist the default value when the key is absent.
   *
   * @default true
   */
  readonly writeDefaults?: boolean;

  /**
   * Follow changes made by other documents and other instances.
   *
   * @default true
   */
  readonly listenToStorageChanges?: boolean;

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
   * When the stored value is first read. `"post-flush"` keeps the default
   * through the first render so server-rendered markup hydrates without a
   * mismatch, then reads storage after mounting.
   *
   * @default "sync"
   */
  readonly initialRead?: "sync" | "post-flush";

  /**
   * Observe read, validation, write, and removal failures. Failures never
   * throw out of the composable.
   *
   * @default undefined
   */
  readonly onError?: (failure: StorageFailure) => void;
}

/** Reactive state and controls returned by {@link useStorage}. */
export interface StorageControls<Value> {
  /** Writable value synchronized with storage. */
  readonly state: Ref<Value>;

  /** Whether a storage backend is currently attached. */
  readonly supported: Readonly<Ref<boolean>>;

  /** Most recent synchronization failure, cleared by the next success. */
  readonly error: Readonly<ShallowRef<StorageFailure | undefined>>;

  /** Re-read the stored value, replacing the reactive value. */
  readonly refresh: () => void;

  /** Remove the key from storage and restore the default value. */
  readonly remove: () => void;
}

const SAME_DOCUMENT_EVENT = "vize:storage";

interface StorageCodec {
  readonly read: (raw: string) => unknown;
  readonly write: (value: unknown) => string;
}

function jsonRead(raw: string): unknown {
  return JSON.parse(raw);
}

const codecs: Readonly<Record<StorageSerializerKind, StorageCodec>> = {
  bigint: { read: (raw) => BigInt(raw), write: (value) => String(value) },
  boolean: { read: (raw) => raw === "true", write: (value) => String(value) },
  date: {
    read: (raw) => new Date(raw),
    write: (value) => (value instanceof Date ? value.toISOString() : String(value)),
  },
  map: {
    read: (raw) => {
      const entries = jsonRead(raw);
      return new Map(Array.isArray(entries) ? entries.filter(isEntry) : []);
    },
    write: (value) => JSON.stringify(value instanceof Map ? [...value.entries()] : []),
  },
  number: { read: (raw) => Number.parseFloat(raw), write: (value) => String(value) },
  object: { read: jsonRead, write: (value) => JSON.stringify(value) },
  set: {
    read: (raw) => {
      const values = jsonRead(raw);
      return new Set(Array.isArray(values) ? values : []);
    },
    write: (value) => JSON.stringify(value instanceof Set ? [...value] : []),
  },
  string: { read: (raw) => raw, write: (value) => String(value) },
};

function isEntry(candidate: unknown): candidate is readonly [unknown, unknown] {
  return Array.isArray(candidate) && candidate.length === 2;
}

/**
 * Typed built-in serializers, usable directly as the `serializer` option.
 *
 * `object` round-trips JSON-compatible values; `map` and `set` store their
 * entries as JSON arrays; `date` stores an ISO-8601 string.
 */
export const storageSerializers: {
  /** Decimal `bigint` text. */
  readonly bigint: StorageSerializer<bigint>;
  /** `"true"` / `"false"`. */
  readonly boolean: StorageSerializer<boolean>;
  /** ISO-8601 timestamp. */
  readonly date: StorageSerializer<Date>;
  /** JSON array of `[key, value]` entries. */
  readonly map: StorageSerializer<Map<unknown, unknown>>;
  /** Decimal number text. */
  readonly number: StorageSerializer<number>;
  /** JSON text. */
  readonly object: StorageSerializer<unknown>;
  /** JSON array of members. */
  readonly set: StorageSerializer<Set<unknown>>;
  /** Raw string. */
  readonly string: StorageSerializer<string>;
} = {
  bigint: {
    read: (raw) => BigInt(raw),
    write: (value) => value.toString(),
  },
  boolean: { read: (raw) => raw === "true", write: (value) => String(value) },
  date: { read: (raw) => new Date(raw), write: (value) => value.toISOString() },
  map: {
    read: (raw) => {
      const entries = jsonRead(raw);
      return new Map(Array.isArray(entries) ? entries.filter(isEntry) : []);
    },
    write: (value) => JSON.stringify([...value.entries()]),
  },
  number: { read: (raw) => Number.parseFloat(raw), write: (value) => String(value) },
  object: { read: jsonRead, write: (value) => JSON.stringify(value) },
  set: {
    read: (raw) => {
      const values = jsonRead(raw);
      return new Set(Array.isArray(values) ? values : []);
    },
    write: (value) => JSON.stringify([...value]),
  },
  string: { read: (raw) => raw, write: (value) => value },
};

/**
 * Select the built-in serializer kind for a default value.
 *
 * `null`, arrays, and plain objects use JSON (`"object"`); `Map`, `Set`, and
 * `Date` use their dedicated kinds; primitives use their `typeof`.
 * Functions and symbols cannot be stored and fall back to `"string"`.
 *
 * @param value Default value supplied to {@link useStorage}.
 * @returns The inferred serializer kind.
 */
export function inferStorageSerializerKind(value: unknown): StorageSerializerKind {
  if (value instanceof Map) return "map";
  if (value instanceof Set) return "set";
  if (value instanceof Date) return "date";
  switch (typeof value) {
    case "bigint":
      return "bigint";
    case "boolean":
      return "boolean";
    case "number":
      return "number";
    case "object":
      return "object";
    default:
      return "string";
  }
}

function sameKind<Value>(defaults: Value, candidate: unknown): candidate is Value {
  const kind = inferStorageSerializerKind(defaults);
  if (kind === "object") {
    if (defaults === null) return true;
    if (Array.isArray(defaults)) return Array.isArray(candidate);
    return typeof candidate === "object" && candidate !== null && !Array.isArray(candidate);
  }
  if (kind === "number") return typeof candidate === "number" && !Number.isNaN(candidate);
  if (kind === "date") return candidate instanceof Date && !Number.isNaN(candidate.getTime());
  return inferStorageSerializerKind(candidate) === kind;
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function shallowMerge<Value>(stored: Value, defaults: Value): Value {
  if (!isPlainObject(stored) || !isPlainObject(defaults)) return stored;
  const merged: unknown = { ...defaults, ...stored };
  return sameKind(defaults, merged) ? merged : stored;
}

function browserWindow(): (Window & typeof globalThis) | undefined {
  return typeof window !== "undefined" ? window : undefined;
}

function defaultStorage(area: "localStorage" | "sessionStorage"): StorageLike | undefined {
  try {
    return browserWindow()?.[area] ?? undefined;
  } catch {
    // Accessing Web Storage throws when it is disabled (for example in
    // sandboxed iframes or with third-party storage blocked).
    return undefined;
  }
}

interface SameDocumentDetail {
  readonly storage: StorageLike;
  readonly key: string;
  readonly raw: string | null;
  readonly source: object;
}

function isSameDocumentDetail(detail: unknown): detail is SameDocumentDetail {
  return isPlainObject(detail) && typeof detail.key === "string" && "storage" in detail;
}

/**
 * Synchronize a typed reactive value with a Web Storage compatible backend.
 *
 * The serializer is inferred from `defaultValue` (numbers stay numbers,
 * `Map`/`Set`/`Date` round-trip), and every decoded value passes the
 * `validate` hook before it is accepted, so corrupted or foreign data falls
 * back to the default instead of leaking the wrong type. Writes follow the
 * reactive value; changes from other tabs (`storage` events) and from other
 * instances in the same document using the same key are applied live.
 *
 * Server rendering: without a browser `window` no storage is touched and
 * `state` holds the default. Node's global `localStorage` is intentionally
 * ignored because it would be shared between requests. Use
 * `initialRead: "post-flush"` to hydrate server markup that rendered the
 * default before switching to the stored value.
 *
 * Failures (quota, disabled storage, malformed data) never throw; they are
 * exposed through `error` and `onError`.
 *
 * @example
 * ```ts
 * const { state: theme } = useStorage("theme", "light" as "light" | "dark");
 * theme.value = "dark"; // persisted
 * ```
 *
 * @typeParam Value Stored value type, inferred from `defaultValue`.
 * @param key Reactive storage key; changing it re-reads the new key.
 * @param defaultValue Value used when the key is absent or unreadable.
 * @param options Backend, serialization, validation, and timing options.
 * @default options {}
 * @returns The synchronized value and its controls.
 */
export function useStorage<Value>(
  key: MaybeRefOrGetter<string>,
  defaultValue: MaybeRefOrGetter<Value>,
  options: UseStorageOptions<Value> = {},
): StorageControls<Value> {
  const initial = toValue(defaultValue);
  // Stored values are plain serializable data, for which `UnwrapRef<Value>`
  // and `Value` coincide; `ref` (not `shallowRef`) keeps nested mutations of
  // objects, maps, and sets observable so they are persisted too.
  const state = ref(initial) as Ref<Value>;
  const supported = ref(false);
  const error = shallowRef<StorageFailure | undefined>(undefined);
  const identity = {};
  const codec: StorageCodec | StorageSerializer<Value> =
    options.serializer ?? codecs[inferStorageSerializerKind(initial)];
  const validate: StorageValidator<Value> =
    options.validate ?? ((candidate): candidate is Value => sameKind(initial, candidate));
  const listen = options.listenToStorageChanges ?? true;
  let ready = false;
  // Encoded form of `state` that needs no write: the raw value most recently
  // read from or written to storage, or the default after a fallback/removal.
  let syncedRaw: string | undefined;

  const resolveStorage = (): StorageLike | undefined =>
    options.storage === undefined
      ? defaultStorage("localStorage")
      : (toValue(options.storage) ?? undefined);
  const resolveEvents = (): EventTarget | undefined =>
    options.eventTarget === undefined
      ? browserWindow()
      : (toValue(options.eventTarget) ?? undefined);

  const fail = (code: StorageErrorCode, cause: unknown): void => {
    const failure: StorageFailure = { code, key: toValue(key), cause };
    error.value = failure;
    options.onError?.(failure);
  };

  const encode = (value: Value): string | undefined => {
    try {
      return codec.write(value);
    } catch (cause) {
      fail("write-failed", cause);
      return undefined;
    }
  };

  const assign = (value: Value, raw: string | undefined): void => {
    syncedRaw = raw ?? encode(value);
    state.value = value;
  };

  const decode = (raw: string): { readonly value: Value; readonly accepted: boolean } => {
    const defaults = toValue(defaultValue);
    let candidate: unknown;
    try {
      candidate = codec.read(raw);
    } catch (cause) {
      fail("read-failed", cause);
      return { value: defaults, accepted: false };
    }
    if (!validate(candidate)) {
      fail("invalid-value", candidate);
      return { value: defaults, accepted: false };
    }
    const merge = options.mergeDefaults ?? false;
    if (merge === false) return { value: candidate, accepted: true };
    const value = merge === true ? shallowMerge(candidate, defaults) : merge(candidate, defaults);
    return { value, accepted: true };
  };

  const notify = (storage: StorageLike, raw: string | null): void => {
    const events = resolveEvents();
    if (!events || !listen) return;
    const detail: SameDocumentDetail = { storage, key: toValue(key), raw, source: identity };
    events.dispatchEvent(new CustomEvent(SAME_DOCUMENT_EVENT, { detail }));
  };

  const write = (storage: StorageLike, value: Value): void => {
    const raw = encode(value);
    if (raw === undefined) return;
    try {
      storage.setItem(toValue(key), raw);
    } catch (cause) {
      fail("write-failed", cause);
      return;
    }
    syncedRaw = raw;
    error.value = undefined;
    notify(storage, raw);
  };

  const applyRaw = (raw: string | null): void => {
    if (raw === null) {
      assign(toValue(defaultValue), undefined);
      return;
    }
    const decoded = decode(raw);
    // A rejected value keeps the stored data untouched until the next write.
    assign(decoded.value, decoded.accepted ? raw : undefined);
  };

  const refresh = (): void => {
    const storage = resolveStorage();
    supported.value = storage !== undefined;
    if (!storage) {
      assign(toValue(defaultValue), undefined);
      return;
    }
    let raw: string | null;
    try {
      raw = storage.getItem(toValue(key));
    } catch (cause) {
      fail("read-failed", cause);
      assign(toValue(defaultValue), undefined);
      return;
    }
    error.value = undefined;
    applyRaw(raw);
    if (raw === null && (options.writeDefaults ?? true)) write(storage, state.value);
  };

  const remove = (): void => {
    const storage = resolveStorage();
    if (storage) {
      try {
        storage.removeItem(toValue(key));
        notify(storage, null);
      } catch (cause) {
        fail("remove-failed", cause);
      }
    }
    assign(toValue(defaultValue), undefined);
  };

  const applyExternal = (storage: StorageLike, changedKey: string | null, raw: string | null) => {
    if (!ready || storage !== resolveStorage()) return;
    if (changedKey === null) applyRaw(null);
    else if (changedKey === toValue(key)) applyRaw(raw);
  };

  const onStorageEvent = (event: Event): void => {
    if (!("storageArea" in event) || !("key" in event) || !("newValue" in event)) return;
    const { storageArea, key: changedKey, newValue } = event;
    if (typeof changedKey !== "string" && changedKey !== null) return;
    if (typeof newValue !== "string" && newValue !== null) return;
    const storage = resolveStorage();
    if (storage === undefined || storageArea !== storage) return;
    applyExternal(storage, changedKey, newValue);
  };

  const onSameDocumentEvent = (event: Event): void => {
    if (!(event instanceof CustomEvent)) return;
    const detail: unknown = event.detail;
    if (!isSameDocumentDetail(detail) || detail.source === identity) return;
    applyExternal(detail.storage, detail.key, detail.raw);
  };

  const stopSource = watch(
    [() => toValue(key), resolveStorage],
    () => {
      if (ready) refresh();
    },
    { flush: "sync" },
  );

  const stopEvents = watch(
    resolveEvents,
    (events, _previous, onCleanup) => {
      if (!events || !listen) return;
      events.addEventListener("storage", onStorageEvent);
      events.addEventListener(SAME_DOCUMENT_EVENT, onSameDocumentEvent);
      onCleanup(() => {
        events.removeEventListener("storage", onStorageEvent);
        events.removeEventListener(SAME_DOCUMENT_EVENT, onSameDocumentEvent);
      });
    },
    { immediate: true, flush: "sync" },
  );

  const stopWrite = watch(
    state,
    (value) => {
      if (!ready) return;
      const storage = resolveStorage();
      if (!storage) return;
      const raw = encode(value);
      if (raw === undefined || raw === syncedRaw) return;
      write(storage, value);
    },
    { deep: options.deep ?? true, flush: options.flush ?? "pre" },
  );

  const start = (): void => {
    ready = true;
    refresh();
  };
  let stopDeferred: (() => void) | undefined;
  if ((options.initialRead ?? "sync") === "sync") {
    start();
  } else {
    // A post-flush job runs after the owning component mounted (so hydration
    // saw the default), or on the next microtask outside components.
    const trigger = ref(0);
    stopDeferred = watch(
      trigger,
      () => {
        stopDeferred?.();
        start();
      },
      { flush: "post" },
    );
    trigger.value += 1;
  }

  tryOnScopeDispose(() => {
    ready = false;
    stopDeferred?.();
    stopSource();
    stopEvents();
    stopWrite();
  });

  return {
    state,
    supported: readonly(supported),
    error: readonly(error),
    refresh,
    remove,
  };
}

/**
 * {@link useStorage} bound to `window.localStorage` by default.
 *
 * @typeParam Value Stored value type, inferred from `defaultValue`.
 * @param key Reactive storage key.
 * @param defaultValue Value used when the key is absent or unreadable.
 * @param options Serialization, validation, and timing options.
 * @default options {}
 * @returns The synchronized value and its controls.
 */
export function useLocalStorage<Value>(
  key: MaybeRefOrGetter<string>,
  defaultValue: MaybeRefOrGetter<Value>,
  options: UseStorageOptions<Value> = {},
): StorageControls<Value> {
  return useStorage(key, defaultValue, options);
}

/**
 * {@link useStorage} bound to `window.sessionStorage` by default.
 *
 * @typeParam Value Stored value type, inferred from `defaultValue`.
 * @param key Reactive storage key.
 * @param defaultValue Value used when the key is absent or unreadable.
 * @param options Serialization, validation, and timing options.
 * @default options {}
 * @returns The synchronized value and its controls.
 */
export function useSessionStorage<Value>(
  key: MaybeRefOrGetter<string>,
  defaultValue: MaybeRefOrGetter<Value>,
  options: UseStorageOptions<Value> = {},
): StorageControls<Value> {
  return useStorage(key, defaultValue, {
    ...options,
    storage:
      options.storage === undefined ? () => defaultStorage("sessionStorage") : options.storage,
  });
}
