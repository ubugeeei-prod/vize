import { shallowReactive } from "vue";

/** Minimal cache contract used by {@link useMemoize}; a `Map` satisfies it. */
export interface MemoizeCache<Key, Result> {
  /** Read a cached result. */
  get(key: Key): Result | undefined;
  /** Store a result. */
  set(key: Key, value: Result): unknown;
  /** Whether a result is cached for `key`. */
  has(key: Key): boolean;
  /** Drop one result. */
  delete(key: Key): unknown;
  /** Drop every result. */
  clear(): void;
}

/** Options for {@link useMemoize}. */
export interface UseMemoizeOptions<Arguments extends readonly unknown[], Key, Result> {
  /**
   * Derive the cache key from the arguments.
   *
   * @default JSON.stringify of the argument list
   */
  readonly getKey?: (...args: Arguments) => Key;

  /**
   * Storage for results; supply an LRU or a persistent cache here.
   *
   * @default a reactive Map
   */
  readonly cache?: MemoizeCache<Key, Result>;
}

/** Memoized function returned by {@link useMemoize}. */
export interface Memoized<Arguments extends readonly unknown[], Result, Key> {
  /** Return the cached result for these arguments, computing it on a miss. */
  (...args: Arguments): Result;

  /** Recompute and re-cache the result for these arguments. */
  readonly load: (...args: Arguments) => Result;

  /** Drop the cached result for these arguments. */
  readonly delete: (...args: Arguments) => void;

  /** Drop every cached result. */
  readonly clear: () => void;

  /** Cache key for these arguments. */
  readonly generateKey: (...args: Arguments) => Key;

  /** The underlying cache. */
  readonly cache: MemoizeCache<Key, Result>;
}

/**
 * Cache a function's results per argument list.
 *
 * The default cache is a reactive `Map`, so computeds and templates that
 * read a memoized value re-evaluate after `load`, `delete`, or `clear`.
 * Promise-returning resolvers cache the promise itself, which deduplicates
 * concurrent requests. Keys default to `JSON.stringify(args)` (a `string`);
 * a custom `getKey` sets the key type, and a custom `cache` must use that
 * key type. On the server keep memoized caches request-local (create them in
 * `setup`), since a module-level cache is shared across requests.
 *
 * @example
 * ```ts
 * const getUser = useMemoize((id: string) => fetchUser(id), { getKey: (id) => id });
 * const user = computed(() => getUser(props.id));
 * getUser.load(props.id); // refresh
 * ```
 *
 * @param resolver Function whose results are cached.
 * @param options Key derivation and cache storage.
 * @default options {}
 * @returns The memoized function with cache controls.
 */
export function useMemoize<Arguments extends readonly unknown[], Result, Key>(
  resolver: (...args: Arguments) => Result,
  options: UseMemoizeOptions<Arguments, Key, Result> & {
    readonly getKey: (...args: Arguments) => Key;
  },
): Memoized<Arguments, Result, Key>;
export function useMemoize<Arguments extends readonly unknown[], Result>(
  resolver: (...args: Arguments) => Result,
  options?: UseMemoizeOptions<Arguments, string, Result>,
): Memoized<Arguments, Result, string>;
export function useMemoize(
  resolver: (...args: readonly unknown[]) => unknown,
  options: UseMemoizeOptions<readonly unknown[], unknown, unknown> = {},
): Memoized<readonly unknown[], unknown, unknown> {
  const cache = options.cache ?? shallowReactive(new Map<unknown, unknown>());
  const generateKey = (...args: readonly unknown[]): unknown =>
    options.getKey === undefined ? JSON.stringify(args) : options.getKey(...args);

  const load = (...args: readonly unknown[]): unknown => {
    const result = resolver(...args);
    cache.set(generateKey(...args), result);
    return result;
  };

  const memoized = (...args: readonly unknown[]): unknown => {
    const key = generateKey(...args);
    return cache.has(key) ? cache.get(key) : load(...args);
  };

  return Object.assign(memoized, {
    load,
    delete: (...args: readonly unknown[]) => {
      cache.delete(generateKey(...args));
    },
    clear: () => {
      cache.clear();
    },
    generateKey,
    cache,
  });
}
