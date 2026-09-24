import { computed, readonly, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { retryAsync } from "./retry-async.ts";
import type { RetryAsyncOptions } from "./retry-async.ts";
import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Minimal `fetch` implementation accepted by {@link useFetch}. */
export type FetchImplementation = (input: string, init: RequestInit) => Promise<Response>;

/** Built-in body readers selectable with `responseType`. */
export type FetchResponseType = "json" | "text" | "blob" | "arrayBuffer" | "formData";

/** Data type produced by each built-in {@link FetchResponseType}. */
export interface FetchResponseTypeMap {
  /** Parsed JSON; narrow it with `validate`. */
  readonly json: unknown;
  /** Response text. */
  readonly text: string;
  /** Binary body as a `Blob`. */
  readonly blob: Blob;
  /** Binary body as an `ArrayBuffer`. */
  readonly arrayBuffer: ArrayBuffer;
  /** Multipart or URL-encoded form body. */
  readonly formData: FormData;
}

/** Lifecycle status of a {@link useFetch} resource. */
export type FetchStatus = "idle" | "pending" | "success" | "error" | "aborted";

/** Discriminated failure reported by {@link useFetch}. */
export type FetchFailure =
  | {
      /** The server answered with a non-2xx status. */
      readonly kind: "http";
      /** HTTP status code. */
      readonly status: number;
      /** The complete response, body unread. */
      readonly response: Response;
    }
  | {
      /** The request could not be performed (DNS, CORS, offline, ...). */
      readonly kind: "network";
      /** Exact value thrown by the fetch implementation. */
      readonly cause: unknown;
    }
  | {
      /** The body could not be decoded. */
      readonly kind: "parse";
      /** Exact value thrown by the body reader or `parse`. */
      readonly cause: unknown;
    }
  | {
      /** The decoded body was rejected by `validate`. */
      readonly kind: "invalid";
      /** Rejected decoded body. */
      readonly data: unknown;
    }
  | {
      /** The request was aborted by `abort()`, a newer execution, or the scope. */
      readonly kind: "aborted";
      /** Abort reason. */
      readonly reason: unknown;
    }
  | {
      /** The request exceeded `timeoutMs`. */
      readonly kind: "timeout";
      /** Configured timeout in milliseconds. */
      readonly timeoutMs: number;
    };

/** Explicit outcome of one {@link FetchControls.execute} call. */
export type FetchResult<Data> =
  | {
      /** The request succeeded. */
      readonly status: "success";
      /** Decoded, validated data. */
      readonly data: Data;
      /** Response the data was read from. */
      readonly response: Response;
    }
  | {
      /** The request failed; see {@link FetchFailure}. */
      readonly status: "error";
      /** Failure details. */
      readonly error: FetchFailure;
    }
  | {
      /** A newer execution replaced this one before it settled. */
      readonly status: "superseded";
    };

/** Request description passed through the `beforeRequest` interceptor. */
export interface FetchRequest {
  /** Resolved request URL. */
  readonly url: string;
  /** Request options (headers, method, body, ...). */
  readonly init: RequestInit;
}

/** Context supplied to the `beforeRequest` interceptor. */
export interface FetchBeforeRequestContext extends FetchRequest {
  /** Signal aborted when the execution is aborted, superseded, or times out. */
  readonly signal: AbortSignal;
}

/** Context supplied to the `afterResponse` interceptor. */
export interface FetchAfterResponseContext<Data> {
  /** Request that produced the response. */
  readonly request: FetchRequest;
  /** Successful response. */
  readonly response: Response;
  /** Decoded, validated data. */
  readonly data: Data;
}

/** Retry policy accepted by {@link UseFetchOptions.retry}. */
export type FetchRetryOptions = Omit<RetryAsyncOptions, "signal" | "scheduler">;

/** Options for {@link useFetch}. */
export interface UseFetchOptions<Data> {
  /**
   * Reactive request options. Changes trigger a refetch when `refetch` is on.
   *
   * @default {}
   */
  readonly init?: MaybeRefOrGetter<RequestInit | undefined>;

  /**
   * Fetch implementation. Passing one explicitly also enables automatic
   * execution on the server.
   *
   * @default window.fetch when a browser window exists
   */
  readonly fetch?: FetchImplementation;

  /**
   * Built-in body reader. Ignored when `parse` is supplied.
   *
   * @default "json"
   */
  readonly responseType?: FetchResponseType;

  /**
   * Custom body decoder. Throwing produces a `"parse"` failure.
   *
   * @default the `responseType` reader
   */
  readonly parse?: (response: Response) => Promise<Data>;

  /**
   * Type guard applied to the decoded body. Rejection produces an
   * `"invalid"` failure. Without it, JSON is trusted to match `Data`.
   *
   * @default accepts every decoded body
   */
  readonly validate?: (data: unknown) => data is Data;

  /**
   * Execute as soon as a URL is available. Automatic execution only happens
   * in a browser or when `fetch` is supplied explicitly.
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Re-execute when the reactive URL or `init` changes.
   *
   * @default true
   */
  readonly refetch?: boolean;

  /**
   * Data exposed before the first success and after `reset`-like failures.
   *
   * @default undefined
   */
  readonly initialData?: Data;

  /**
   * Abort the request after this many milliseconds (`"timeout"` failure).
   *
   * @default undefined (no timeout)
   */
  readonly timeoutMs?: number;

  /**
   * Retry count or full retry policy (see `retryAsync`). By default only
   * network failures and HTTP 408, 429, and 5xx responses are retried.
   *
   * @default 0
   */
  readonly retry?: number | FetchRetryOptions;

  /**
   * Timer host for timeouts and retry backoff.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;

  /**
   * Inspect or rewrite the request before it is sent. Return a replacement
   * request, nothing to keep it, or `false` to abort the execution.
   *
   * @default undefined
   */
  readonly beforeRequest?: (
    context: FetchBeforeRequestContext,
  ) => FetchRequest | false | void | Promise<FetchRequest | false | void>;

  /**
   * Observe or transform successful data before it is committed.
   *
   * @default undefined
   */
  readonly afterResponse?: (
    context: FetchAfterResponseContext<Data>,
  ) => Data | void | Promise<Data | void>;

  /**
   * Observe failures of the newest execution.
   *
   * @default undefined
   */
  readonly onError?: (failure: FetchFailure) => void;
}

/** Reactive state and controls returned by {@link useFetch}. */
export interface FetchControls<Data> {
  /** Data of the newest successful execution, or `initialData`. */
  readonly data: Readonly<ShallowRef<Data | undefined>>;
  /** Failure of the newest settled execution. */
  readonly error: Readonly<ShallowRef<FetchFailure | undefined>>;
  /** Lifecycle status driven by the newest execution. */
  readonly status: Readonly<Ref<FetchStatus>>;
  /** Response of the newest execution that reached the server. */
  readonly response: Readonly<ShallowRef<Response | undefined>>;
  /** HTTP status of `response`. */
  readonly statusCode: ComputedRef<number | undefined>;
  /** Whether an execution is pending. */
  readonly pending: ComputedRef<boolean>;
  /**
   * Run the request now, superseding any pending one. Never rejects. Await it
   * in `onServerPrefetch` to fetch during server rendering.
   */
  readonly execute: () => Promise<FetchResult<Data>>;
  /**
   * Abort the pending execution.
   *
   * @returns Whether an execution was aborted.
   */
  readonly abort: (reason?: unknown) => boolean;
}

interface Execution {
  readonly controller: AbortController;
  superseded: boolean;
}

class RetryableFailure {
  readonly failure: FetchFailure;
  constructor(failure: FetchFailure) {
    this.failure = failure;
  }
}

const TIMEOUT_REASON = "VIZE_COMPOSE_FETCH_TIMEOUT";

function browserFetch(): FetchImplementation | undefined {
  if (typeof window === "undefined" || typeof window.fetch !== "function") return undefined;
  const fetchImplementation = window.fetch.bind(window);
  return (input, init) => fetchImplementation(input, init);
}

function isRetryableStatus(status: number): boolean {
  return status === 408 || status === 429 || status >= 500;
}

function readBody(response: Response, responseType: FetchResponseType): Promise<unknown> {
  switch (responseType) {
    case "text":
      return response.text();
    case "blob":
      return response.blob();
    case "arrayBuffer":
      return response.arrayBuffer();
    case "formData":
      return response.formData();
    default:
      return response.json();
  }
}

function assertTimeout(timeoutMs: number | undefined): void {
  if (timeoutMs === undefined) return;
  if (!Number.isFinite(timeoutMs) || timeoutMs < 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_FETCH_INVALID_TIMEOUT] timeoutMs must be a finite non-negative number; received ${String(timeoutMs)}`,
    );
  }
}

/**
 * Fetch typed data with a custom `parse` decoder.
 *
 * @typeParam Data Value produced by `parse`.
 * @param url Reactive URL; `null`/`undefined` keeps the resource idle.
 * @param options Options including the decoder.
 * @returns Reactive state and controls.
 */
export function useFetch<Data>(
  url: MaybeRefOrGetter<string | URL | null | undefined>,
  options: UseFetchOptions<Data> & { readonly parse: (response: Response) => Promise<Data> },
): FetchControls<Data>;
/**
 * Fetch a non-JSON body whose type follows `responseType`.
 *
 * @param url Reactive URL; `null`/`undefined` keeps the resource idle.
 * @param options Options including the body reader.
 * @returns Reactive state and controls.
 */
export function useFetch<const Kind extends Exclude<FetchResponseType, "json">>(
  url: MaybeRefOrGetter<string | URL | null | undefined>,
  options: UseFetchOptions<FetchResponseTypeMap[Kind]> & { readonly responseType: Kind },
): FetchControls<FetchResponseTypeMap[Kind]>;
/**
 * Fetch JSON. Supply `validate` to narrow `unknown` to a checked type.
 *
 * @param url Reactive URL; `null`/`undefined` keeps the resource idle.
 * @param options Options.
 * @returns Reactive state and controls.
 */
export function useFetch<Data = unknown>(
  url: MaybeRefOrGetter<string | URL | null | undefined>,
  options?: UseFetchOptions<Data> & { readonly responseType?: "json" },
): FetchControls<Data>;
/**
 * Reactive, abortable, retrying `fetch` with typed decoding and interceptors.
 *
 * Latest wins: starting an execution aborts the pending one, which then
 * settles as `"superseded"` without touching state. The pending request is
 * aborted when the owning reactive scope stops; outside a scope the caller
 * owns `abort()`. Automatic execution (initial and on reactive URL/`init`
 * changes) only happens in a browser or with an explicit `fetch`, so server
 * rendering performs no I/O unless you `await execute()` in
 * `onServerPrefetch`; state then holds `initialData` and status `"idle"`.
 *
 * @example
 * ```ts
 * const id = ref(1);
 * const { data, error } = useFetch(() => `/api/users/${id.value}`, {
 *   validate: isUser,
 *   retry: 2,
 *   timeoutMs: 5_000,
 * });
 * ```
 *
 * @param url Reactive URL; `null`/`undefined` keeps the resource idle.
 * @param options Decoding, retry, timeout, and interceptor options.
 * @default options {}
 * @throws {RangeError} `[VIZE_COMPOSE_FETCH_INVALID_TIMEOUT]` for an invalid timeout.
 * @returns Reactive state and controls.
 */
export function useFetch<Data>(
  url: MaybeRefOrGetter<string | URL | null | undefined>,
  options: UseFetchOptions<Data> = {},
): FetchControls<Data> {
  assertTimeout(options.timeoutMs);
  const data = shallowRef<Data | undefined>(options.initialData);
  const error = shallowRef<FetchFailure | undefined>(undefined);
  const status = shallowRef<FetchStatus>("idle");
  const response = shallowRef<Response | undefined>(undefined);
  const statusCode = computed(() => response.value?.status);
  const pending = computed(() => status.value === "pending");
  const scheduler: TimeoutScheduler = options.scheduler ?? {
    setTimeout: (callback, delayMs) => {
      const handle = globalThis.setTimeout(callback, delayMs);
      return () => globalThis.clearTimeout(handle);
    },
    clearTimeout: (cancel) => {
      if (typeof cancel === "function") cancel();
    },
  };
  let active: Execution | undefined;

  const resolveFetch = (): FetchImplementation | undefined => options.fetch ?? browserFetch();

  const decode = async (result: Response): Promise<unknown> =>
    options.parse ? options.parse(result) : readBody(result, options.responseType ?? "json");

  const accepts = (candidate: unknown): candidate is Data =>
    // Without `validate`, the caller's `Data` argument is a declared trust
    // boundary for the decoded body (exactly like `Response.json()`).
    options.validate ? options.validate(candidate) : true;

  const fail = (failure: FetchFailure, record: Execution): FetchResult<Data> => {
    if (active !== record) return { status: "superseded" };
    active = undefined;
    error.value = failure;
    status.value = failure.kind === "aborted" ? "aborted" : "error";
    options.onError?.(failure);
    return { status: "error", error: failure };
  };

  const abortFailure = (signal: AbortSignal, timeoutMs: number | undefined): FetchFailure =>
    signal.reason === TIMEOUT_REASON && timeoutMs !== undefined
      ? { kind: "timeout", timeoutMs }
      : { kind: "aborted", reason: signal.reason };

  const execute = async (): Promise<FetchResult<Data>> => {
    if (active !== undefined) {
      active.superseded = true;
      active.controller.abort(new DOMException("A newer fetch started.", "AbortError"));
    }
    const record: Execution = { controller: new AbortController(), superseded: false };
    active = record;
    const { signal } = record.controller;
    error.value = undefined;
    status.value = "pending";

    const target = toValue(url);
    const fetchImplementation = resolveFetch();
    if (target === null || target === undefined || fetchImplementation === undefined) {
      return fail(
        {
          kind: "network",
          cause: new TypeError(
            target === null || target === undefined
              ? "[VIZE_COMPOSE_FETCH_NO_URL] no request URL is available"
              : "[VIZE_COMPOSE_FETCH_UNSUPPORTED] no fetch implementation is available",
          ),
        },
        record,
      );
    }

    const timeoutMs = options.timeoutMs;
    const timer =
      timeoutMs === undefined
        ? undefined
        : scheduler.setTimeout(() => record.controller.abort(TIMEOUT_REASON), timeoutMs);

    try {
      let request: FetchRequest = { url: String(target), init: { ...toValue(options.init) } };
      if (options.beforeRequest) {
        const replaced = await options.beforeRequest({ ...request, signal });
        if (replaced === false) {
          record.controller.abort(
            new DOMException("beforeRequest cancelled the request.", "AbortError"),
          );
        } else if (replaced) {
          request = replaced;
        }
      }
      if (signal.aborted) {
        return record.superseded
          ? { status: "superseded" }
          : fail(abortFailure(signal, timeoutMs), record);
      }

      const retry = options.retry ?? 0;
      const retryOptions: FetchRetryOptions =
        typeof retry === "number" ? { maximumRetries: retry, initialDelayMs: 0 } : retry;
      const userShouldRetry = retryOptions.shouldRetry;
      const attemptOutcome = await retryAsync(
        async () => {
          let result: Response;
          try {
            result = await fetchImplementation(request.url, { ...request.init, signal });
          } catch (cause) {
            throw new RetryableFailure({ kind: "network", cause });
          }
          if (!result.ok && isRetryableStatus(result.status)) {
            throw new RetryableFailure({ kind: "http", status: result.status, response: result });
          }
          return result;
        },
        {
          ...retryOptions,
          signal,
          scheduler,
          shouldRetry: (context) =>
            userShouldRetry ? userShouldRetry(context) : context.error instanceof RetryableFailure,
        },
      ).then(
        (result) => ({ ok: true, result }) as const,
        (cause: unknown) => ({ ok: false, cause }) as const,
      );

      if (!attemptOutcome.ok) {
        if (signal.aborted) {
          return record.superseded
            ? { status: "superseded" }
            : fail(abortFailure(signal, timeoutMs), record);
        }
        const { cause } = attemptOutcome;
        if (active === record && cause instanceof RetryableFailure) {
          if (cause.failure.kind === "http") response.value = cause.failure.response;
          return fail(cause.failure, record);
        }
        return fail({ kind: "network", cause }, record);
      }

      const result = attemptOutcome.result;
      if (active === record) response.value = result;
      if (!result.ok)
        return fail({ kind: "http", status: result.status, response: result }, record);

      let decoded: unknown;
      try {
        decoded = await decode(result);
      } catch (cause) {
        if (signal.aborted) {
          return record.superseded
            ? { status: "superseded" }
            : fail(abortFailure(signal, timeoutMs), record);
        }
        return fail({ kind: "parse", cause }, record);
      }
      if (!accepts(decoded)) return fail({ kind: "invalid", data: decoded }, record);
      let value: Data = decoded;
      if (options.afterResponse) {
        const transformed = await options.afterResponse({ request, response: result, data: value });
        if (transformed !== undefined) value = transformed;
      }
      if (active !== record) return { status: "superseded" };
      active = undefined;
      data.value = value;
      status.value = "success";
      return { status: "success", data: value, response: result };
    } catch (cause) {
      // Interceptor failures surface as network-level failures.
      return fail({ kind: "network", cause }, record);
    } finally {
      if (timer !== undefined) scheduler.clearTimeout(timer);
    }
  };

  const abort = (reason: unknown = new DOMException("The request was aborted.", "AbortError")) => {
    if (active === undefined) return false;
    active.controller.abort(reason);
    return true;
  };

  const automatic = options.fetch !== undefined || typeof window !== "undefined";
  if (automatic) {
    let initial = true;
    const stop = watch(
      [() => toValue(url), () => toValue(options.init)],
      ([target]) => {
        const first = initial;
        initial = false;
        if (target === null || target === undefined) return;
        if (first ? (options.immediate ?? true) : (options.refetch ?? true)) void execute();
      },
      { immediate: true, deep: true },
    );
    tryOnScopeDispose(stop);
  }

  tryOnScopeDispose(() => {
    abort(new DOMException("The reactive scope was disposed.", "AbortError"));
  });

  return {
    data,
    error,
    status: readonly(status),
    response,
    statusCode,
    pending,
    execute,
    abort,
  };
}
