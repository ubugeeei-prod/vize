import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, WritableComputedRef } from "vue";

/** Raw query value: repeated keys become arrays, absent keys are `undefined`. */
export type RawQueryValue = string | readonly string[] | undefined;

/** Query record as produced by `@vizejs/router` (`RouteMatch["query"]`). */
export type RouteQueryRecord = Readonly<Record<string, string | readonly string[]>>;

/**
 * Structural subset of a matched route. `@vizejs/router`'s `RouteMatch`
 * satisfies it, so its inferred `params` type flows through without this
 * package depending on the router.
 */
export interface RouteLocationLike {
  /** Decoded path params. */
  readonly params: object;
  /** Parsed query. */
  readonly query: RouteQueryRecord;
}

/** Converts a raw query value; `undefined` selects the default. */
export type QueryParser<Value> = (raw: RawQueryValue) => Value | undefined;

/** Converts a value back to a raw query value; `undefined` removes the key. */
export type QuerySerializer<Value> = (value: Value) => string | readonly string[] | undefined;

/** Built-in {@link QueryParser}s. */
export const queryParsers = {
  /** First value as a string. */
  string: (raw: RawQueryValue): string | undefined => (typeof raw === "string" ? raw : raw?.[0]),
  /** First value as a finite number. */
  number: (raw: RawQueryValue): number | undefined => {
    const text = typeof raw === "string" ? raw : raw?.[0];
    if (text === undefined || text.trim() === "") return undefined;
    const value = Number(text);
    return Number.isFinite(value) ? value : undefined;
  },
  /** First value as a safe integer. */
  integer: (raw: RawQueryValue): number | undefined => {
    const text = typeof raw === "string" ? raw : raw?.[0];
    return text !== undefined && /^-?\d+$/.test(text) && Number.isSafeInteger(Number(text))
      ? Number(text)
      : undefined;
  },
  /** `"true"`/`"1"`/`""` (present flag) → `true`, `"false"`/`"0"` → `false`. */
  boolean: (raw: RawQueryValue): boolean | undefined => {
    const text = typeof raw === "string" ? raw : raw?.[0];
    if (text === undefined) return undefined;
    if (text === "true" || text === "1" || text === "") return true;
    if (text === "false" || text === "0") return false;
    return undefined;
  },
  /** Every value as a string array. */
  array: (raw: RawQueryValue): string[] | undefined =>
    raw === undefined ? undefined : typeof raw === "string" ? [raw] : [...raw],
} as const;

/**
 * Parser accepting only the given literal values.
 *
 * @example
 * ```ts
 * useRouteQuery(route, "sort", { parse: oneOf(["new", "top"]), default: "new" });
 * ```
 *
 * @param values Allowed values.
 * @returns A parser narrowing to the literal union.
 */
export function oneOf<const Values extends readonly string[]>(
  values: Values,
): QueryParser<Values[number]> {
  const allowed = new Set<string>(values);
  const isAllowed = (text: string): text is Values[number] => allowed.has(text);
  return (raw) => {
    const text = typeof raw === "string" ? raw : raw?.[0];
    return text !== undefined && isAllowed(text) ? text : undefined;
  };
}

/** Options for {@link useRouteQuery}. */
export interface UseRouteQueryOptions<Value> {
  /**
   * Converts the raw value.
   *
   * @default queryParsers.string
   */
  readonly parse?: QueryParser<Value>;

  /**
   * Converts a written value back into the query.
   *
   * @default strings and string arrays unchanged, other values via `String`
   */
  readonly serialize?: QuerySerializer<Value>;

  /**
   * Performs the navigation for a write, receiving the complete next query.
   * Without it the returned ref is read-only.
   *
   * @default undefined
   */
  readonly navigate?: (query: RouteQueryRecord) => void;

  /**
   * Remove the key instead of writing it when the value equals the default.
   *
   * @default true
   */
  readonly omitDefault?: boolean;
}

/**
 * Implementation view of every overload's options. Method syntax keeps the
 * serializer parameter bivariant so each typed overload is compatible.
 */
interface ErasedQueryOptions {
  readonly parse?: QueryParser<unknown> | undefined;
  serialize?(value: unknown): string | readonly string[] | undefined;
  readonly navigate?: (query: RouteQueryRecord) => void;
  readonly omitDefault?: boolean;
  readonly default?: unknown;
}

/** Options of a string query parameter (no parser). */
export type StringQueryOptions = Omit<UseRouteQueryOptions<string>, "parse"> & {
  /** Must be omitted: string parameters use {@link queryParsers.string}. */
  readonly parse?: undefined;
};

function defaultSerialize(value: unknown): string | readonly string[] | undefined {
  if (value === undefined || value === null) return undefined;
  if (typeof value === "string") return value;
  if (Array.isArray(value))
    return value.map((item) => (typeof item === "string" ? item : JSON.stringify(item)));
  return typeof value === "number" || typeof value === "boolean" || typeof value === "bigint"
    ? value.toString()
    : JSON.stringify(value);
}

function sameRaw(left: RawQueryValue, right: RawQueryValue): boolean {
  if (typeof left === "string" || typeof right === "string" || !left || !right) {
    return left === right;
  }
  return left.length === right.length && left.every((item, index) => item === right[index]);
}

