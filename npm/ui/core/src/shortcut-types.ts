import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Modifier layout family used to resolve `Mod` and to format keycaps. */
export type ShortcutPlatform = "apple" | "standard";

/** One normalized simultaneous key press within a shortcut. */
export interface ShortcutChord {
  /** Canonical `KeyboardEvent.key` value; single characters are lowercase. */
  readonly key: string;

  /** Alt (Option) must be held. */
  readonly altKey: boolean;

  /** Control must be held. */
  readonly ctrlKey: boolean;

  /** Meta (Command / Windows) must be held. */
  readonly metaKey: boolean;

  /** Shift must be held. */
  readonly shiftKey: boolean;
}

/** Ordered chord steps typed one after another, e.g. `G` then `D`. */
export type ShortcutSequence = readonly ShortcutChord[];

/** Options for `parseShortcut` and the keycap formatting helpers. */
export interface ShortcutParseOptions {
  /**
   * Layout used to resolve the `Mod` platform modifier: Meta on `apple`,
   * Control on `standard`.
   *
   * @default detectShortcutPlatform()
   */
  readonly platform?: ShortcutPlatform;
}

/** Presentation options for `formatShortcut` and `getShortcutKeycaps`. */
export interface ShortcutFormatOptions extends ShortcutParseOptions {
  /**
   * `symbol` renders platform glyphs such as `⌘` and `⇧`; `text` renders
   * spelled-out names such as `Ctrl` and `Shift`.
   *
   * @default "symbol" on apple, "text" on standard
   */
  readonly style?: "symbol" | "text";
}

/** Immutable description of one dispatched shortcut. */
export interface ShortcutMatch {
  /** Normalized sequence that completed. */
  readonly shortcut: ShortcutSequence;

  /** Scope that owned the winning binding. */
  readonly scope: string;

  /** Help description supplied at registration, or `null`. */
  readonly description: string | null;

  /** Native event that completed the sequence. */
  readonly originalEvent: KeyboardEvent;
}

/** One registered binding, as surfaced by help and conflict metadata. */
export interface ShortcutBindingInfo {
  /** Normalized sequence this binding listens for. */
  readonly shortcut: ShortcutSequence;

  /** Scope name the binding belongs to. */
  readonly scope: string;

  /** Help description supplied at registration, or `null`. */
  readonly description: string | null;
}

/** Bindings in one scope that listen for the same normalized sequence. */
export interface ShortcutConflict {
  /** Scope in which the collision occurs. */
  readonly scope: string;

  /** Normalized sequence claimed more than once. */
  readonly shortcut: ShortcutSequence;

  /** Every binding claiming the sequence, in registration order. */
  readonly bindings: readonly ShortcutBindingInfo[];
}

/** Options accepted by `ShortcutRegistry.register`. */
export interface ShortcutBindingOptions {
  /** Pattern such as `"Mod+K"` or `"G D"`, or a pre-parsed sequence. */
  readonly shortcut: string | ShortcutSequence;

  /** Called when the sequence completes and this binding wins routing. */
  readonly handler: (match: ShortcutMatch) => void;

  /**
   * Scope owning the binding. Non-global scopes only receive input while
   * activated, and later-activated scopes shadow earlier ones.
   *
   * @default "global"
   */
  readonly scope?: string;

  /**
   * Additional enablement gate checked on every dispatch.
   *
   * @default true
   */
  readonly when?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Cancel the native action when the binding matches or extends a pending
   * sequence.
   *
   * @default true
   */
  readonly preventDefault?: boolean;

  /**
   * Accept auto-repeated keydown events.
   *
   * @default false
   */
  readonly allowRepeat?: boolean;

  /**
   * Dispatch even when the event originates from text-editing elements.
   *
   * @default false
   */
  readonly allowInEditable?: boolean;

  /** Help description surfaced through binding and conflict metadata. */
  readonly description?: string;
}

/** Stable keyboard handler to spread onto an element-scoped shortcut host. */
export interface ShortcutProps {
  readonly onKeydown: (event: KeyboardEvent) => void;
}

/** Options for `createShortcutRegistry` and `useShortcutRegistry`. */
export interface ShortcutRegistryOptions {
  /**
   * Event target that feeds the registry: a document, window, shadow root, or
   * element. `undefined` resolves the ambient document on the client and
   * nothing during server rendering; `null` disables automatic attachment.
   *
   * @default globalThis.document ?? null
   */
  readonly target?: MaybeRefOrGetter<EventTarget | null | undefined>;

  /**
   * Layout used to resolve the `Mod` platform modifier.
   *
   * @default detectShortcutPlatform()
   */
  readonly platform?: ShortcutPlatform;

  /**
   * Idle milliseconds before a pending multi-chord sequence resets.
   *
   * @default 1000
   */
  readonly sequenceTimeout?: MaybeRefOrGetter<number | undefined>;

  /**
   * Ignore input and clear any pending sequence while true.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Attach native listeners during the capture phase.
   *
   * @default false
   */
  readonly capture?: boolean;
}

/** Scoped, sequence-aware keyboard shortcut registry. */
export interface ShortcutRegistry {
  /** Chords accepted so far by a pending multi-chord sequence. */
  readonly pendingSequence: Readonly<ShallowRef<ShortcutSequence>>;

  /** Non-global scopes currently activated, in activation order. */
  readonly activeScopes: Readonly<ShallowRef<readonly string[]>>;

  /** Stable keyboard handler for declarative element-scoped hosts. */
  readonly shortcutProps: Readonly<ShortcutProps>;

  /** Add one binding and return its releaser. */
  readonly register: (binding: ShortcutBindingOptions) => () => void;

  /** Activate a scope on top of the stack and return its releaser. */
  readonly activateScope: (scope: string) => () => void;

  /** Route one keyboard event and report whether a binding completed. */
  readonly input: (event: KeyboardEvent) => boolean;

  /** Attach native listeners to an extra target and return the detacher. */
  readonly attach: (target: EventTarget, options?: { readonly capture?: boolean }) => () => void;

  /** Clear a pending sequence and report whether one was pending. */
  readonly reset: () => boolean;

  /** Snapshot every registered binding in registration order. */
  readonly getBindings: () => readonly ShortcutBindingInfo[];

  /** Report same-scope bindings that claim an identical sequence. */
  readonly getConflicts: () => readonly ShortcutConflict[];

  /** Detach listeners, clear timers, and make imperative calls terminal. */
  readonly dispose: () => void;
}
