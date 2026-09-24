import { hasInjectionContext, inject, provide, readonly, ref, shallowRef, watch } from "vue";
import type { InjectionKey, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** `SameSite` attribute values. */
export type CookieSameSite = "lax" | "strict" | "none";

/** Attributes written with a cookie (RFC 6265bis). */
export interface CookieAttributes {
  /**
   * `Path` attribute.
   *
   * @default "/" when written by {@link useCookie}; omitted by {@link serializeCookie}
   */
  readonly path?: string;

  /**
   * `Domain` attribute.
   *
   * @default omitted (host-only cookie)
   */
  readonly domain?: string;

  /**
   * `Max-Age` in integer seconds; `0` or less expires the cookie.
   *
   * @default omitted (session cookie)
   */
  readonly maxAge?: number;

  /**
   * `Expires` date.
   *
   * @default omitted (session cookie)
   */
  readonly expires?: Date;

  /**
   * `SameSite` policy. `"none"` requires `secure`.
   *
   * @default omitted (browser default, usually lax)
   */
  readonly sameSite?: CookieSameSite;

  /**
   * `Secure` flag.
   *
   * @default false
   */
  readonly secure?: boolean;

  /**
   * `HttpOnly` flag. Only honored by server adapters: browsers ignore
   * cookies with this flag written through `document.cookie`.
   *
   * @default false
   */
  readonly httpOnly?: boolean;

  /**
   * `Partitioned` flag (CHIPS). Requires `secure`.
   *
   * @default false
   */
  readonly partitioned?: boolean;
}

const TOKEN = /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/;
const DOMAIN =
  /^\.?[0-9A-Za-z]([0-9A-Za-z-]*[0-9A-Za-z])?(\.[0-9A-Za-z]([0-9A-Za-z-]*[0-9A-Za-z])?)*$/;
// oxlint-disable-next-line no-control-regex -- control characters are exactly what is rejected.
const UNSAFE_PATH = /[\u0000-\u001f\u007f;]/;

function decode(value: string): string {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

/**
 * Parse a `Cookie` request header (or `document.cookie`) into name/value
 * pairs. Values are percent-decoded; the first occurrence of a name wins,
 * matching how browsers order more specific cookies first. Pure and
 * SSR-safe.
 *
 * @param header Raw header value.
 * @returns Cookie values by name.
 */
export function parseCookieHeader(header: string): Record<string, string> {
  const cookies = new Map<string, string>();
  for (const part of header.split(";")) {
    const separator = part.indexOf("=");
    if (separator === -1) continue;
    const name = part.slice(0, separator).trim();
    if (name === "" || cookies.has(name)) continue;
    let value = part.slice(separator + 1).trim();
    if (value.length >= 2 && value.startsWith('"') && value.endsWith('"')) {
      value = value.slice(1, -1);
    }
    cookies.set(name, decode(value));
  }
  return Object.fromEntries(cookies);
}

/**
 * Serialize one cookie as a `Set-Cookie` header value (also accepted by
 * `document.cookie`). The value is percent-encoded so any string is safe.
 *
 * @param name Cookie name; must be an RFC 6265 token.
 * @param value Unencoded cookie value.
 * @param attributes Cookie attributes.
 * @default attributes {}
 * @throws {TypeError} `[VIZE_COMPOSE_COOKIE_INVALID_NAME]`,
 * `[VIZE_COMPOSE_COOKIE_INVALID_DOMAIN]`, `[VIZE_COMPOSE_COOKIE_INVALID_PATH]`,
 * or `[VIZE_COMPOSE_COOKIE_REQUIRES_SECURE]` for malformed attributes.
 * @throws {RangeError} `[VIZE_COMPOSE_COOKIE_INVALID_MAX_AGE]` or
 * `[VIZE_COMPOSE_COOKIE_INVALID_EXPIRES]` for invalid lifetimes.
 * @returns The serialized cookie.
 */
export function serializeCookie(
  name: string,
  value: string,
  attributes: CookieAttributes = {},
): string {
  if (!TOKEN.test(name)) {
    throw new TypeError(
      `[VIZE_COMPOSE_COOKIE_INVALID_NAME] cookie names must be RFC 6265 tokens; received ${JSON.stringify(name)}`,
    );
  }
  let cookie = `${name}=${encodeURIComponent(value)}`;
  if (attributes.maxAge !== undefined) {
    if (!Number.isSafeInteger(attributes.maxAge)) {
      throw new RangeError(
        `[VIZE_COMPOSE_COOKIE_INVALID_MAX_AGE] maxAge must be an integer number of seconds; received ${String(attributes.maxAge)}`,
      );
    }
    cookie += `; Max-Age=${String(attributes.maxAge)}`;
  }
  if (attributes.expires !== undefined) {
    if (Number.isNaN(attributes.expires.getTime())) {
      throw new RangeError("[VIZE_COMPOSE_COOKIE_INVALID_EXPIRES] expires must be a valid Date");
    }
    cookie += `; Expires=${attributes.expires.toUTCString()}`;
  }
  if (attributes.domain !== undefined) {
    if (!DOMAIN.test(attributes.domain)) {
      throw new TypeError(
        `[VIZE_COMPOSE_COOKIE_INVALID_DOMAIN] invalid cookie domain ${JSON.stringify(attributes.domain)}`,
      );
    }
    cookie += `; Domain=${attributes.domain}`;
  }
  if (attributes.path !== undefined) {
    if (UNSAFE_PATH.test(attributes.path)) {
      throw new TypeError(
        `[VIZE_COMPOSE_COOKIE_INVALID_PATH] cookie paths cannot contain ';' or control characters; received ${JSON.stringify(attributes.path)}`,
      );
    }
    cookie += `; Path=${attributes.path}`;
  }
  const secure = attributes.secure ?? false;
  if ((attributes.sameSite === "none" || attributes.partitioned === true) && !secure) {
    throw new TypeError(
      "[VIZE_COMPOSE_COOKIE_REQUIRES_SECURE] SameSite=None and Partitioned cookies must be Secure",
    );
  }
  if (attributes.sameSite !== undefined) {
    const sameSite = attributes.sameSite;
    cookie += `; SameSite=${sameSite === "lax" ? "Lax" : sameSite === "strict" ? "Strict" : "None"}`;
  }
  if (secure) cookie += "; Secure";
  if (attributes.httpOnly === true) cookie += "; HttpOnly";
  if (attributes.partitioned === true) cookie += "; Partitioned";
  return cookie;
}

/**
 * Request/response bridge used by {@link useCookie}.
 *
 * On the server, `read` returns the incoming `Cookie` header and `write`
 * appends a `Set-Cookie` header to the response. In the browser the default
 * adapter reads and writes `document.cookie`.
 */
export interface CookieAdapter {
  /** Return the current `Cookie` header (`name=value; …`). */
  readonly read: () => string;

  /** Persist one serialized cookie (a `Set-Cookie` header value). */
  readonly write: (setCookie: string) => void;
}

/** Injection key for an app- or subtree-wide {@link CookieAdapter}. */
export const COOKIE_ADAPTER_KEY: InjectionKey<CookieAdapter> = Symbol("vize:cookie-adapter");

/**
 * Provide a {@link CookieAdapter} to descendant components (for example the
 * request/response pair of a server render). Must be called during setup;
 * use `app.provide(COOKIE_ADAPTER_KEY, adapter)` for a whole app.
 *
 * @param adapter Adapter used by descendants' {@link useCookie} calls.
 */
export function provideCookieAdapter(adapter: CookieAdapter): void {
  provide(COOKIE_ADAPTER_KEY, adapter);
}

/** Converts a cookie value to and from its string form. */
export interface CookieSerializer<Value> {
  /** Decode the percent-decoded cookie value. Throwing selects the default. */
  readonly read: (raw: string) => Value;

  /** Encode a value; the result is percent-encoded when written. */
  readonly write: (value: Value) => string;
}

/** Stable failure codes reported by {@link useCookie}. */
export type CookieErrorCode = "read-failed" | "invalid-value" | "write-failed";

/** Failure observed while synchronizing a cookie. */
export interface CookieFailure {
  /** Which synchronization step failed. */
  readonly code: CookieErrorCode;

  /** Cookie name involved in the failure. */
  readonly name: string;

  /** Exact thrown value, or the rejected candidate for `"invalid-value"`. */
  readonly cause: unknown;
}

/** Options for {@link useCookie}. */
export interface UseCookieOptions<Value> extends CookieAttributes {
  /**
   * Value used when the cookie is absent or unreadable. Also selects the
   * serializer: strings are stored raw, everything else as JSON.
   *
   * @default undefined (the state is `string | undefined`)
   */
  readonly default?: Value;

  /**
   * Explicit serializer.
   *
   * @default raw strings for string defaults, JSON otherwise
   */
  readonly serializer?: CookieSerializer<Value>;

  /**
   * Validation hook applied to every decoded value.
   *
   * @default a check that the value has the default's kind
   */
  readonly validate?: (candidate: unknown) => candidate is Value;

  /**
   * Request/response adapter. Takes precedence over an injected adapter.
   *
   * @default the injected adapter, else `document.cookie` in browsers
   */
  readonly adapter?: CookieAdapter | null;

  /**
   * Observe failures. Failures never throw out of the composable.
   *
   * @default undefined
   */
  readonly onError?: (failure: CookieFailure) => void;
}

/** Reactive state and controls returned by {@link useCookie}. */
export interface CookieControls<Value> {
  /** Writable cookie value; assignments write the cookie. */
  readonly state: Ref<Value>;

  /** Whether an adapter is attached. */
  readonly supported: Readonly<Ref<boolean>>;

  /** Most recent failure, cleared by the next success. */
  readonly error: Readonly<ShallowRef<CookieFailure | undefined>>;

  /** Re-read the cookie from the adapter. */
  readonly refresh: () => void;

  /** Expire the cookie and restore the default value. */
  readonly remove: () => void;
}

function browserAdapter(): CookieAdapter | undefined {
  if (typeof window === "undefined") return undefined;
  const document = window.document;
  return {
    read: () => document.cookie,
    write: (setCookie) => {
      document.cookie = setCookie;
    },
  };
}

function sameKind<Value>(defaults: Value, candidate: unknown): candidate is Value {
  if (defaults === null || defaults === undefined) return true;
  if (Array.isArray(defaults)) return Array.isArray(candidate);
  if (typeof defaults === "object") {
    return typeof candidate === "object" && candidate !== null && !Array.isArray(candidate);
  }
  return typeof candidate === typeof defaults;
}

interface CookieCodec {
  readonly read: (raw: string) => unknown;
  readonly write: (value: unknown) => string;
}

const rawCodec: CookieCodec = { read: (raw) => raw, write: (value) => String(value) };
const jsonCodec: CookieCodec = {
  read: (raw): unknown => JSON.parse(raw),
  write: (value) => JSON.stringify(value),
};

/**
 * Read and write one cookie with a typed default.
 *
 * @typeParam Value Cookie value type, inferred from `default`.
 * @param name Cookie name.
 * @param options Default, serializer, validation, attributes, and adapter.
 * @returns The typed cookie value and its controls.
 */
export function useCookie<Value>(
  name: string,
  options: UseCookieOptions<Value> & { readonly default: Value },
): CookieControls<Value>;

/**
 * Read and write one raw string cookie; absent cookies are `undefined`.
 *
 * @param name Cookie name.
 * @param options Attributes and adapter.
 * @returns The cookie string and its controls.
 */
export function useCookie(
  name: string,
  options?: UseCookieOptions<string>,
): CookieControls<string | undefined>;

/**
 * Synchronize a typed reactive value with one cookie, on the server and in
 * the browser.
 *
 * The adapter is resolved from `options.adapter`, then from
 * {@link COOKIE_ADAPTER_KEY} (see {@link provideCookieAdapter}), then from
 * `document.cookie` in browsers. During a server render with an adapter the
 * value comes from the request `Cookie` header and assignments emit
 * `Set-Cookie` headers through `adapter.write`, so server and client render
 * the same value and hydrate without a mismatch. Without any adapter (plain
 * SSR) the default is used and nothing is written.
 *
 * String defaults are stored raw; other defaults as JSON checked by
 * `validate` (by default: same kind as the default). Assigning `undefined`
 * (or calling `remove`) expires the cookie. Cookies are written with
 * `Path=/` unless `path` is given. Failures never throw; they are exposed
 * through `error` and `onError`. The write watcher stops with the owning
 * reactive scope.
 *
 * @example
 * ```ts
 * // server entry: app.provide(COOKIE_ADAPTER_KEY, { read: () => req.headers.cookie ?? "",
 * //   write: (value) => res.appendHeader("Set-Cookie", value) });
 * const { state: locale } = useCookie("locale", { default: "en", maxAge: 31_536_000 });
 * ```
 *
 * @param name Cookie name (an RFC 6265 token).
 * @param options Default, serializer, validation, attributes, and adapter.
 * @default options {}
 * @returns The synchronized value and its controls.
 */
export function useCookie<Value>(
  name: string,
  options: UseCookieOptions<Value> = {},
): CookieControls<Value | undefined> {
  const injected = hasInjectionContext() ? inject(COOKIE_ADAPTER_KEY, undefined) : undefined;
  const adapter =
    options.adapter === undefined ? (injected ?? browserAdapter()) : (options.adapter ?? undefined);
  const fallback = options.default;
  const codec: CookieCodec | CookieSerializer<Value> =
    options.serializer ??
    (fallback === undefined || typeof fallback === "string" ? rawCodec : jsonCodec);
  const validate =
    options.validate ?? ((candidate: unknown): candidate is Value => sameKind(fallback, candidate));
  const state = shallowRef<Value | undefined>(fallback);
  const error = shallowRef<CookieFailure | undefined>(undefined);
  const supported = ref(adapter !== undefined);
  const attributes: CookieAttributes = {
    path: "/",
    ...(options.domain === undefined ? {} : { domain: options.domain }),
    ...(options.path === undefined ? {} : { path: options.path }),
    ...(options.maxAge === undefined ? {} : { maxAge: options.maxAge }),
    ...(options.expires === undefined ? {} : { expires: options.expires }),
    ...(options.sameSite === undefined ? {} : { sameSite: options.sameSite }),
    ...(options.secure === undefined ? {} : { secure: options.secure }),
    ...(options.httpOnly === undefined ? {} : { httpOnly: options.httpOnly }),
    ...(options.partitioned === undefined ? {} : { partitioned: options.partitioned }),
  };
  let syncedRaw: string | undefined;

  const fail = (code: CookieErrorCode, cause: unknown): void => {
    const failure: CookieFailure = { code, name, cause };
    error.value = failure;
    options.onError?.(failure);
  };

  // Values read from the adapter (or restored defaults) are not written back.
  let applying = false;
  const assign = (value: Value | undefined, raw: string | undefined): void => {
    syncedRaw = raw;
    applying = true;
    try {
      state.value = value;
    } finally {
      applying = false;
    }
  };

  const refresh = (): void => {
    if (adapter === undefined) return;
    let raw: string | undefined;
    try {
      raw = parseCookieHeader(adapter.read())[name];
    } catch (cause) {
      fail("read-failed", cause);
      return;
    }
    if (raw === undefined) {
      assign(fallback, undefined);
      return;
    }
    let candidate: unknown;
    try {
      candidate = codec.read(raw);
    } catch (cause) {
      fail("read-failed", cause);
      assign(fallback, raw);
      return;
    }
    if (!validate(candidate)) {
      fail("invalid-value", candidate);
      assign(fallback, raw);
      return;
    }
    error.value = undefined;
    assign(candidate, raw);
  };

  const writeCookie = (value: Value | undefined): void => {
    if (adapter === undefined) return;
    try {
      if (value === undefined) {
        adapter.write(
          serializeCookie(name, "", { ...attributes, maxAge: 0, expires: new Date(0) }),
        );
        syncedRaw = undefined;
      } else {
        const raw = codec.write(value);
        if (raw === syncedRaw) return;
        adapter.write(serializeCookie(name, raw, attributes));
        syncedRaw = raw;
      }
    } catch (cause) {
      fail("write-failed", cause);
      return;
    }
    error.value = undefined;
  };

  refresh();

  const stopWrite = watch(
    state,
    (value) => {
      if (!applying) writeCookie(value);
    },
    { flush: "sync" },
  );

  const remove = (): void => {
    writeCookie(undefined);
    assign(fallback, undefined);
  };

  tryOnScopeDispose(stopWrite);

  return { state, supported: readonly(supported), error, refresh, remove };
}
