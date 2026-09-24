import { readonly, ref, shallowReactive, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/**
 * Typed codec for one search parameter.
 *
 * `parse` receives every value of the key (`URLSearchParams.getAll`), which
 * is empty when the key is absent; throwing selects `default`. `serialize`
 * returns the values to write; an empty array removes the key.
 */
export interface SearchParamCodec<Value> {
  /** Value used when the key is absent or unparsable. */
  readonly default: Value;

  /** Decode all values of the key. */
  parse(values: readonly string[]): Value;

  /** Encode a value to zero or more raw values. */
  serialize(value: Value): readonly string[];
}

/** Schema mapping parameter names to codecs. */
export type SearchParamSchema = Readonly<Record<string, SearchParamCodec<unknown>>>;

/** Values described by a {@link SearchParamSchema}, inferred per key. */
export type InferSearchParams<Schema extends SearchParamSchema> = {
  -readonly [Key in keyof Schema]: Schema[Key] extends SearchParamCodec<infer Value>
    ? Value
    : never;
};

/** Single-value codec definition accepted by {@link searchParam}.custom. */
export interface CustomSearchParam<Value> {
  /** Decode the first raw value. Throwing selects `default`. */
  readonly parse: (raw: string) => Value;

  /** Encode a value to one raw value. */
  readonly serialize: (value: Value) => string;

  /** Value used when the key is absent or unparsable. */
  readonly default: Value;
}

function single<Value>(definition: CustomSearchParam<Value>): SearchParamCodec<Value> {
  return {
    default: definition.default,
    parse(values) {
      const [first] = values;
      return first === undefined ? definition.default : definition.parse(first);
    },
    serialize(value) {
      return [definition.serialize(value)];
    },
  };
}

function sameJsonKind<Value>(defaults: Value, candidate: unknown): candidate is Value {
  if (defaults === null) return true;
  if (Array.isArray(defaults)) return Array.isArray(candidate);
  if (typeof defaults === "object") {
    return typeof candidate === "object" && candidate !== null && !Array.isArray(candidate);
  }
  return typeof candidate === typeof defaults;
}

/**
 * Repeated-key codec of raw strings.
 *
 * @returns A string array codec.
 */
function arrayCodec(): SearchParamCodec<string[]>;
function arrayCodec<Item>(item: SearchParamCodec<Item>): SearchParamCodec<Item[]>;
function arrayCodec<Item>(
  item?: SearchParamCodec<Item>,
): SearchParamCodec<Item[]> | SearchParamCodec<string[]> {
  if (item === undefined) {
    const strings: SearchParamCodec<string[]> = {
      default: [],
      parse: (values) => [...values],
      serialize: (value) => value,
    };
    return strings;
  }
  const items: SearchParamCodec<Item[]> = {
    default: [],
    parse: (values) => values.map((value) => item.parse([value])),
    serialize: (value) => value.flatMap((entry) => item.serialize(entry)),
  };
  return items;
}

/** Built-in typed codecs for {@link useUrlSearchParams} schemas. */
export const searchParam = {
  /**
   * Raw string value.
   *
   * @param fallback Value for an absent key.
   * @default fallback ""
   * @returns A string codec.
   */
  string(fallback = ""): SearchParamCodec<string> {
    return single({ parse: (raw) => raw, serialize: (value) => value, default: fallback });
  },

  /**
   * Finite number; non-numeric input selects the default.
   *
   * @param fallback Value for an absent or invalid key.
   * @default fallback 0
   * @returns A number codec.
   */
  number(fallback = 0): SearchParamCodec<number> {
    return single({
      parse: (raw) => {
        const value = raw.trim() === "" ? Number.NaN : Number(raw);
        return Number.isFinite(value) ? value : fallback;
      },
      serialize: String,
      default: fallback,
    });
  },

  /**
   * Boolean: `true`/`1`/empty read as `true`, `false`/`0` as `false`.
   *
   * @param fallback Value for an absent or unrecognized key.
   * @default fallback false
   * @returns A boolean codec.
   */
  boolean(fallback = false): SearchParamCodec<boolean> {
    return single({
      parse: (raw) => {
        if (raw === "" || raw === "true" || raw === "1") return true;
        if (raw === "false" || raw === "0") return false;
        return fallback;
      },
      serialize: String,
      default: fallback,
    });
  },

  /**
   * One of a closed set of string literals.
   *
   * @param values Allowed values.
   * @param fallback Value for an absent or unknown key.
   * @returns A literal-union codec.
   */
  enum<const Values extends readonly [string, ...string[]]>(
    values: Values,
    fallback: NoInfer<Values[number]>,
  ): SearchParamCodec<Values[number]> {
    const allowed = new Set<string>(values);
    const isAllowed = (raw: string): raw is Values[number] => allowed.has(raw);
    return single({
      parse: (raw) => (isAllowed(raw) ? raw : fallback),
      serialize: (value) => value,
      default: fallback,
    });
  },

  /**
   * Repeated key (`?tag=a&tag=b`) decoded item by item.
   *
   * @param item Single-value codec applied to every value.
   * @default item searchParam.string()
   * @returns An array codec whose default is `[]`.
   */
  array: arrayCodec,

  /**
   * JSON value checked by `validate`, or by kind against the default.
   *
   * @param fallback Value for an absent or invalid key.
   * @param validate Type guard applied to the decoded JSON.
   * @default validate the decoded value has the default's kind
   * @returns A JSON codec.
   */
  json<Value>(
    fallback: Value,
    validate?: (candidate: unknown) => candidate is Value,
  ): SearchParamCodec<Value> {
    const accept =
      validate ?? ((candidate): candidate is Value => sameJsonKind(fallback, candidate));
    return single({
      parse: (raw) => {
        const candidate: unknown = JSON.parse(raw);
        return accept(candidate) ? candidate : fallback;
      },
      serialize: (value) => JSON.stringify(value),
      default: fallback,
    });
  },

  /**
   * Single-value codec from explicit parse/serialize functions.
   *
   * @param definition Parse, serialize, and default.
   * @returns The codec.
   */
  custom<Value>(definition: CustomSearchParam<Value>): SearchParamCodec<Value> {
    return single(definition);
  },
} as const;

function safeParse<Value>(codec: SearchParamCodec<Value>, values: readonly string[]): Value {
  try {
    return codec.parse(values);
  } catch {
    return codec.default;
  }
}

function toSearch(source: string | URL | URLSearchParams): URLSearchParams {
  if (source instanceof URLSearchParams) return new URLSearchParams(source);
  if (source instanceof URL) return new URLSearchParams(source.search);
  const withoutHash = source.split("#", 1)[0] ?? "";
  const query = withoutHash.indexOf("?");
  return new URLSearchParams(query === -1 ? "" : withoutHash.slice(query + 1));
}

/**
 * Decode search parameters with a schema. Pure and SSR-safe.
 *
 * @typeParam Schema Parameter codecs.
 * @param schema Parameter codecs.
 * @param search URL, URL string, query string, or parsed parameters.
 * @returns One typed value per schema key.
 */
export function parseSearchParams<Schema extends SearchParamSchema>(
  schema: Schema,
  search: string | URL | URLSearchParams,
): InferSearchParams<Schema> {
  const params = toSearch(search);
  const entries = Object.keys(schema).map((key) => {
    const codec = schema[key];
    return [key, codec === undefined ? undefined : safeParse(codec, params.getAll(key))] as const;
  });
  // `Object.fromEntries` loses the per-key correlation that the mapped
  // `InferSearchParams` type restores; every key was produced by its codec.
  return Object.fromEntries(entries) as InferSearchParams<Schema>;
}

/** Options for {@link serializeSearchParams}. */
export interface SerializeSearchParamsOptions {
  /**
   * Existing parameters to keep for keys outside the schema.
   *
   * @default no base parameters
   */
  readonly base?: string | URL | URLSearchParams;

  /**
   * Omit keys whose value serializes like the codec default.
   *
   * @default true
   */
  readonly removeDefaults?: boolean;
}

function sameValues(left: readonly string[], right: readonly string[]): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

/**
 * Encode typed values with a schema. Pure and SSR-safe.
 *
 * @typeParam Schema Parameter codecs.
 * @param schema Parameter codecs.
 * @param values Values to encode; keys absent from `values` keep the base.
 * @param options Base parameters and default removal.
 * @default options {}
 * @returns The encoded parameters.
 */
export function serializeSearchParams<Schema extends SearchParamSchema>(
  schema: Schema,
  values: Partial<InferSearchParams<Schema>>,
  options: SerializeSearchParamsOptions = {},
): URLSearchParams {
  const params = options.base === undefined ? new URLSearchParams() : toSearch(options.base);
  const removeDefaults = options.removeDefaults ?? true;
  for (const key of Object.keys(schema)) {
    const codec = schema[key];
    if (codec === undefined || !(key in values)) continue;
    const encoded = codec.serialize(values[key]);
    const [first, ...rest] = encoded;
    if (
      first === undefined ||
      (removeDefaults && sameValues(encoded, codec.serialize(codec.default)))
    ) {
      params.delete(key);
      continue;
    }
    // `set` keeps the key's existing position, so URLs stay stable.
    params.set(key, first);
    for (const value of rest) params.append(key, value);
  }
  return params;
}

/** Minimal `window` surface used by {@link useUrlSearchParams}. */
export interface UrlSearchParamsHost extends Pick<
  EventTarget,
  "addEventListener" | "removeEventListener"
> {
  /** Current location; only the URL parts are read. */
  readonly location: {
    /** Query including the leading `?`, or `""`. */
    readonly search: string;
    /** Fragment including the leading `#`, or `""`. */
    readonly hash: string;
    /** Path component. */
    readonly pathname: string;
  };

  /** Session history used to write the query. */
  readonly history: {
    /** Current history entry state, preserved when replacing. */
    readonly state: unknown;
    /** Replace the current entry's URL. */
    readonly replaceState: (data: unknown, unused: string, url: string) => void;
    /** Push a new entry with the given URL. */
    readonly pushState: (data: unknown, unused: string, url: string) => void;
  };
}

/** Options for {@link useUrlSearchParams}. */
export interface UseUrlSearchParamsOptions {
  /**
   * Browser host. `null`/`undefined` keeps server values.
   *
   * @default window when a browser window exists
   */
  readonly host?: MaybeRefOrGetter<UrlSearchParamsHost | null | undefined>;

  /**
   * History write mode for changes made through `params`.
   *
   * @default "replace"
   */
  readonly mode?: "replace" | "push";

  /**
   * Omit parameters equal to their codec default from the URL.
   *
   * @default true
   */
  readonly removeDefaults?: boolean;

  /**
   * Request URL used while no host is attached, so server markup matches
   * the client's first render.
   *
   * @default undefined (codec defaults are used)
   */
  readonly ssrUrl?: MaybeRefOrGetter<string | URL | URLSearchParams | undefined>;
}

/** Reactive state and controls returned by {@link useUrlSearchParams}. */
export interface UrlSearchParamsControls<Schema extends SearchParamSchema> {
  /**
   * Writable typed parameters. Assign whole values (including new arrays);
   * the object is shallowly reactive.
   */
  readonly params: InferSearchParams<Schema>;

  /** Whether a browser host is attached. */
  readonly supported: Readonly<Ref<boolean>>;

  /** Re-read the current location. */
  readonly refresh: () => void;

  /** Stop following the URL. Idempotent; also runs on scope disposal. */
  readonly stop: () => void;
}

function browserHost(): UrlSearchParamsHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}

