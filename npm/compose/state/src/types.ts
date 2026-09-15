/** Source-owned Vue file that documents where a state model is used. */
export type StateSourceSpecifier = `${string}.vue`;

/** Stable state model key. Prefer domain-qualified literals such as `cart.session`. */
export type StateModelKey = string;

/** Serializable metadata that generators, docs, and devtools can consume. */
export type StateModelMeta = Readonly<Record<string, unknown>>;

/** Reducer action accepted by a state model. */
export type StateAction = Readonly<{ type: string }>;

/** Pure reducer used by server renderers and clients. */
export type StateReducer<State, Action extends StateAction> = {
  bivarianceHack(state: Readonly<State>, action: Action): State;
}["bivarianceHack"];

/** Command execution context. Commands return one or more reducer actions. */
export interface StateCommandContext<State> {
  readonly state: Readonly<State>;
}

/** Command factory that maps typed arguments to reducer actions. */
export type StateCommand<State, Action extends StateAction, Args extends readonly unknown[]> = (
  context: StateCommandContext<State>,
  ...args: Args
) => Action | readonly Action[];

/** Named command registry for a model. */
export type StateCommandMap<State, Action extends StateAction> = Readonly<
  Record<string, StateCommand<State, Action, readonly never[]>>
>;

/** Versioned migration called when a persisted or SSR snapshot is stale. */
export type StateMigration<State> = (state: unknown, context: StateMigrationContext) => State;

/** Context supplied to migration functions. */
export interface StateMigrationContext {
  readonly fromVersion: number;
  readonly toVersion: number;
  readonly key: StateModelKey;
}

/** Optional persistence contract attached to a model. */
export interface StatePersistenceDefinition<State> {
  /** Stable storage key used by persistence adapters. */
  readonly storageKey: string;

  /**
   * Expire persisted payloads older than this many milliseconds.
   *
   * @default undefined
   */
  readonly ttlMs?: number;

  /**
   * Validate a restored state payload before hydration.
   *
   * @default accepts any migrated payload
   */
  readonly validate?: (state: unknown) => state is State;

  /**
   * Upgrade old persisted payloads to the current model version.
   *
   * @default only same-version payloads are accepted
   */
  readonly migrate?: StateMigration<State>;
}

/** Typed state model accepted by createStateStore. */
export interface StateModelDefinition<
  Key extends StateModelKey = StateModelKey,
  State = unknown,
  Action extends StateAction = StateAction,
  Commands extends StateCommandMap<State, Action> = StateCommandMap<State, Action>,
  Meta extends StateModelMeta = StateModelMeta,
> {
  readonly key: Key;
  readonly source: StateSourceSpecifier;
  readonly version: number;
  readonly initialState: State;
  readonly reducer: StateReducer<State, Action>;
  readonly commands?: Commands;
  readonly persistence?: StatePersistenceDefinition<State>;
  readonly meta?: Meta;
}

export type AnyStateModelDefinition = StateModelDefinition<
  StateModelKey,
  any,
  StateAction,
  StateCommandMap<any, StateAction>,
  StateModelMeta
>;

export type InferState<Model> =
  Model extends StateModelDefinition<StateModelKey, infer State> ? State : never;

export type InferAction<Model> =
  Model extends StateModelDefinition<StateModelKey, infer _State, infer Action> ? Action : never;

export type InferCommands<Model> =
  Model extends StateModelDefinition<StateModelKey, infer _State, infer _Action, infer Commands>
    ? Commands
    : never;

export type InferKey<Model> = Model extends { readonly key: infer Key extends StateModelKey }
  ? Key
  : StateModelKey;

/** Store snapshot embedded into SSR HTML or persistence adapters. */
export interface StateSnapshot<Key extends StateModelKey = StateModelKey, State = unknown> {
  readonly schemaVersion: 1;
  readonly key: Key;
  readonly source: StateSourceSpecifier;
  readonly version: number;
  readonly state: State;
  readonly updatedAt: number;
}

export type StateHydrationStatus =
  | "accepted"
  | "expired"
  | "invalid"
  | "migrated"
  | "missing"
  | "mismatch";

export interface StateHydrationResult<State> {
  readonly status: StateHydrationStatus;
  readonly state: State;
  readonly recovered: boolean;
}

/** Synchronous persistence adapter that is also safe in SSR tests. */
export interface StatePersistenceAdapter {
  readonly getItem: (key: string) => string | undefined;
  readonly setItem: (key: string, value: string) => void;
  readonly removeItem: (key: string) => void;
}

export interface CreateStateStoreOptions<State> {
  readonly now?: () => number;
  readonly snapshot?: StateSnapshot<StateModelKey, State>;
}

export interface StateTransaction<State, Action extends StateAction> {
  readonly id: number;
  readonly actions: readonly Action[];
  readonly before: State;
  readonly after: State;
}

export interface StateOptimisticResult<State> {
  readonly status: "committed" | "rolled-back";
  readonly state: State;
  readonly reason?: unknown;
}

/** Runtime store returned by createStateStore. */
export interface StateStore<Model extends AnyStateModelDefinition> {
  readonly key: InferKey<Model>;
  readonly persistenceKey: string;
  readonly source: Model["source"];
  readonly version: Model["version"];
  readonly state: InferState<Model>;
  readonly dispatch: (action: InferAction<Model>) => InferState<Model>;
  readonly command: <Name extends keyof InferCommands<Model> & string>(
    name: Name,
    ...args: CommandArguments<InferCommands<Model>[Name]>
  ) => InferState<Model>;
  readonly transaction: (
    actions: readonly InferAction<Model>[],
  ) => StateTransaction<InferState<Model>, InferAction<Model>>;
  readonly optimistic: (
    action: InferAction<Model>,
    confirm: () => void | Promise<void>,
  ) => Promise<StateOptimisticResult<InferState<Model>>>;
  readonly rollback: (transaction: StateTransaction<InferState<Model>, InferAction<Model>>) => void;
  readonly snapshot: () => StateSnapshot<InferKey<Model>, InferState<Model>>;
}

export type CommandArguments<Command> = Command extends (
  context: StateCommandContext<any>,
  ...args: infer Args
) => StateAction | readonly StateAction[]
  ? Args
  : never;

/** Machine-readable state manifest for generators and docs. */
export interface StateManifest {
  readonly schemaVersion: 1;
  readonly models: readonly StateManifestEntry[];
}

export interface StateManifestEntry {
  readonly key: StateModelKey;
  readonly source: StateSourceSpecifier;
  readonly version: number;
  readonly meta: StateModelMeta;
  readonly persistence: boolean;
}

/** Finite-state transition map used for literal transition type checks. */
export type StateTransitionTable = Readonly<Record<string, readonly string[]>>;

export type AllowedTransition<
  Table extends StateTransitionTable,
  From extends keyof Table & string,
> = Table[From][number] & string;
