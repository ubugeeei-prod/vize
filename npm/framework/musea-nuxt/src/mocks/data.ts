/**
 * Mock Nuxt data-fetching composables.
 */

import { ref } from "vue";

let _fetchMocks: Record<string, unknown> = {};

export function _setFetchMocks(mocks: Record<string, unknown>): void {
  _fetchMocks = mocks;
}

function findMockData(key: string): unknown {
  // Exact match first
  if (key in _fetchMocks) return _fetchMocks[key];
  // Pattern match
  for (const [pattern, data] of Object.entries(_fetchMocks)) {
    if (key.includes(pattern)) return data;
  }
  return undefined;
}

interface AsyncDataResult<T> {
  data: ReturnType<typeof ref<T | null>>;
  pending: ReturnType<typeof ref<boolean>>;
  error: ReturnType<typeof ref<Error | null>>;
  refresh: () => Promise<void>;
  execute: () => Promise<void>;
  status: ReturnType<typeof ref<string>>;
}

/**
 * Mock useFetch - returns reactive data based on mock config.
 */
export function useFetch<T = unknown>(
  url: string | (() => string),
  _opts?: Record<string, unknown>,
): AsyncDataResult<T> {
  const key = typeof url === "function" ? url() : url;
  const mockData = findMockData(key);

  const data = ref(mockData ?? null) as ReturnType<typeof ref<T | null>>;
  const pending = ref(false);
  const error = ref(null) as ReturnType<typeof ref<Error | null>>;
  const status = ref("success");

  const refresh = async () => {};
  const execute = async () => {};

  return { data, pending, error, refresh, execute, status };
}

/**
 * Mock useAsyncData - similar to useFetch but with key-based lookup.
 */
export function useAsyncData<T = unknown>(
  key: string,
  _handler?: () => Promise<T>,
  _opts?: Record<string, unknown>,
): AsyncDataResult<T> {
  const mockData = findMockData(key);

  const data = ref(mockData ?? null) as ReturnType<typeof ref<T | null>>;
  const pending = ref(false);
  const error = ref(null) as ReturnType<typeof ref<Error | null>>;
  const status = ref("success");

  const refresh = async () => {};
  const execute = async () => {};

  return { data, pending, error, refresh, execute, status };
}

/**
 * Mock useLazyFetch - lazy variant of useFetch.
 */
export function useLazyFetch<T = unknown>(
  url: string | (() => string),
  opts?: Record<string, unknown>,
): AsyncDataResult<T> {
  return useFetch<T>(url, { ...opts, lazy: true });
}

/**
 * Mock useLazyAsyncData - lazy variant of useAsyncData.
 */
export function useLazyAsyncData<T = unknown>(
  key: string,
  handler?: () => Promise<T>,
  opts?: Record<string, unknown>,
): AsyncDataResult<T> {
  return useAsyncData<T>(key, handler, { ...opts, lazy: true });
}
