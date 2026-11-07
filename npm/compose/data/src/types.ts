/** Source-owned Vue file that documents where a resource is used. */
export type DataSourceSpecifier = `${string}.vue`;

/** Serializable metadata that generators and devtools can consume. */
export type DataResourceMeta = Readonly<Record<string, unknown>>;

/** Stable resource key. Prefer domain-qualified literals such as `article.byId`. */
export type DataResourceKey = string;

/** Reason a fetch was started or refreshed. */
export type DataLoadReason = "initial" | "hydrate" | "refresh" | "retry" | "optimistic";

/** Context passed to every resource loader. */
export interface DataLoadContext<Input = unknown> {
  readonly key: DataResourceKey;
  readonly input: Input;
  readonly meta: DataResourceMeta;
  readonly reason: DataLoadReason;
  readonly signal: AbortSignal;
}

/** Loader invoked by server renderers, client hydration, or manual refreshes. */
export type DataLoader<Input, Data> = {
  bivarianceHack(context: DataLoadContext<Input>): Data | Promise<Data>;
}["bivarianceHack"];

/** Host capability check for a resource input. Return a reason to explain unsupported hosts. */
export type DataSupport<Input> = {
  bivarianceHack(input: Input): boolean | string;
}["bivarianceHack"];

/** Typed data endpoint accepted by createDataClient. */
export interface DataResourceDefinition<
  Key extends DataResourceKey = DataResourceKey,
  Input = unknown,
  Data = unknown,
  ErrorPayload = unknown,
  Meta extends DataResourceMeta = DataResourceMeta,
> {
  readonly key: Key;
  readonly source: DataSourceSpecifier;
  readonly meta?: Meta;
  readonly loader: DataLoader<Input, Data>;
  readonly serializeError?: (error: unknown) => ErrorPayload;
  /** Return false or a reason when the active host cannot load this resource. */
  readonly supports?: DataSupport<Input>;
}

/** Broad resource definition used by registries, clients, and generated manifests. */
export type AnyDataResourceDefinition = DataResourceDefinition<
  DataResourceKey,
  unknown,
  unknown,
  unknown,
  DataResourceMeta
>;

/** Resolve the successful payload type declared by a resource. */
export type InferData<Resource> =
  Resource extends DataResourceDefinition<DataResourceKey, infer _Input, infer Data> ? Data : never;

/** Resolve the serialized error payload type declared by a resource. */
export type InferError<Resource> =
  Resource extends DataResourceDefinition<
    DataResourceKey,
    infer _Input,
    infer _Data,
    infer ErrorPayload
  >
    ? ErrorPayload
    : never;

/** Resolve the loader input type declared by a resource. */
export type InferInput<Resource> =
  Resource extends DataResourceDefinition<infer _Key, infer Input> ? Input : never;

/** Resolve a resource's literal key, falling back to the public key type. */
export type InferKey<Resource> = Resource extends {
  readonly key: infer Key extends DataResourceKey;
}
  ? Key
  : DataResourceKey;

/** Complete lifecycle state for one keyed resource request. */
export type DataState<
  Key extends DataResourceKey = DataResourceKey,
  Input = unknown,
  Data = unknown,
  ErrorPayload = unknown,
> =
  | DataIdleState<Key, Input>
  | DataPendingState<Key, Input>
  | DataSuccessState<Key, Input, Data>
  | DataStaleState<Key, Input, Data>
  | DataCancelledState<Key, Input>
  | DataUnsupportedState<Key, Input>
  | DataErrorState<Key, Input, ErrorPayload>;

/** Identity shared by every resource lifecycle state. */
export interface DataStateBase<Key extends DataResourceKey, Input> {
  readonly key: Key;
  readonly input: Input;
  readonly requestKey: string;
}

/** Resource request that has not started. */
export interface DataIdleState<Key extends DataResourceKey, Input> extends DataStateBase<
  Key,
  Input
> {
  readonly status: "idle";
}

/** Resource request with an active loader attempt. */
export interface DataPendingState<Key extends DataResourceKey, Input> extends DataStateBase<
  Key,
  Input
> {
  readonly status: "pending";
  readonly attempt: number;
  readonly startedAt: number;
}

/** Resource request whose current data completed successfully. */
export interface DataSuccessState<Key extends DataResourceKey, Input, Data> extends DataStateBase<
  Key,
  Input
> {
  readonly status: "success";
  readonly data: Data;
  readonly updatedAt: number;
}

/** Previously successful data marked for refresh or optimistic reconciliation. */
export interface DataStaleState<Key extends DataResourceKey, Input, Data> extends DataStateBase<
  Key,
  Input
> {
  readonly status: "stale";
  readonly data: Data;
  readonly staleAt: number;
  readonly updatedAt: number;
  readonly reason?: string;
}

/** Resource request cancelled before it could commit data. */
export interface DataCancelledState<Key extends DataResourceKey, Input> extends DataStateBase<
  Key,
  Input
> {
  readonly status: "cancelled";
  readonly cancelledAt: number;
  readonly reason: string;
}

