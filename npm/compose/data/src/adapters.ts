import { defineDataResource, hydrateDataClient } from "./runtime.ts";
import type {
  DataLoadContext,
  DataResourceDefinition,
  DataResourceKey,
  DataResourceMeta,
  DataSnapshot,
  DataSourceSpecifier,
  SerializedDataState,
} from "./types.ts";

export class DataHttpError extends Error {
  readonly status: number;
  readonly statusText: string;
  readonly body: unknown;

  constructor(response: Response, body: unknown) {
    super(`HTTP ${response.status} ${response.statusText}`.trim());
    this.name = "DataHttpError";
    this.status = response.status;
    this.statusText = response.statusText;
    this.body = body;
  }
}

export interface FetchDataResourceOptions<
  Key extends DataResourceKey,
  Input,
  Data,
  ErrorPayload = unknown,
  Meta extends DataResourceMeta = DataResourceMeta,
> {
  readonly key: Key;
  readonly source: DataSourceSpecifier;
  readonly endpoint: (input: Input) => string | URL;
  readonly init?: (input: Input) => RequestInit;
  readonly fetch?: typeof globalThis.fetch;
  readonly decode?: (response: Response) => Data | Promise<Data>;
  readonly decodeError?: (response: Response) => unknown | Promise<unknown>;
  readonly serializeError?: (error: unknown) => ErrorPayload;
  readonly meta?: Meta;
}

/** Adapt a typed HTTP endpoint without reading `fetch` during module evaluation. */
export function createFetchDataResource<
  const Key extends DataResourceKey,
  Input,
  Data,
  ErrorPayload = unknown,
  const Meta extends DataResourceMeta = DataResourceMeta,
>(
  options: FetchDataResourceOptions<Key, Input, Data, ErrorPayload, Meta>,
): DataResourceDefinition<Key, Input, Data, ErrorPayload, Meta> {
  return defineDataResource({
    key: options.key,
    source: options.source,
    ...(options.meta === undefined ? {} : { meta: options.meta }),
    ...(options.serializeError === undefined ? {} : { serializeError: options.serializeError }),
    supports: () =>
      options.fetch !== undefined || typeof globalThis.fetch === "function" || "fetch unavailable",
    async loader({ input, signal }) {
      const implementation = options.fetch ?? globalThis.fetch;
      const response = await implementation(options.endpoint(input), {
        ...options.init?.(input),
        signal,
      });
      if (!response.ok) {
        const body = options.decodeError
          ? await options.decodeError(response)
          : await response.text().catch(() => undefined);
        throw new DataHttpError(response, body);
      }
      return options.decode ? await options.decode(response) : ((await response.json()) as Data);
    },
  });
}

export interface StreamDataResourceOptions<
  Key extends DataResourceKey,
  Input,
  Chunk,
  Data,
  ErrorPayload = unknown,
  Meta extends DataResourceMeta = DataResourceMeta,
> {
  readonly key: Key;
  readonly source: DataSourceSpecifier;
  readonly initial: (input: Input) => Data;
  readonly open: (context: DataLoadContext<Input>) => AsyncIterable<Chunk>;
  readonly reduce: (data: Data, chunk: Chunk) => Data;
  readonly serializeError?: (error: unknown) => ErrorPayload;
  readonly meta?: Meta;
}

/** Collect an abort-aware async stream into one typed resource state. */
export function createStreamDataResource<
  const Key extends DataResourceKey,
  Input,
  Chunk,
  Data,
  ErrorPayload = unknown,
  const Meta extends DataResourceMeta = DataResourceMeta,
>(
  options: StreamDataResourceOptions<Key, Input, Chunk, Data, ErrorPayload, Meta>,
): DataResourceDefinition<Key, Input, Data, ErrorPayload, Meta> {
  return defineDataResource({
    key: options.key,
    source: options.source,
    ...(options.meta === undefined ? {} : { meta: options.meta }),
    ...(options.serializeError === undefined ? {} : { serializeError: options.serializeError }),
    async loader(context) {
      let data = options.initial(context.input);
      for await (const chunk of options.open(context)) {
        context.signal.throwIfAborted();
        data = options.reduce(data, chunk);
      }
      return data;
    },
  });
}

/** Reconstruct an SSR cache from an incremental snapshot stream. */
export async function hydrateDataClientFromStream(
  stream: AsyncIterable<SerializedDataState>,
  options: Parameters<typeof hydrateDataClient>[1] = {},
) {
  const resources: SerializedDataState[] = [];
  for await (const state of stream) resources.push(state);
  const snapshot: DataSnapshot = { schemaVersion: 1, resources };
  return hydrateDataClient(snapshot, options);
}

export interface DataTestAdapter<Input, Data> {
  readonly calls: readonly DataLoadContext<Input>[];
  readonly loader: (context: DataLoadContext<Input>) => Promise<Data>;
  readonly enqueue: (
    result: Data | Error | ((context: DataLoadContext<Input>) => Data | Promise<Data>),
  ) => void;
}

/** Deterministic queue-backed loader for resource and hydration tests. */
export function createDataTestAdapter<Input, Data>(): DataTestAdapter<Input, Data> {
  const calls: DataLoadContext<Input>[] = [];
  const results: Array<Data | Error | ((context: DataLoadContext<Input>) => Data | Promise<Data>)> =
    [];
  return {
    calls,
    enqueue: (result) => results.push(result),
    async loader(context) {
      calls.push(context);
      const result = results.shift();
      if (result === undefined) throw new Error("No queued data test result");
      if (result instanceof Error) throw result;
      return typeof result === "function"
        ? await (result as (context: DataLoadContext<Input>) => Data | Promise<Data>)(context)
        : result;
    },
  };
}
