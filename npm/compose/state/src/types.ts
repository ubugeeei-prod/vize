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

/** Erased state model used when a collection contains independently typed models. */
export type AnyStateModelDefinition = StateModelDefinition<
  StateModelKey,
  any,
  StateAction,
  StateCommandMap<any, StateAction>,
  StateModelMeta
>;

/** Infer the state value owned by a state model definition. */
export type InferState<Model> =
  Model extends StateModelDefinition<StateModelKey, infer State> ? State : never;

/** Infer the reducer action accepted by a state model definition. */
export type InferAction<Model> =
  Model extends StateModelDefinition<StateModelKey, infer _State, infer Action> ? Action : never;

/** Infer the named command map exposed by a state model definition. */
export type InferCommands<Model> =
  Model extends StateModelDefinition<StateModelKey, infer _State, infer _Action, infer Commands>
    ? Commands
    : never;

/** Infer a model's stable literal key, falling back to the general key type. */
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

/** Outcome of validating and restoring an SSR or persistence snapshot. */
export type StateHydrationStatus =
  | "accepted"
  | "expired"
  | "invalid"
  | "migrated"
  | "missing"
  | "mismatch";

/** Restored state together with its validation and recovery outcome. */
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

/** Runtime controls for creating a deterministic state store. */
export interface CreateStateStoreOptions<State> {
  /**
   * Clock used to timestamp and expire snapshots.
   *
   * @default Date.now
   */
  readonly now?: () => number;
  /**
   * Snapshot to validate and restore before the store is returned.
   *
   * @default undefined
   */
  readonly snapshot?: StateSnapshot<StateModelKey, State>;
  /**
   * Undo/redo history configuration, or `false` to disable history.
   *
   * @default { capacity: 100 }
   */
  readonly history?: false | StateHistoryOptions;
}

/** Bounded undo/redo history configuration. */
export interface StateHistoryOptions {
  /**
   * Maximum number of past states retained by the store.
   *
   * @default 100
   */
  readonly capacity?: number;
}

/** Reducer transaction receipt that can be passed back to `rollback`. */
export interface StateTransaction<State, Action extends StateAction> {
  readonly id: number;
  readonly actions: readonly Action[];
  readonly before: State;
  readonly after: State;
}

/** Final state and disposition of an optimistic reducer update. */
export interface StateOptimisticResult<State> {
  readonly status: "committed" | "rolled-back" | "superseded";
  readonly state: State;
  readonly reason?: unknown;
}

/** Read-only undo/redo capacity and depth exposed by a store. */
export interface StateHistoryStatus {
  readonly canUndo: boolean;
  readonly canRedo: boolean;
  readonly undoDepth: number;
  readonly redoDepth: number;
  readonly capacity: number;
}

/** Runtime store returned by createStateStore. */
export interface StateStore<Model extends AnyStateModelDefinition> {
  readonly key: InferKey<Model>;
  readonly persistenceKey: string;
  readonly source: Model["source"];
  readonly version: Model["version"];
  readonly state: InferState<Model>;
  readonly history: StateHistoryStatus;
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
  readonly undo: () => InferState<Model>;
  readonly redo: () => InferState<Model>;
  readonly clearHistory: () => void;
  readonly snapshot: () => StateSnapshot<InferKey<Model>, InferState<Model>>;
}

/** Extract the caller-supplied argument tuple from a state command. */
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

/** Machine-readable ownership and persistence metadata for one state model. */
export interface StateManifestEntry {
  readonly key: StateModelKey;
  readonly source: StateSourceSpecifier;
  readonly version: number;
  readonly meta: StateModelMeta;
  readonly persistence: boolean;
}

/** Finite-state transition map used for literal transition type checks. */
export type StateTransitionTable = Readonly<Record<string, readonly string[]>>;

/** Union of states reachable from `From` in a finite transition table. */
export type AllowedTransition<
  Table extends StateTransitionTable,
  From extends keyof Table & string,
> = Table[From][number] & string;

/** Direct or functional update accepted by a controllable state holder. */
export type StateValueUpdater<State> = State | ((previous: Readonly<State>) => State);

/** Previous and proposed values supplied to a controllable-state change handler. */
export interface StateChangeContext<State> {
  readonly previous: Readonly<State>;
  readonly next: Readonly<State>;
  readonly controlled: boolean;
}

/** Observer called when a controllable state holder proposes a distinct value. */
export type StateChangeHandler<State> = (next: State, context: StateChangeContext<State>) => void;

/** Options for state whose current value is owned by its caller. */
export interface ControlledStateOptions<State> {
  readonly value: () => State;
  /**
   * Controlled state cannot own an initial value.
   *
   * @default undefined
   */
  readonly defaultValue?: never;
  /**
   * Observer for proposed value changes.
   *
   * @default undefined
   */
  readonly onChange?: StateChangeHandler<State>;
  /**
   * Equality predicate used to suppress unchanged proposals.
   *
   * @default Object.is
   */
  readonly equals?: (left: Readonly<State>, right: Readonly<State>) => boolean;
}

/** Options for state whose current value is owned by the returned holder. */
export interface UncontrolledStateOptions<State> {
  readonly defaultValue: State;
  /**
   * Uncontrolled state has no external value reader.
   *
   * @default undefined
   */
  readonly value?: never;
  /**
   * Observer called after a distinct local value is proposed.
   *
   * @default undefined
   */
  readonly onChange?: StateChangeHandler<State>;
  /**
   * Equality predicate used to suppress unchanged local updates.
   *
   * @default Object.is
   */
  readonly equals?: (left: Readonly<State>, right: Readonly<State>) => boolean;
}

/** Discriminated controlled or uncontrolled state ownership options. */
export type ControllableStateOptions<State> =
  | ControlledStateOptions<State>
  | UncontrolledStateOptions<State>;

/** Stateful facade returned by `createControllableState`. */
export interface ControllableState<State, Controlled extends boolean = boolean> {
  readonly controlled: Controlled;
  readonly value: State;
  readonly set: (next: StateValueUpdater<State>) => State;
  readonly reset: () => State;
  readonly snapshot: () => State;
}

/** Controls whether diagnostic manifests may include development metadata. */
export interface StateDiagnosticsOptions {
  /**
   * Omit model details from the diagnostics manifest.
   *
   * @default false
   */
  readonly production?: boolean;
}

/** Machine-readable development metadata for state diagnostics and devtools. */
export interface StateDiagnosticsManifest {
  readonly schemaVersion: 1;
  readonly production: boolean;
  readonly models: readonly StateDiagnosticsEntry[];
}

/** Diagnostic metadata for one state model. */
export interface StateDiagnosticsEntry {
  readonly key: StateModelKey;
  readonly source: StateSourceSpecifier;
  readonly version: number;
  readonly commands: readonly string[];
  readonly persistence: boolean;
  readonly meta: StateModelMeta;
}
