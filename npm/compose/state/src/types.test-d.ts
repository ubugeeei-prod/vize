import {
  createControllableState,
  createStateDiagnosticsManifest,
  createStateStore,
  defineStateModel,
  defineStateTransitions,
  transitionState,
  type InferAction,
  type InferKey,
  type InferState,
} from "./index.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const formMachine = defineStateTransitions({
  idle: ["editing"],
  editing: ["saving", "idle"],
  saving: ["saved", "failed"],
  saved: ["editing"],
  failed: ["editing"],
} as const);

transitionState(formMachine, "idle", "editing") satisfies "editing";
// @ts-expect-error idle cannot jump directly to saved.
transitionState(formMachine, "idle", "saved");

type CounterState = { readonly count: number; readonly pending: boolean };
type CounterAction =
  | { readonly type: "increment"; readonly by: number }
  | { readonly type: "pending"; readonly value: boolean };

const counter = defineStateModel({
  key: "counter.panel",
  source: "../fixtures/CounterPanel.vue",
  version: 1,
  initialState: { count: 0, pending: false } as CounterState,
  reducer(state, action: CounterAction): CounterState {
    switch (action.type) {
      case "increment":
        return { ...state, count: state.count + action.by };
      case "pending":
        return { ...state, pending: action.value };
    }
  },
  commands: {
    incrementBy(_context, by: number) {
      return { type: "increment", by } as const;
    },
  },
} as const);

type _StateInference = Expect<Equal<InferState<typeof counter>, CounterState>>;
type _ActionInference = Expect<Equal<InferAction<typeof counter>, CounterAction>>;
type _KeyInference = Expect<Equal<InferKey<typeof counter>, "counter.panel">>;

const store = createStateStore(counter);
store.dispatch({ type: "increment", by: 2 }) satisfies CounterState;
store.command("incrementBy", 3) satisfies CounterState;
store.state.count satisfies number;
store.undo() satisfies CounterState;
store.redo() satisfies CounterState;
store.history.canUndo satisfies boolean;

// @ts-expect-error action payload is exact.
store.dispatch({ type: "increment" });
// @ts-expect-error unknown actions are rejected.
store.dispatch({ type: "remove", id: "x" });
// @ts-expect-error commands keep argument tuples.
store.command("incrementBy", "3");

const controlled = createControllableState({
  value: () => ({ count: 1 }),
  onChange(next) {
    next.count satisfies number;
  },
});
controlled.controlled satisfies true;
controlled.value.count satisfies number;

const uncontrolled = createControllableState({
  defaultValue: { count: 0 },
});
uncontrolled.controlled satisfies false;
uncontrolled.set((previous) => ({ count: previous.count + 1 })) satisfies { count: number };

// @ts-expect-error controlled state cannot also declare a default value.
createControllableState({ value: () => ({ count: 1 }), defaultValue: { count: 0 } });

const diagnostics = createStateDiagnosticsManifest([counter], { production: false });
diagnostics.models[0]?.commands satisfies readonly string[] | undefined;

defineStateModel({
  key: "broken",
  // @ts-expect-error source ownership is Vue-only.
  source: "./broken.ts",
  version: 1,
  initialState: {},
  reducer: (s) => s,
});
