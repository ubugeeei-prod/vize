import type {
  AnyDataResourceDefinition,
  DataClient,
  DataClientOptions,
  DataLoadOptions,
  DataManifest,
  DataResourceDefinition,
  DataResourceKey,
  DataResourceMeta,
  DataRetryDelay,
  DataRetryDelayContext,
  DataSleep,
  DataSnapshot,
  DataState,
  DataStaleState,
  InferData,
  InferError,
  InferInput,
  InferKey,
  SerializedDataState,
} from "./types.ts";

const DATA_ERROR = {
  invalidSource: "VIZE_DATA_SOURCE_NOT_VUE",
  invalidKey: "VIZE_DATA_INVALID_KEY",
  invalidDeadline: "VIZE_DATA_INVALID_DEADLINE",
  invalidRetry: "VIZE_DATA_INVALID_RETRY",
} as const;

/** Define a typed SSR data resource and preserve literal endpoint metadata. */
export function defineDataResource<
  const Key extends DataResourceKey,
  Input,
  Data,
  ErrorPayload = unknown,
  const Meta extends DataResourceMeta = DataResourceMeta,
>(
  resource: DataResourceDefinition<Key, Input, Data, ErrorPayload, Meta>,
): DataResourceDefinition<Key, Input, Data, ErrorPayload, Meta> {
  validateResource(resource);
  return resource;
}

/** Create an SSR-safe data client with request deduplication and hydration cache. */
export function createDataClient(options: DataClientOptions = {}): DataClient {
  const now = options.now ?? Date.now;
  const sleep = options.sleep ?? defaultSleep;
  const cache = new Map<string, DataState>();
  const inflight = new Map<string, Promise<DataState>>();

  for (const state of options.snapshot?.resources ?? []) cache.set(state.requestKey, state);

  return {
    load(resource, input, loadOptions) {
      validateResource(resource);
      const requestKey = createRequestKey(resource.key, input);
      const cached = cache.get(requestKey);
      const policy = loadOptions?.policy ?? "dedupe";

      if (policy !== "revalidate" && policy !== "replace" && cached?.status === "success") {
        return Promise.resolve(cached as TypedState<typeof resource>);
      }
      if (policy === "cache-first" && cached != null && cached.status !== "pending") {
        return Promise.resolve(cached as TypedState<typeof resource>);
      }
      if (policy === "dedupe") {
        const current = inflight.get(requestKey);
        if (current) return current as Promise<TypedState<typeof resource>>;
      }

      const controller = createLinkedAbortController(loadOptions);
      const task = runLoadAttempt({
        cache,
        input,
        loadOptions,
        now,
        requestKey,
        resource,
        signal: controller.signal,
        sleep,
      }).finally(() => {
        if (inflight.get(requestKey) === task) inflight.delete(requestKey);
        controller.dispose();
      });

      inflight.set(requestKey, task);
      return task as Promise<TypedState<typeof resource>>;
    },
    peek(resource, input) {
      validateResource(resource);
      return cache.get(createRequestKey(resource.key, input)) as TypedState<typeof resource>;
    },
    invalidate(resource, input, reason = "manual") {
      validateResource(resource);
      const prefix = `${resource.key}:`;
      let changed = 0;
      for (const [requestKey, state] of cache) {
        if (input !== undefined && requestKey !== createRequestKey(resource.key, input)) continue;
        if (input === undefined && !requestKey.startsWith(prefix)) continue;
        if (state.status !== "success") continue;
        cache.set(requestKey, {
          ...state,
          status: "stale",
          staleAt: now(),
          reason,
        } as DataStaleState<DataResourceKey, unknown, unknown>);
        changed += 1;
      }
      return changed;
    },
    snapshot: () => ({
      schemaVersion: 1,
      resources: [...cache.values()].filter(isSerializableState),
    }),
  };
}

/** Serialize the current cache into an SSR payload. */
export function serializeDataSnapshot(client: Pick<DataClient, "snapshot">): DataSnapshot {
  return client.snapshot();
}

/** Hydrate a data client from a server-rendered payload. */
export function hydrateDataClient(
  snapshot: DataSnapshot,
  options: Omit<DataClientOptions, "snapshot"> = {},
) {
  return createDataClient({ ...options, snapshot });
}

/** Emit metadata for generators, documentation, and devtools. */
export function createDataManifest<const Resources extends readonly AnyDataResourceDefinition[]>(
  resources: Resources,
): DataManifest {
  for (const resource of resources) validateResource(resource);
  return {
    schemaVersion: 1,
    resources: resources.map((resource) => ({
      key: resource.key,
      source: resource.source,
      meta: resource.meta ?? {},
    })),
  };
}

/** Stream a snapshot as deterministic records for render pipelines. */
export async function* streamDataSnapshot(
  snapshot: DataSnapshot,
): AsyncGenerator<SerializedDataState, void, void> {
  for (const state of snapshot.resources) yield state;
}

type TypedState<Resource extends AnyDataResourceDefinition> = DataState<
  InferKey<Resource>,
  InferInput<Resource>,
  InferData<Resource>,
  InferError<Resource>
>;

interface LoadAttemptOptions<Resource extends AnyDataResourceDefinition> {
  cache: Map<string, DataState>;
  input: InferInput<Resource>;
  loadOptions: DataLoadOptions<InferKey<Resource>, InferInput<Resource>> | undefined;
  now: () => number;
  requestKey: string;
  resource: Resource;
  signal: AbortSignal;
  sleep: DataSleep;
}