/**
 * Two-way bind typed URL search parameters described by a codec schema.
 *
 * Every property type is inferred from its codec. Assigning a property
 * rewrites the query through `history.replaceState` (or `pushState` with
 * `mode: "push"`), keeping parameters outside the schema and the fragment,
 * and dropping values equal to their default. Back/forward navigation
 * (`popstate`) re-reads the URL.
 *
 * Server rendering: no global is read. Pass `ssrUrl` (the request URL) so
 * the server renders the same values the client reads during hydration.
 * Listeners are removed with the owning reactive scope or `stop()`.
 *
 * @example
 * ```ts
 * const { params } = useUrlSearchParams({
 *   page: searchParam.number(1),
 *   sort: searchParam.enum(["new", "top"], "new"),
 *   tags: searchParam.array(),
 * });
 * params.page += 1; // ?page=2
 * ```
 *
 * @typeParam Schema Parameter codecs.
 * @param schema Parameter codecs.
 * @param options Host, history mode, default removal, and server URL.
 * @default options {}
 * @returns Typed reactive parameters and controls.
 */
export function useUrlSearchParams<const Schema extends SearchParamSchema>(
  schema: Schema,
  options: UseUrlSearchParamsOptions = {},
): UrlSearchParamsControls<Schema> {
  const resolveHost = (): UrlSearchParamsHost | undefined =>
    options.host === undefined ? browserHost() : (toValue(options.host) ?? undefined);
  const removeDefaults = options.removeDefaults ?? true;
  const ssrUrl = toValue(options.ssrUrl);
  const params = shallowReactive(parseSearchParams(schema, ssrUrl ?? ""));
  const supported = ref(false);
  let syncedSearch: string | undefined;
  let applying = false;

  const encode = (host: UrlSearchParamsHost): string =>
    serializeSearchParams(schema, params, {
      base: host.location.search,
      removeDefaults,
    }).toString();

  const refresh = (): void => {
    const host = resolveHost();
    supported.value = host !== undefined;
    const source = host === undefined ? (toValue(options.ssrUrl) ?? "") : host.location.search;
    applying = true;
    try {
      Object.assign(params, parseSearchParams(schema, source));
    } finally {
      applying = false;
    }
    if (host !== undefined) syncedSearch = encode(host);
  };

  const stopHost = watch(
    resolveHost,
    (host, _previous, onCleanup) => {
      refresh();
      if (host === undefined) return;
      host.addEventListener("popstate", refresh);
      onCleanup(() => host.removeEventListener("popstate", refresh));
    },
    { immediate: true, flush: "sync" },
  );

  const stopWrite = watch(
    () => {
      const host = resolveHost();
      return host === undefined ? undefined : encode(host);
    },
    (search) => {
      const host = resolveHost();
      if (applying || search === undefined || host === undefined) return;
      if (search === syncedSearch) return;
      syncedSearch = search;
      const { pathname, hash } = host.location;
      const url = `${pathname}${search === "" ? "" : `?${search}`}${hash}`;
      if ((options.mode ?? "replace") === "push") host.history.pushState(null, "", url);
      else host.history.replaceState(host.history.state, "", url);
    },
    { flush: "sync" },
  );

  const stop = (): void => {
    stopHost();
    stopWrite();
  };
  tryOnScopeDispose(stop);

  return { params, supported: readonly(supported), refresh, stop };
}
