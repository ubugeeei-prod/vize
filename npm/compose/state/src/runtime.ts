import type {
  AnyStateModelDefinition,
  CreateStateStoreOptions,
  InferAction,
  InferCommands,
  InferKey,
  InferState,
  StateAction,
  StateCommandMap,
  StateHistoryOptions,
  StateHydrationResult,
  StateModelDefinition,
  StatePersistenceAdapter,
  StateSnapshot,
  StateStore,
} from "./types.ts";
import { STATE_ERROR, validateModel } from "./validation.ts";

const DEFAULT_HISTORY_CAPACITY = 100;

/** Define a typed reducer store and preserve literal model metadata. */
export function defineStateModel<
  const Key extends string,
  State,
  Action extends StateAction,
  const Commands extends StateCommandMap<State, Action> = StateCommandMap<State, Action>,
  const Meta extends Readonly<Record<string, unknown>> = Readonly<Record<string, unknown>>,
>(
  model: StateModelDefinition<Key, State, Action, Commands, Meta>,
): StateModelDefinition<Key, State, Action, Commands, Meta> {
  validateModel(model);
  return model;
}

/** Create an SSR-safe state store with no request-global state. */
export function createStateStore<const Model extends AnyStateModelDefinition>(
  model: Model,
  options: CreateStateStoreOptions<InferState<Model>> = {},
): StateStore<Model> {
  validateModel(model);
  const now = options.now ?? Date.now;
  let state = restoreState(model, options.snapshot, now).state;
  let past: InferState<Model>[] = [];
  let future: InferState<Model>[] = [];
  const historyCapacity = resolveHistoryCapacity(options.history);
  let transactionId = 0;
  let revision = 0;

  const remember = (before: InferState<Model>, after: InferState<Model>) => {
    if (historyCapacity === 0 || Object.is(before, after)) return;
    past.push(before);
    if (past.length > historyCapacity) past = past.slice(past.length - historyCapacity);
    future = [];
  };

  const commit = (next: InferState<Model>, recordHistory = true) => {
    const before = state;
    state = next;
    if (recordHistory) remember(before, state);
    if (!Object.is(before, state)) revision += 1;
    return state;
  };

  const reduce = (current: InferState<Model>, action: InferAction<Model>) =>
    model.reducer(current, action) as InferState<Model>;

  const reduceAll = (
    current: InferState<Model>,
    actions: readonly InferAction<Model>[],
  ): InferState<Model> => {
    let next = current;
    for (const action of actions) next = reduce(next, action);
    return next;
  };

  const store = {
    key: model.key as InferKey<Model>,
    persistenceKey: model.persistence?.storageKey ?? model.key,
    source: model.source,
    version: model.version,
    get state() {
      return state;
    },
    get history() {
      return {
        canUndo: past.length > 0,
        canRedo: future.length > 0,
        undoDepth: past.length,
        redoDepth: future.length,
        capacity: historyCapacity,
      };
    },
    dispatch(action) {
      return commit(reduce(state, action));
    },
    command(name, ...args) {
      const command = (model.commands as InferCommands<Model> | undefined)?.[name];
      if (command == null) {
        throw new Error(`[${STATE_ERROR.missingCommand}] Unknown state command ${name}`);
      }
      const runCommand = command as unknown as (
        context: { readonly state: InferState<Model> },
        ...arguments_: unknown[]
      ) => InferAction<Model> | readonly InferAction<Model>[];
      const output = runCommand({ state }, ...args);
      const actions = Array.isArray(output) ? output : [output];
      return commit(reduceAll(state, actions as readonly InferAction<Model>[]));
    },
    transaction(actions) {
      const before = state;
      commit(reduceAll(state, actions));
      return {
        id: ++transactionId,
        actions: [...actions],
        before,
        after: state,
      };
    },
    async optimistic(action, confirm) {
      const before = state;
      const beforeRevision = revision;
      const previousPast = past.slice();
      const previousFuture = future.slice();
      commit(reduce(state, action));
      const optimisticRevision = revision;
      try {
        await confirm();
        return { status: "committed", state } as const;
      } catch (reason) {
        if (revision !== optimisticRevision) {
          return { status: "superseded", state, reason } as const;
        }
        state = before;
        past = previousPast;
        future = previousFuture;
        revision = beforeRevision;
        return { status: "rolled-back", state, reason } as const;
      }
    },
    rollback(transaction) {
      const current = state;
      if (Object.is(state, transaction.after) && past[past.length - 1] === transaction.before) {
        past = past.slice(0, -1);
      }
      state = transaction.before;
      future = [];
      if (!Object.is(current, state)) revision += 1;
    },
    undo() {
      if (past.length === 0) return state;
      const previous = past[past.length - 1] as InferState<Model>;
      past = past.slice(0, -1);
      future = [state, ...future];
      state = previous;
      revision += 1;
      return state;
    },
    redo() {
      if (future.length === 0) return state;
      const next = future[0] as InferState<Model>;
      future = future.slice(1);
      if (historyCapacity > 0) {
        past.push(state);
        if (past.length > historyCapacity) past = past.slice(past.length - historyCapacity);
      }
      state = next;
      revision += 1;
      return state;
    },
    clearHistory() {
      past = [];
      future = [];
    },
    snapshot(): StateSnapshot<InferKey<Model>, InferState<Model>> {
      return {
        schemaVersion: 1,
        key: model.key as InferKey<Model>,
        source: model.source,
        version: model.version,
        state,
        updatedAt: now(),
      };
    },
  } satisfies StateStore<Model>;

  return store;
}

