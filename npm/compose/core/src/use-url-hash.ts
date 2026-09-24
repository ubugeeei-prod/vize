import { readonly, ref, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Minimal `window` surface used by {@link useUrlHash}. */
export interface UrlHashHost extends Pick<EventTarget, "addEventListener" | "removeEventListener"> {
  /** Current location; only the URL parts are read. */
  readonly location: {
    /** Fragment including the leading `#`, or `""`. */
    readonly hash: string;
    /** Path component. */
    readonly pathname: string;
    /** Query component including the leading `?`, or `""`. */
    readonly search: string;
  };

  /** Session history used to write the fragment without scrolling. */
  readonly history: {
    /** Current history entry state, preserved when replacing. */
    readonly state: unknown;
    /** Replace the current entry's URL. */
    readonly replaceState: (data: unknown, unused: string, url: string) => void;
    /** Push a new entry with the given URL. */
    readonly pushState: (data: unknown, unused: string, url: string) => void;
  };
}

/** How a changed hash is written to session history. */
export type UrlHistoryMode = "replace" | "push";

/** Options shared by both {@link useUrlHash} overloads. */
export interface UseUrlHashOptions {
  /**
   * Browser host. `null`/`undefined` keeps the default (server rendering).
   *
   * @default window when a browser window exists
   */
  readonly host?: MaybeRefOrGetter<UrlHashHost | null | undefined>;

  /**
   * History write mode for changes made through `state`.
   *
   * @default "replace"
   */
  readonly mode?: UrlHistoryMode;

  /**
   * Raw hash (without `#`) to use while no host is attached. Browsers never
   * send the fragment to servers, so pass it only when it is known (for
   * example from a client-side redirect).
   *
   * @default undefined (the default value is used)
   */
  readonly ssrHash?: MaybeRefOrGetter<string | undefined>;

  /**
   * When the browser hash is first read. `"post-flush"` keeps the server
   * value through hydration and reads the real hash after mounting.
   *
   * @default "sync"
   */
  readonly initialRead?: "sync" | "post-flush";
}

/** Options for a typed {@link useUrlHash} with an explicit codec. */
export interface UseTypedUrlHashOptions<Value> extends UseUrlHashOptions {
  /** Decode the raw (percent-decoded, `#`-less) hash. Throwing selects `default`. */
  readonly parse: (raw: string) => Value;

  /** Encode a value to a raw hash. An empty string removes the fragment. */
  readonly serialize: (value: Value) => string;

  /** Value used for an empty or unparsable hash. */
  readonly default: Value;
}

/** Reactive state and controls returned by {@link useUrlHash}. */
export interface UrlHashControls<Value> {
  /** Writable hash value; assignments update the URL. */
  readonly state: Ref<Value>;

  /** Whether a browser host is attached. */
  readonly supported: Readonly<Ref<boolean>>;

  /** Most recent parse failure, cleared by the next successful read. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /** Re-read the current location hash. */
  readonly refresh: () => void;
}

function browserHost(): UrlHashHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}

function decodeFragment(hash: string): string {
  const raw = hash.startsWith("#") ? hash.slice(1) : hash;
  try {
    return decodeURIComponent(raw);
  } catch {
    return raw;
  }
}

function encodeFragment(raw: string): string {
  return encodeURI(raw).replaceAll("#", "%23");
}

interface HashCodec<Value> {
  readonly parse: (raw: string) => Value;
  readonly serialize: (value: Value) => string;
  readonly default: Value;
}

/**
 * Track the URL fragment as a typed value decoded by `parse`.
 *
 * @typeParam Value Hash value type, inferred from the codec.
 * @param options Codec, host, history mode, and server fallback.
 * @returns The typed hash value and its controls.
 */
export function useUrlHash<Value>(options: UseTypedUrlHashOptions<Value>): UrlHashControls<Value>;

/**
 * Track the URL fragment as a plain string (without the leading `#`).
 *
 * @param options Host, history mode, and server fallback.
 * @default options {}
 * @returns The hash string and its controls.
 */
export function useUrlHash(options?: UseUrlHashOptions): UrlHashControls<string>;

