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
}

export type AnyDataResourceDefinition = DataResourceDefinition<
  DataResourceKey,
  unknown,
  unknown,
  unknown,
  DataResourceMeta
>;

export type InferData<Resource> =
  Resource extends DataResourceDefinition<DataResourceKey, infer _Input, infer Data> ? Data : never;

export type InferError<Resource> =
  Resource extends DataResourceDefinition<
    DataResourceKey,
    infer _Input,
    infer _Data,
    infer ErrorPayload
  >
    ? ErrorPayload
    : never;

export type InferInput<Resource> =
  Resource extends DataResourceDefinition<infer _Key, infer Input> ? Input : never;

export type InferKey<Resource> = Resource extends {
  readonly key: infer Key extends DataResourceKey;
}
  ? Key
  : DataResourceKey;

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

export interface DataStateBase<Key extends DataResourceKey, Input> {
  readonly key: Key;
  readonly input: Input;
  readonly requestKey: string;
}

export interface DataIdleState<Key extends DataResourceKey, Input> extends DataStateBase<
  Key,
  Input
> {
  readonly status: "idle";
}

export interface DataPendingState<Key extends DataResourceKey, Input> extends DataStateBase<
  Key,
  Input
> {
  readonly status: "pending";
  readonly attempt: number;
  readonly startedAt: number;
}

export interface DataSuccessState<Key extends DataResourceKey, Input, Data> extends DataStateBase<
  Key,
  Input
> {
  readonly status: "success";
  readonly data: Data;
  readonly updatedAt: number;
}

export interface DataStaleState<Key extends DataResourceKey, Input, Data> extends DataStateBase<
  Key,
  Input
> {
  readonly status: "stale";
  readonly data: Data;
  readonly staleAt: number;
  readonly updatedAt: number;
}

export interface DataCancelledState<Key extends DataResourceKey, Input> extends DataStateBase<
  Key,
  Input
> {
  readonly status: "cancelled";
  readonly cancelledAt: number;
  readonly reason: string;
}

export interface DataUnsupportedState<Key extends DataResourceKey, Input> extends DataStateBase<
  Key,
  Input
> {
  readonly status: "unsupported";
  readonly reason: string;
}

export interface DataErrorState<
  Key extends DataResourceKey,
  Input,
  ErrorPayload,
> extends DataStateBase<Key, Input> {
  readonly status: "error";
  readonly error: ErrorPayload;
  readonly updatedAt: number;
}

export interface DataLoadOptions {
  readonly reason?: DataLoadReason;
  readonly signal?: AbortSignal;
  readonly policy?: "dedupe" | "cache-first" | "revalidate" | "replace";
}

export interface DataClient {
  readonly load: <Resource extends AnyDataResourceDefinition>(
    resource: Resource,
    input: InferInput<Resource>,
    options?: DataLoadOptions,
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
  readonly snapshot: () => DataSnapshot;
}

export interface DataClientOptions {
  readonly now?: () => number;
  readonly snapshot?: DataSnapshot;
}

export interface DataSnapshot {
  readonly schemaVersion: 1;
  readonly resources: readonly SerializedDataState[];
}

export type SerializedDataState =
  | DataSuccessState<DataResourceKey, unknown, unknown>
  | DataStaleState<DataResourceKey, unknown, unknown>
  | DataCancelledState<DataResourceKey, unknown>
  | DataUnsupportedState<DataResourceKey, unknown>
  | DataErrorState<DataResourceKey, unknown, unknown>;

export interface DataManifest {
  readonly schemaVersion: 1;
  readonly resources: readonly DataManifestEntry[];
}

export interface DataManifestEntry {
  readonly key: DataResourceKey;
  readonly source: DataSourceSpecifier;
  readonly meta: DataResourceMeta;
}
