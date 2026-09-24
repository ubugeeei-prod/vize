import { computed, shallowRef } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

/** Guarded, effectful transition to `target`. */
export interface MachineTransition<State extends string, Context> {
  /** State entered when the transition is taken. */
  readonly target: State;

  /** Take the transition only when this returns `true`. */
  readonly guard?: (context: Context) => boolean;

  /** Produce the next context while transitioning. */
  readonly action?: (context: Context) => Context;
}

/** One state of a {@link MachineConfig}. */
export interface MachineStateNode<State extends string, Event extends string, Context> {
  /**
   * Transitions keyed by event: a target state name, one transition object,
   * or a list of guarded transitions where the first passing guard wins.
   */
  readonly on?: {
    readonly [Name in Event]?:
      | State
      | MachineTransition<State, Context>
      | readonly MachineTransition<State, Context>[];
  };

  /** Called with the context after the machine enters this state. */
  readonly entry?: (context: Context) => void;

  /** Called with the context before the machine leaves this state. */
  readonly exit?: (context: Context) => void;
}

/** Definition consumed by {@link useMachine}. */
export interface MachineConfig<State extends string, Event extends string, Context> {
  /** State the machine starts (and resets) in. */
  readonly initial: NoInfer<State>;

  /** Initial extended state. */
  readonly context: Context;

  /** Every state, keyed by name. */
  readonly states: { readonly [Name in State]: MachineStateNode<NoInfer<State>, Event, Context> };
}

/** Reactive machine returned by {@link useMachine}. */
export interface Machine<State extends string, Event extends string, Context> {
  /** Current state name. */
  readonly state: Readonly<ShallowRef<State>>;

  /** Current extended state. */
  readonly context: Readonly<ShallowRef<Context>>;

  /** Events that would cause a transition from the current state right now. */
  readonly nextEvents: ComputedRef<Event[]>;

  /**
   * Deliver an event.
   *
   * @returns Whether a transition was taken.
   */
  readonly send: (event: Event) => boolean;

  /** Whether `event` would cause a transition right now (guards included). */
  readonly can: (event: Event) => boolean;

  /** Whether the current state is one of `states`. */
  readonly matches: (...states: readonly State[]) => boolean;

  /** Return to the initial state and context without running hooks. */
  readonly reset: () => void;
}

/** Shallow ref typed through an unconstrained parameter, so literal unions stay intact. */
function ownedRef<Value>(value: Value): ShallowRef<Value> {
  return shallowRef(value);
}

/**
 * Finite state machine with typed states, events, and targets.
 *
 * State names are inferred from the keys of `states`, event names from the
 * keys of every `on` map, and every `target` (and `initial`) must name a
 * declared state, so typos fail to compile. Transitions may carry guards
 * (first passing guard wins) and actions that return the next context;
 * `exit` runs on the old state before `entry` runs on the new one, and a
 * self-transition runs both. Synchronous and free of globals: identical on
 * server and client, nothing to dispose.
 *
 * @example
 * ```ts
 * const fetcher = useMachine({
 *   initial: "idle",
 *   context: { retries: 0 },
 *   states: {
 *     idle: { on: { FETCH: "loading" } },
 *     loading: { on: { RESOLVE: "success", REJECT: "failure" } },
 *     failure: {
 *       on: {
 *         RETRY: { target: "loading", guard: (c) => c.retries < 3, action: (c) => ({ retries: c.retries + 1 }) },
 *       },
 *     },
 *     success: {},
 *   },
 * });
 * fetcher.send("FETCH");
 * ```
 *
 * @param config States, events, context, and hooks.
 * @returns The reactive machine.
 */
export function useMachine<State extends string, Event extends string = never, Context = undefined>(
  config: MachineConfig<State, Event, Context>,
): Machine<State, Event, Context> {
  const state = ownedRef<State>(config.initial);
  const context = ownedRef<Context>(config.context);

  const isTarget = (
    option:
      | State
      | MachineTransition<State, Context>
      | readonly MachineTransition<State, Context>[],
  ): option is State => typeof option === "string";

  const resolve = (event: Event): MachineTransition<State, Context> | undefined => {
    const node: MachineStateNode<State, Event, Context> = config.states[state.value];
    const option = node.on?.[event];
    if (option === undefined) return undefined;
    if (isTarget(option)) return { target: option };
    const candidates: readonly MachineTransition<State, Context>[] = Array.isArray(option)
      ? option
      : [option];
    return candidates.find((candidate) => candidate.guard?.(context.value) ?? true);
  };

  const send = (event: Event): boolean => {
    const transition = resolve(event);
    if (transition === undefined) return false;
    const leaving: MachineStateNode<State, Event, Context> = config.states[state.value];
    leaving.exit?.(context.value);
    if (transition.action !== undefined) context.value = transition.action(context.value);
    state.value = transition.target;
    const entering: MachineStateNode<State, Event, Context> = config.states[transition.target];
    entering.entry?.(context.value);
    return true;
  };

  const can = (event: Event): boolean => resolve(event) !== undefined;

  const nextEvents = computed(() => {
    const node: MachineStateNode<State, Event, Context> = config.states[state.value];
    const on = node.on ?? {};
    const isEvent = (name: string): name is Event => name in on;
    return Object.keys(on).filter(isEvent).filter(can);
  });

  return {
    state,
    context,
    nextEvents,
    send,
    can,
    matches: (...states) => states.some((candidate) => candidate === state.value),
    reset: () => {
      state.value = config.initial;
      context.value = config.context;
    },
  };
}
