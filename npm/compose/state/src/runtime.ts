import type {
  AllowedTransition,
  AnyStateModelDefinition,
  CreateStateStoreOptions,
  InferAction,
  InferCommands,
  InferKey,
  InferState,
  StateAction,
  StateCommandMap,
  StateHydrationResult,
  StateManifest,
  StateModelDefinition,
  StatePersistenceAdapter,
  StateSnapshot,
  StateStore,
  StateTransitionTable,
} from "./types.ts";

const STATE_ERROR = {
  invalidKey: "VIZE_STATE_INVALID_KEY",
  invalidSource: "VIZE_STATE_SOURCE_NOT_VUE",
  invalidVersion: "VIZE_STATE_INVALID_VERSION",
  missingCommand: "VIZE_STATE_UNKNOWN_COMMAND",
} as const;

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
  let transactionId = 0;

  const apply = (action: InferAction<Model>) => {
    state = model.reducer(state, action) as InferState<Model>;
    return state;
  };

  const store = {
    key: model.key as InferKey<Model>,
    persistenceKey: model.persistence?.storageKey ?? model.key,
    source: model.source,
    version: model.version,
    get state() {
      return state;
    },
    dispatch(action) {
      return apply(action);
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
      for (const action of actions) apply(action as InferAction<Model>);
      return state;
    },
    transaction(actions) {
      const before = state;
      for (const action of actions) apply(action);
      return {
        id: ++transactionId,
        actions: [...actions],
        before,
        after: state,
      };
    },
    async optimistic(action, confirm) {
      const before = state;
      apply(action);
      try {
        await confirm();
        return { status: "committed", state } as const;
      } catch (reason) {
        state = before;
        return { status: "rolled-back", state, reason } as const;
      }
    },
    rollback(transaction) {
      state = transaction.before;
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

/** Emit metadata for generators, documentation, and devtools. */
export function createStateManifest<const Models extends readonly AnyStateModelDefinition[]>(
  models: Models,
): StateManifest {
  for (const model of models) validateModel(model);
  return {
    schemaVersion: 1,
    models: models.map((model) => ({
      key: model.key,
      source: model.source,
      version: model.version,
      meta: model.meta ?? {},
      persistence: model.persistence != null,
    })),
  };
}

/** Define a literal finite-state table for type-checked transitions. */
export function defineStateTransitions<const Table extends StateTransitionTable>(
  table: Table,
): Table {
  return table;
}

/** Runtime guard for finite-state transitions. */
export function canTransition<
  const Table extends StateTransitionTable,
  const From extends keyof Table & string,
>(table: Table, from: From, to: string): to is AllowedTransition<Table, From> {
  return table[from]?.includes(to) ?? false;
}

/** Assert and return a literal transition destination. */
export function transitionState<
  const Table extends StateTransitionTable,
  const From extends keyof Table & string,
  const To extends AllowedTransition<Table, From>,
>(table: Table, from: From, to: To): To {
  if (!canTransition(table, from, to)) {
    throw new Error(`Invalid state transition ${from} -> ${String(to)}`);
  }
  return to;
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

function validateModel(model: Pick<AnyStateModelDefinition, "key" | "source" | "version">): void {
  if (!model.key) throw new Error(`[${STATE_ERROR.invalidKey}] State model key is required`);
  if (!model.source.endsWith(".vue")) {
    throw new Error(`[${STATE_ERROR.invalidSource}] State model ${model.key} must use .vue`);
  }
  if (!Number.isInteger(model.version) || model.version < 1) {
    throw new Error(
      `[${STATE_ERROR.invalidVersion}] State model ${model.key} version must be >= 1`,
    );
  }
}
