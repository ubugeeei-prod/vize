import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Context handed to a command implementation when it runs. */
export interface CommandExecution<Id extends string = string> {
  /** Identifier the dispatch was routed to. */
  readonly id: Id;

  /** Caller-supplied argument, or `undefined` for argument-free dispatches. */
  readonly payload: unknown;

  /** How the dispatch was initiated, for analytics and help surfaces. */
  readonly source: CommandDispatchSource;
}

/** Origin of one dispatch, threaded through to handlers and observers. */
export type CommandDispatchSource = "imperative" | "menu" | "palette" | "shortcut";

/** One registerable command definition. */
export interface CommandDefinition<Id extends string = string> {
  /** Unique identifier; registering a duplicate identifier is a conflict. */
  readonly id: Id;

  /** Perform the command. The return value is surfaced on the dispatch. */
  readonly run: (execution: CommandExecution<Id>) => unknown;

  /**
   * Enablement gate; a disabled command stays listed but refuses to run.
   *
   * @default true
   */
  readonly when?: MaybeRefOrGetter<boolean | undefined>;

  /** Human-readable name for menus and palettes. */
  readonly title?: string;

  /** Longer help text for palettes and documentation surfaces. */
  readonly description?: string;

  /** Extra search terms recognized by palettes. */
  readonly keywords?: readonly string[];

  /** Grouping label used to cluster related commands in help surfaces. */
  readonly group?: string;
}

/** Immutable help metadata describing one registered command. */
export interface CommandInfo<Id extends string = string> {
  /** Unique identifier. */
  readonly id: Id;

  /** Human-readable name, or `null` when the command is anonymous. */
  readonly title: string | null;

  /** Longer help text, or `null`. */
  readonly description: string | null;

  /** Extra palette search terms. */
  readonly keywords: readonly string[];

  /** Grouping label, or `null`. */
  readonly group: string | null;

  /** Read the current enablement without dispatching. */
  readonly isEnabled: () => boolean;
}

/** Immutable outcome of one routed dispatch. */
export interface CommandDispatch<Id extends string = string> {
  /** Identifier the dispatch targeted. */
  readonly id: Id;

  /** Whether the command ran, was disabled, or was not registered. */
  readonly status: "executed" | "disabled" | "not-found";

  /** Value returned by the command, or `undefined` when it did not run. */
  readonly value: unknown;

  /** Origin threaded through from the dispatch call. */
  readonly source: CommandDispatchSource;
}

/** Options for `createCommandRouter` and `useCommandRouter`. */
export interface CommandRouterOptions<Id extends string = string> {
  /**
   * Ignore dispatches while true; every command reads as disabled.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /** Observer called after every dispatch that found its command. */
  readonly onDidExecute?: (dispatch: CommandDispatch<Id>) => void;
}

/** Typed command registry and dispatcher. */
export interface CommandRouter<Id extends string = string> {
  /** Reactive help metadata for every registered command, in registration order. */
  readonly commands: Readonly<ShallowRef<readonly CommandInfo<Id>[]>>;

  /** Register one command and return its releaser. */
  readonly register: (command: CommandDefinition<Id>) => () => void;

  /** Report whether an identifier is currently registered. */
  readonly has: (id: Id) => boolean;

  /** Read the current enablement of one identifier. */
  readonly isEnabled: (id: Id) => boolean;

  /** Route one dispatch and report the routing outcome. */
  readonly execute: (
    id: Id,
    payload?: unknown,
    options?: { readonly source?: CommandDispatchSource },
  ) => CommandDispatch<Id>;

  /** Clear every registration and make imperative calls terminal. */
  readonly dispose: () => void;
}