/** Resource request rejected by its host capability check. */
export interface DataUnsupportedState<Key extends DataResourceKey, Input> extends DataStateBase<
  Key,
  Input
> {
  readonly status: "unsupported";
  readonly reason: string;
}

/** Resource request that exhausted its loader and retry policy. */
export interface DataErrorState<
  Key extends DataResourceKey,
  Input,
  ErrorPayload,
> extends DataStateBase<Key, Input> {
  readonly status: "error";
  readonly error: ErrorPayload;
  readonly updatedAt: number;
}

/** Context used to compute deterministic retry backoff. */
export interface DataRetryDelayContext<
  Key extends DataResourceKey = DataResourceKey,
  Input = unknown,
> {
  readonly key: Key;
  readonly input: Input;
  readonly attempt: number;
  readonly error: unknown;
}

/** Retry backoff in milliseconds or a deterministic backoff factory. */
export type DataRetryDelay<Key extends DataResourceKey = DataResourceKey, Input = unknown> =
  | number
  | ((context: DataRetryDelayContext<Key, Input>) => number);

/** Testable sleep hook used by retry backoff. */
export type DataSleep = (milliseconds: number, signal: AbortSignal) => Promise<void>;

/** Per-request cache, cancellation, and retry policy. */
export interface DataLoadOptions<Key extends DataResourceKey = DataResourceKey, Input = unknown> {
  /**
   * Reason exposed to the first loader attempt.
   *
   * @default "initial"
   */
  readonly reason?: DataLoadReason;

  /**
   * External abort signal linked to the request-owned controller.
   *
   * @default undefined
   */
  readonly signal?: AbortSignal;

  /**
   * Cache and in-flight request policy.
   *
   * @default "dedupe"
   */
  readonly policy?: "dedupe" | "cache-first" | "revalidate" | "replace" | "stale-while-revalidate";
  /**
   * Additional attempts after the first loader call.
   *
   * @default 0
   */
  readonly retries?: number;

  /**
   * Backoff before retry attempts.
   *
   * @default 0
   */
  readonly retryDelayMs?: DataRetryDelay<Key, Input>;

  /**
   * Abort the load after this many milliseconds.
   *
   * @default undefined
   */
  readonly deadlineMs?: number;
}

/** Cache-owning resource client used by SSR and hydrated applications. */
export interface DataClient {
  readonly load: <Resource extends AnyDataResourceDefinition>(
    resource: Resource,
    input: InferInput<Resource>,
    options?: DataLoadOptions<InferKey<Resource>, InferInput<Resource>>,
  ) => Promise<
    DataState<InferKey<Resource>, InferInput<Resource>, InferData<Resource>, InferError<Resource>>
  >;
  readonly peek: <Resource extends AnyDataResourceDefinition>(
    resource: Resource,
    input: InferInput<Resource>,
  ) =>
    | DataState<InferKey<Resource>, InferInput<Resource>, InferData<Resource>, InferError<Resource>>
    | undefined;
  readonly invalidate: <Resource extends AnyDataResourceDefinition>(
    resource: Resource,
    input?: InferInput<Resource>,
    reason?: string,
  ) => number;
  readonly optimistic: <Resource extends AnyDataResourceDefinition>(
    resource: Resource,
    input: InferInput<Resource>,
    update: (current: InferData<Resource>) => InferData<Resource>,
  ) => () => boolean;
  readonly cancel: <Resource extends AnyDataResourceDefinition>(
    resource: Resource,
    input: InferInput<Resource>,
    reason?: unknown,
  ) => boolean;
  readonly subscribe: (listener: (state: DataState) => void) => () => void;
  readonly snapshot: () => DataSnapshot;
}

/** Deterministic hooks and initial cache state for a data client. */
export interface DataClientOptions {
  /**
   * Clock used for lifecycle timestamps.
   *
   * @default Date.now
   */
  readonly now?: () => number;

  /**
   * Server snapshot used to seed the cache.
   *
   * @default undefined
   */
  readonly snapshot?: DataSnapshot;

  /**
   * Abort-aware wait implementation used by retry backoff.
   *
   * @default built-in setTimeout-based sleep
   */
  readonly sleep?: DataSleep;
}

/** Versioned collection of serializable resource states for hydration. */
export interface DataSnapshot {
  readonly schemaVersion: 1;
  readonly resources: readonly SerializedDataState[];
}

/** Resource states that may cross the server-to-client boundary. */
export type SerializedDataState =
  | DataSuccessState<DataResourceKey, unknown, unknown>
  | DataStaleState<DataResourceKey, unknown, unknown>
  | DataCancelledState<DataResourceKey, unknown>
  | DataUnsupportedState<DataResourceKey, unknown>
  | DataErrorState<DataResourceKey, unknown, unknown>;

/** Versioned resource metadata emitted for tooling and documentation. */
export interface DataManifest {
  readonly schemaVersion: 1;
  readonly resources: readonly DataManifestEntry[];
}

/** One resource's stable key, source owner, and serializable metadata. */
export interface DataManifestEntry {
  readonly key: DataResourceKey;
  readonly source: DataSourceSpecifier;
  readonly meta: DataResourceMeta;
}