/**
 * Synchronize a typed value with the URL fragment (`location.hash`).
 *
 * Reading follows `hashchange` and `popstate`. Writing `state` updates the
 * fragment through `history.replaceState` (or `pushState` with
 * `mode: "push"`), which neither scrolls to an anchor nor fires
 * `hashchange`; an empty serialized value removes the fragment. Values are
 * percent-decoded on read and minimally encoded on write.
 *
 * Server rendering: browsers never send the fragment, so without a host the
 * value is `ssrHash` (when given) or the default, and no global is read. Use
 * `initialRead: "post-flush"` when the markup depends on the hash to avoid a
 * hydration mismatch. Listeners are removed with the owning reactive scope.
 *
 * @example
 * ```ts
 * const { state: tab } = useUrlHash({
 *   parse: (raw) => (raw === "settings" ? "settings" : "profile"),
 *   serialize: (value) => value,
 *   default: "profile" as "profile" | "settings",
 * });
 * ```
 *
 * @param options Codec, host, history mode, and server fallback.
 * @default options {}
 * @returns The synchronized value and its controls.
 */
export function useUrlHash<Value>(
  options: UseUrlHashOptions | UseTypedUrlHashOptions<Value> = {},
): UrlHashControls<Value> | UrlHashControls<string> {
  if ("parse" in options) {
    return createUrlHash<Value>(
      { parse: options.parse, serialize: options.serialize, default: options.default },
      options,
    );
  }
  return createUrlHash<string>({ parse: (raw) => raw, serialize: String, default: "" }, options);
}

function createUrlHash<Value>(
  codec: HashCodec<Value>,
  options: UseUrlHashOptions,
): UrlHashControls<Value> {
  const supported = ref(false);
  const error = shallowRef<unknown>(undefined);
  let ready = false;
  let syncedRaw: string | undefined;

  const resolveHost = (): UrlHashHost | undefined =>
    options.host === undefined ? browserHost() : (toValue(options.host) ?? undefined);

  const parse = (raw: string): Value => {
    if (raw === "") return codec.default;
    try {
      const value = codec.parse(raw);
      error.value = undefined;
      return value;
    } catch (cause) {
      error.value = cause;
      return codec.default;
    }
  };

  const ssrRaw = toValue(options.ssrHash);
  const state = shallowRef<Value>(ssrRaw === undefined ? codec.default : parse(ssrRaw));

  const assign = (raw: string): void => {
    syncedRaw = raw;
    state.value = parse(raw);
  };

  const refresh = (): void => {
    const host = resolveHost();
    supported.value = host !== undefined;
    if (host === undefined) {
      const fallback = toValue(options.ssrHash);
      assign(fallback ?? "");
      return;
    }
    assign(decodeFragment(host.location.hash));
  };

  const onNavigate = (): void => {
    if (ready) refresh();
  };

  const stopHost = watch(
    resolveHost,
    (host, _previous, onCleanup) => {
      if (ready) refresh();
      if (host === undefined) return;
      host.addEventListener("hashchange", onNavigate);
      host.addEventListener("popstate", onNavigate);
      onCleanup(() => {
        host.removeEventListener("hashchange", onNavigate);
        host.removeEventListener("popstate", onNavigate);
      });
    },
    { immediate: true, flush: "sync" },
  );

  const stopWrite = watch(
    state,
    (value) => {
      const host = resolveHost();
      if (!ready || host === undefined) return;
      const raw = codec.serialize(value);
      if (raw === syncedRaw) return;
      syncedRaw = raw;
      const { pathname, search } = host.location;
      const url = `${pathname}${search}${raw === "" ? "" : `#${encodeFragment(raw)}`}`;
      if ((options.mode ?? "replace") === "push") host.history.pushState(null, "", url);
      else host.history.replaceState(host.history.state, "", url);
    },
    { flush: "sync" },
  );

  const start = (): void => {
    ready = true;
    refresh();
  };
  let stopDeferred: (() => void) | undefined;
  if ((options.initialRead ?? "sync") === "sync") {
    start();
  } else {
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
    stopHost();
    stopWrite();
  });

  return { state, supported: readonly(supported), error, refresh };
}
