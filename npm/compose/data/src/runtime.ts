import type {
  AnyDataResourceDefinition,
  DataClient,
  DataClientOptions,
  DataManifest,
  DataResourceDefinition,
  DataResourceKey,
  DataResourceMeta,
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
  const cache = new Map<string, DataState>();
  const inflight = new Map<string, Promise<DataState>>();

  for (const state of options.snapshot?.resources ?? []) cache.set(state.requestKey, state);

  return {
    load(resource, input, loadOptions) {
      validateResource(resource);
      const requestKey = createRequestKey(resource.key, input);
      const cached = cache.get(requestKey);
      const policy = loadOptions?.policy ?? "dedupe";

      if (policy !== "revalidate" && cached?.status === "success") {
        return Promise.resolve(cached as TypedState<typeof resource>);
      }
      if (policy === "cache-first" && cached != null && cached.status !== "pending") {
        return Promise.resolve(cached as TypedState<typeof resource>);
      }
      if (policy === "dedupe") {
        const current = inflight.get(requestKey);
        if (current) return current as Promise<TypedState<typeof resource>>;
      }

      const controller = new AbortController();
      const abort = () => controller.abort(loadOptions?.signal?.reason ?? "aborted");
      if (loadOptions?.signal?.aborted) abort();
      else loadOptions?.signal?.addEventListener("abort", abort, { once: true });

      const pending = {
        status: "pending",
        key: resource.key,
        input,
        requestKey,
        attempt: 1,
        startedAt: now(),
      } satisfies DataState;
      cache.set(requestKey, pending);

      const task = Promise.resolve()
        .then(() =>
          resource.loader({
            key: resource.key,
            input,
            meta: resource.meta ?? {},
            reason: loadOptions?.reason ?? "initial",
            signal: controller.signal,
          }),
        )
        .then((data): DataState => {
          const success = {
            status: "success",
            key: resource.key,
            input,
            requestKey,
            data,
            updatedAt: now(),
          } satisfies DataState;
          cache.set(requestKey, success);
          return success;
        })
        .catch((error): DataState => {
          const failed = controller.signal.aborted
            ? {
                status: "cancelled",
                key: resource.key,
                input,
                requestKey,
                reason: String(controller.signal.reason ?? "aborted"),
                cancelledAt: now(),
              }
            : {
                status: "error",
                key: resource.key,
                input,
                requestKey,
                error: resource.serializeError?.(error) ?? error,
                updatedAt: now(),
              };
          cache.set(requestKey, failed as DataState);
          return failed as DataState;
        })
        .finally(() => {
          if (inflight.get(requestKey) === task) inflight.delete(requestKey);
          loadOptions?.signal?.removeEventListener("abort", abort);
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