/**
 * Typed, optionally writable view of one query parameter.
 *
 * The value type is inferred from `parse` and `default`: with a default the
 * ref never holds `undefined`. Pass `navigate` to make the ref writable;
 * writes build the next query (keeping every other key), drop the key when
 * the value equals the default, and hand it to `navigate`, which is where
 * the router integration pushes or replaces the URL. Works with any route
 * object exposing `query`, including `@vizejs/router`'s `RouteMatch`.
 * Pure derived state: SSR-safe and hydration-stable (it reads the matched
 * route only).
 *
 * @example
 * ```ts
 * const page = useRouteQuery(match, "page", {
 *   parse: queryParsers.integer,
 *   default: 1,
 *   navigate: (query) => navigateTo(router.resolve(match.value.name, match.value.params, { query })),
 * });
 * page.value += 1;
 * ```
 *
 * @param route Reactive matched route.
 * @param key Query key.
 * @param options Parser, default, serializer, and navigation.
 * @default options {}
 * @returns A computed ref (writable when `navigate` is given).
 */
export function useRouteQuery(
  route: MaybeRefOrGetter<RouteLocationLike>,
  key: string,
  options: StringQueryOptions & {
    readonly default: string;
    readonly navigate: (query: RouteQueryRecord) => void;
  },
): WritableComputedRef<string>;
export function useRouteQuery(
  route: MaybeRefOrGetter<RouteLocationLike>,
  key: string,
  options: StringQueryOptions & { readonly navigate: (query: RouteQueryRecord) => void },
): WritableComputedRef<string | undefined, string | undefined>;
export function useRouteQuery(
  route: MaybeRefOrGetter<RouteLocationLike>,
  key: string,
  options: StringQueryOptions & { readonly default: string },
): ComputedRef<string>;
export function useRouteQuery(
  route: MaybeRefOrGetter<RouteLocationLike>,
  key: string,
  options?: StringQueryOptions,
): ComputedRef<string | undefined>;
export function useRouteQuery<Value>(
  route: MaybeRefOrGetter<RouteLocationLike>,
  key: string,
  options: UseRouteQueryOptions<Value> & {
    readonly default: NoInfer<Value>;
    readonly navigate: (query: RouteQueryRecord) => void;
  },
): WritableComputedRef<Value>;
export function useRouteQuery<Value>(
  route: MaybeRefOrGetter<RouteLocationLike>,
  key: string,
  options: UseRouteQueryOptions<Value> & { readonly navigate: (query: RouteQueryRecord) => void },
): WritableComputedRef<Value | undefined, Value | undefined>;
export function useRouteQuery<Value>(
  route: MaybeRefOrGetter<RouteLocationLike>,
  key: string,
  options: UseRouteQueryOptions<Value> & { readonly default: NoInfer<Value> },
): ComputedRef<Value>;
export function useRouteQuery<Value>(
  route: MaybeRefOrGetter<RouteLocationLike>,
  key: string,
  options: UseRouteQueryOptions<Value>,
): ComputedRef<Value | undefined>;
export function useRouteQuery(
  route: MaybeRefOrGetter<RouteLocationLike>,
  key: string,
  options: ErasedQueryOptions = {},
): ComputedRef<unknown> | WritableComputedRef<unknown> {
  const parse = options.parse ?? queryParsers.string;
  const read = (): unknown => parse(toValue(route).query[key]) ?? options.default;
  const navigate = options.navigate;
  if (navigate === undefined) return computed(read);

  const serialize = (value: unknown): string | readonly string[] | undefined =>
    options.serialize === undefined ? defaultSerialize(value) : options.serialize(value);
  return computed({
    get: read,
    set: (value) => {
      const current = toValue(route).query;
      const raw =
        (options.omitDefault ?? true) &&
        "default" in options &&
        sameRaw(serialize(value), serialize(options.default))
          ? undefined
          : serialize(value);
      if (sameRaw(raw, current[key])) return;
      const next: Record<string, string | readonly string[]> = { ...current };
      if (raw === undefined) Reflect.deleteProperty(next, key);
      else next[key] = raw;
      navigate(next);
    },
  });
}

/** Params object of a route location. */
export type RouteParamsOf<Route extends RouteLocationLike> = Route["params"];

/**
 * Typed view of the matched route's path params.
 *
 * Without a key, returns all params (typed from the route table when the
 * route comes from `@vizejs/router`). With a key, returns that param; with
 * a parser, the parsed value. Pure derived state: SSR-safe.
 *
 * @example
 * ```ts
 * const params = useRouteParams(match); // ComputedRef<{ readonly id: string }>
 * const id = useRouteParams(match, "id", Number); // ComputedRef<number>
 * ```
 *
 * @param route Reactive matched route.
 * @param key Param name.
 * @param parse Converts the raw param.
 * @returns Computed params, param, or parsed param.
 */
export function useRouteParams<Route extends RouteLocationLike>(
  route: MaybeRefOrGetter<Route>,
): ComputedRef<RouteParamsOf<Route>>;
export function useRouteParams<Route extends RouteLocationLike, Key extends keyof Route["params"]>(
  route: MaybeRefOrGetter<Route>,
  key: Key,
): ComputedRef<Route["params"][Key]>;
export function useRouteParams<
  Route extends RouteLocationLike,
  Key extends keyof Route["params"],
  Value,
>(
  route: MaybeRefOrGetter<Route>,
  key: Key,
  parse: (raw: Route["params"][Key]) => Value,
): ComputedRef<Value>;
export function useRouteParams(
  route: MaybeRefOrGetter<RouteLocationLike>,
  ...args: readonly [] | readonly [PropertyKey] | readonly [PropertyKey, (raw: unknown) => unknown]
): ComputedRef<unknown> {
  return computed(() => {
    const { params } = toValue(route);
    if (args.length === 0) return params;
    const raw: unknown = Reflect.get(params, args[0]);
    return args.length === 2 ? args[1](raw) : raw;
  });
}
