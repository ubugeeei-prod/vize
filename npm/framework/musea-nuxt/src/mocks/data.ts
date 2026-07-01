/**
 * Mock Nuxt data-fetching composables.
 */

import { ref, toValue, type MaybeRefOrGetter, type Ref } from "vue";

import { getFetchMocks, setFetchMocks } from "../context.js";

export { setFetchMocks as _setFetchMocks };

export function findMockData(key: string): unknown {
  const fetchMocks = getFetchMocks();
  // Exact match first
  if (key in fetchMocks) return fetchMocks[key];
  // Pattern match
  for (const [pattern, data] of Object.entries(fetchMocks)) {
    if (key.includes(pattern)) return data;
  }
  return undefined;
}

interface AsyncDataOptions<T> {
  default?: () => T;
  lazy?: boolean;
  immediate?: boolean;
  transform?: (input: unknown) => T;
  pick?: string[];
}

interface AsyncDataResult<T> {
  data: Ref<T | null>;
  pending: Ref<boolean>;
  error: Ref<Error | null>;
  refresh: () => Promise<void>;
  execute: () => Promise<void>;
  clear: () => void;
  status: Ref<"idle" | "pending" | "success" | "error">;
}

/**
 * Mock useFetch - returns reactive data based on mock config.
 */
export function useFetch<T = unknown>(
  url: MaybeRefOrGetter<string | URL>,
  opts?: AsyncDataOptions<T>,
): AsyncDataResult<T> {
  const key = stringifyDataKey(toValue(url));
  return createAsyncDataResult<T>(key, undefined, opts);
}

/**
 * Mock useAsyncData - similar to useFetch but with key-based lookup.
 */
export function useAsyncData<T = unknown>(
  key: string,
  handler?: () => T | Promise<T>,
  opts?: AsyncDataOptions<T>,
): AsyncDataResult<T> {
  return createAsyncDataResult<T>(key, handler, opts);
}

/**
 * Mock useLazyFetch - lazy variant of useFetch.
 */
export function useLazyFetch<T = unknown>(
  url: MaybeRefOrGetter<string | URL>,
  opts?: AsyncDataOptions<T>,
): AsyncDataResult<T> {
  return useFetch<T>(url, { ...opts, lazy: true });
}

/**
 * Mock useLazyAsyncData - lazy variant of useAsyncData.
 */
export function useLazyAsyncData<T = unknown>(
  key: string,
  handler?: () => T | Promise<T>,
  opts?: AsyncDataOptions<T>,
): AsyncDataResult<T> {
  return useAsyncData<T>(key, handler, { ...opts, lazy: true });
}

export function refreshNuxtData(_keys?: string | string[]): Promise<void> {
  return Promise.resolve();
}

export function clearNuxtData(_keys?: string | string[]): void {
  // no-op: each mock result owns its local refs.
}

export function useNuxtData<T = unknown>(key: string): { data: Ref<T | null> } {
  const mockData = findMockData(key);
  return {
    data: ref((mockData ?? null) as T | null),
  };
}

export function useRequestFetch() {
  return async <T = unknown>(url: string | URL): Promise<T> => {
    const mockData = findMockData(stringifyDataKey(url));
    return mockData as T;
  };
}

function createAsyncDataResult<T>(
  key: string,
  handler: (() => T | Promise<T>) | undefined,
  opts: AsyncDataOptions<T> = {},
): AsyncDataResult<T> {
  const initial = resolveInitialData<T>(key, opts);
  const data = ref(initial) as Ref<T | null>;
  const pending = ref(false);
  const error = ref<Error | null>(null);
  const status = ref<"idle" | "pending" | "success" | "error">(
    initial == null ? "idle" : "success",
  );

  const execute = async () => {
    pending.value = true;
    status.value = "pending";
    error.value = null;
    try {
      const mockData = findMockData(key);
      const value = mockData !== undefined ? mockData : handler ? await handler() : data.value;
      data.value = applyDataOptions(value, opts);
      status.value = "success";
    } catch (caught) {
      error.value = caught instanceof Error ? caught : new Error(String(caught));
      status.value = "error";
    } finally {
      pending.value = false;
    }
  };

  if (opts.immediate !== false && opts.lazy !== true && initial == null && handler) {
    void execute();
  }

  return {
    data,
    pending,
    error,
    refresh: execute,
    execute,
    clear: () => {
      data.value = null;
      error.value = null;
      pending.value = false;
      status.value = "idle";
    },
    status,
  };
}

function resolveInitialData<T>(key: string, opts: AsyncDataOptions<T>): T | null {
  const mockData = findMockData(key);
  if (mockData !== undefined) {
    return applyDataOptions(mockData, opts);
  }
  return opts.default ? opts.default() : null;
}

function applyDataOptions<T>(value: unknown, opts: AsyncDataOptions<T>): T {
  const transformed = opts.transform ? opts.transform(value) : value;
  if (!opts.pick || typeof transformed !== "object" || transformed == null) {
    return transformed as T;
  }

  const picked: Record<string, unknown> = {};
  for (const key of opts.pick) {
    picked[key] = (transformed as Record<string, unknown>)[key];
  }
  return picked as T;
}

function stringifyDataKey(value: string | URL): string {
  return typeof value === "string" ? value : value.toString();
}