/** Serialize the current store state into an SSR or persistence payload. */
export function serializeStateSnapshot<const Model extends AnyStateModelDefinition>(
  store: Pick<StateStore<Model>, "snapshot">,
): StateSnapshot<InferKey<Model>, InferState<Model>> {
  return store.snapshot();
}

/** Hydrate a store from a server-rendered payload. */
export function hydrateStateStore<const Model extends AnyStateModelDefinition>(
  model: Model,
  snapshot: StateSnapshot,
  options: Omit<CreateStateStoreOptions<InferState<Model>>, "snapshot"> = {},
): StateStore<Model> {
  return createStateStore(model, { ...options, snapshot: snapshot as never });
}

/** Persist a store snapshot through a synchronous adapter. */
export function persistStateSnapshot<const Model extends AnyStateModelDefinition>(
  store: Pick<StateStore<Model>, "persistenceKey" | "snapshot">,
  adapter: StatePersistenceAdapter,
): StateSnapshot<InferKey<Model>, InferState<Model>> {
  const snapshot = serializeStateSnapshot(store);
  adapter.setItem(store.persistenceKey, JSON.stringify(snapshot));
  return snapshot;
}

/** Restore persisted state, validating expiry, model ownership, and migration. */
export function restorePersistedState<const Model extends AnyStateModelDefinition>(
  model: Model,
  adapter: StatePersistenceAdapter,
  options: Omit<CreateStateStoreOptions<InferState<Model>>, "snapshot"> = {},
): StateHydrationResult<InferState<Model>> {
  validateModel(model);
  const storageKey = model.persistence?.storageKey ?? model.key;
  const raw = adapter.getItem(storageKey);
  if (raw == null)
    return { status: "missing", state: model.initialState as InferState<Model>, recovered: true };

  let parsed: StateSnapshot | undefined;
  try {
    parsed = JSON.parse(raw) as StateSnapshot;
  } catch {
    adapter.removeItem(storageKey);
    return { status: "invalid", state: model.initialState as InferState<Model>, recovered: true };
  }

  const result = restoreState(model, parsed, options.now ?? Date.now);
  if (result.recovered) adapter.removeItem(storageKey);
  return result;
}

/** Create an in-memory persistence adapter for SSR tests and deterministic fixtures. */
export function createMemoryStatePersistence(
  seed: Readonly<Record<string, string>> = {},
): StatePersistenceAdapter {
  const entries = new Map(Object.entries(seed));
  return {
    getItem: (key) => entries.get(key),
    setItem: (key, value) => {
      entries.set(key, value);
    },
    removeItem: (key) => {
      entries.delete(key);
    },
  };
}

function restoreState<const Model extends AnyStateModelDefinition>(
  model: Model,
  snapshot: StateSnapshot | undefined,
  now: () => number,
): StateHydrationResult<InferState<Model>> {
  if (snapshot == null) {
    return { status: "missing", state: model.initialState as InferState<Model>, recovered: true };
  }
  if (
    snapshot.schemaVersion !== 1 ||
    snapshot.key !== model.key ||
    snapshot.source !== model.source
  ) {
    return { status: "mismatch", state: model.initialState as InferState<Model>, recovered: true };
  }

  const ttlMs = model.persistence?.ttlMs;
  if (ttlMs != null && now() - snapshot.updatedAt > ttlMs) {
    return { status: "expired", state: model.initialState as InferState<Model>, recovered: true };
  }

  let restored: unknown = snapshot.state;
  let status: StateHydrationResult<InferState<Model>>["status"] = "accepted";
  if (snapshot.version !== model.version) {
    const migrate = model.persistence?.migrate;
    if (!migrate) {
      return {
        status: "mismatch",
        state: model.initialState as InferState<Model>,
        recovered: true,
      };
    }
    restored = migrate(snapshot.state, {
      fromVersion: snapshot.version,
      toVersion: model.version,
      key: model.key,
    });
    status = "migrated";
  }

  const validate = model.persistence?.validate;
  if (validate && !validate(restored)) {
    return { status: "invalid", state: model.initialState as InferState<Model>, recovered: true };
  }

  return { status, state: restored as InferState<Model>, recovered: false };
}

function resolveHistoryCapacity(options: false | StateHistoryOptions | undefined): number {
  if (options === false) return 0;
  const capacity = options?.capacity ?? DEFAULT_HISTORY_CAPACITY;
  if (!Number.isFinite(capacity) || capacity < 0) return DEFAULT_HISTORY_CAPACITY;
  return Math.trunc(capacity);
}