async function runLoadAttempt<Resource extends AnyDataResourceDefinition>(
  options: LoadAttemptOptions<Resource>,
): Promise<DataState> {
  const retries = normalizeRetries(options.loadOptions?.retries);
  for (let attempt = 1; ; attempt += 1) {
    options.cache.set(options.requestKey, {
      status: "pending",
      key: options.resource.key,
      input: options.input,
      requestKey: options.requestKey,
      attempt,
      startedAt: options.now(),
    });

    try {
      if (options.signal.aborted) return cacheCancelled(options);
      const data = await options.resource.loader({
        key: options.resource.key,
        input: options.input,
        meta: options.resource.meta ?? {},
        reason: attempt === 1 ? (options.loadOptions?.reason ?? "initial") : "retry",
        signal: options.signal,
      });
      if (options.signal.aborted) return cacheCancelled(options);
      const success = {
        status: "success",
        key: options.resource.key,
        input: options.input,
        requestKey: options.requestKey,
        data,
        updatedAt: options.now(),
      } satisfies DataState;
      options.cache.set(options.requestKey, success);
      return success;
    } catch (error) {
      if (options.signal.aborted) return cacheCancelled(options);
      if (attempt > retries) {
        const failed = {
          status: "error",
          key: options.resource.key,
          input: options.input,
          requestKey: options.requestKey,
          error: options.resource.serializeError?.(error) ?? error,
          updatedAt: options.now(),
        } satisfies DataState;
        options.cache.set(options.requestKey, failed);
        return failed;
      }
      const delay = retryDelay(options.loadOptions?.retryDelayMs, {
        attempt,
        error,
        input: options.input,
        key: options.resource.key as InferKey<Resource>,
      });
      try {
        await options.sleep(delay, options.signal);
      } catch (sleepError) {
        if (options.signal.aborted) return cacheCancelled(options);
        const failed = {
          status: "error",
          key: options.resource.key,
          input: options.input,
          requestKey: options.requestKey,
          error: options.resource.serializeError?.(sleepError) ?? sleepError,
          updatedAt: options.now(),
        } satisfies DataState;
        options.cache.set(options.requestKey, failed);
        return failed;
      }
    }
  }
}

function cacheCancelled<Resource extends AnyDataResourceDefinition>(
  options: LoadAttemptOptions<Resource>,
): DataState {
  const cancelled = {
    status: "cancelled",
    key: options.resource.key,
    input: options.input,
    requestKey: options.requestKey,
    reason: String(options.signal.reason ?? "aborted"),
    cancelledAt: options.now(),
  } satisfies DataState;
  options.cache.set(options.requestKey, cancelled);
  return cancelled;
}

function normalizeRetries(retries: number | undefined): number {
  if (retries == null) return 0;
  if (!Number.isInteger(retries) || retries < 0) {
    throw new Error(`[${DATA_ERROR.invalidRetry}] retries must be a non-negative integer`);
  }
  return retries;
}

function retryDelay<Key extends DataResourceKey, Input>(
  retryDelayMs: DataRetryDelay<Key, Input> | undefined,
  context: DataRetryDelayContext<Key, Input>,
): number {
  const value = typeof retryDelayMs === "function" ? retryDelayMs(context) : (retryDelayMs ?? 0);
  if (!Number.isFinite(value) || value < 0) {
    throw new Error(`[${DATA_ERROR.invalidRetry}] retryDelayMs must be a non-negative number`);
  }
  return value;
}

function createLinkedAbortController(
  loadOptions: Pick<DataLoadOptions, "deadlineMs" | "signal"> | undefined,
) {
  const controller = new AbortController();
  const abort = () => controller.abort(loadOptions?.signal?.reason ?? "aborted");
  if (loadOptions?.signal?.aborted) abort();
  else loadOptions?.signal?.addEventListener("abort", abort, { once: true });

  let deadline: ReturnType<typeof setTimeout> | undefined;
  if (loadOptions?.deadlineMs != null) {
    if (!Number.isFinite(loadOptions.deadlineMs) || loadOptions.deadlineMs < 0) {
      throw new Error(`[${DATA_ERROR.invalidDeadline}] deadlineMs must be a non-negative number`);
    }
    if (loadOptions.deadlineMs === 0) controller.abort("deadline");
    else deadline = setTimeout(() => controller.abort("deadline"), loadOptions.deadlineMs);
  }

  return {
    signal: controller.signal,
    dispose() {
      if (deadline) clearTimeout(deadline);
      loadOptions?.signal?.removeEventListener("abort", abort);
    },
  };
}

function defaultSleep(milliseconds: number, signal: AbortSignal): Promise<void> {
  if (signal.aborted) return Promise.reject(signal.reason);
  if (milliseconds <= 0) return Promise.resolve();
  return new Promise((resolve, reject) => {
    const done = () => {
      signal.removeEventListener("abort", abort);
      resolve();
    };
    const timeout = setTimeout(done, milliseconds);
    const abort = () => {
      clearTimeout(timeout);
      reject(signal.reason);
    };
    signal.addEventListener("abort", abort, { once: true });
  });
}

function createRequestKey(key: DataResourceKey, input: unknown): string {
  return `${key}:${stableStringify(input)}`;
}

function stableStringify(value: unknown): string {
  if (value == null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(stableStringify).join(",")}]`;
  return `{${Object.entries(value as Record<string, unknown>)
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([key, item]) => `${JSON.stringify(key)}:${stableStringify(item)}`)
    .join(",")}}`;
}

function isSerializableState(state: DataState): state is SerializedDataState {
  return state.status !== "idle" && state.status !== "pending";
}

function validateResource(resource: Pick<DataResourceDefinition, "key" | "source">): void {
  if (!resource.key) throw new Error(`[${DATA_ERROR.invalidKey}] Data resource key is required`);
  if (!resource.source.endsWith(".vue")) {
    throw new Error(`[${DATA_ERROR.invalidSource}] Data resource ${resource.key} must use .vue`);
  }
}
