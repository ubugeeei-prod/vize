import type { ShallowRef } from "vue";

/** Stable identifier of one toast inside a store. */
export type ToastId = string;

/** Visual and semantic variant of a toast. */
export type ToastType = "default" | "error" | "info" | "loading" | "success" | "warning";

/**
 * Scheduling priority. `high` jumps the pending queue and is announced
 * assertively; `low` waits behind every other pending toast.
 */
export type ToastPriority = "high" | "low" | "normal";

/** Open state mirrored to the Toast data contract. */
export type ToastState = "closed" | "open";

/** Why a toast left the open state. */
export type ToastDismissReason = "action" | "api" | "close" | "escape" | "swipe" | "timeout";

/** Reason the auto-dismiss timers are paused. Several reasons can overlap. */
export type ToastPauseReason = "focus" | "hidden" | "hover" | "manual" | "swipe";

/** Direction a toast is swiped to dismiss it. */
export type ToastSwipeDirection = "down" | "left" | "right" | "up";

/** Swipe gesture phase published on `data-swipe`. */
export type ToastSwipeState = "cancel" | "end" | "move" | "start";

/** Optional call to action rendered with a toast. */
export interface ToastActionOptions {
  /** Visible action label. */
  readonly label: string;

  /**
   * Screen-reader alternative describing how to reach the action without the
   * toast, because toasts may disappear before a keyboard user reaches them.
   */
  readonly altText: string;

  /**
   * Called when the action is activated. Call `event.preventDefault()` to keep
   * the toast open.
   *
   * @default undefined
   */
  readonly onClick?: (event: MouseEvent) => void;
}

/** Options accepted when creating or updating a toast. */
export interface ToastOptions<Data = unknown> {
  /**
   * Explicit id. Creating a toast with an id that is still in the store
   * replaces that toast in place instead of stacking a duplicate.
   *
   * @default A store-generated deterministic id
   */
  readonly id?: ToastId;

  /**
   * Short headline.
   *
   * @default undefined
   */
  readonly title?: string;

  /**
   * Supporting sentence.
   *
   * @default undefined
   */
  readonly description?: string;

  /**
   * Semantic variant.
   *
   * @default "default"
   */
  readonly type?: ToastType;

  /**
   * Scheduling and announcement priority.
   *
   * @default "normal"
   */
  readonly priority?: ToastPriority;

  /**
   * Milliseconds the toast stays open while visible and unpaused. `Infinity`
   * keeps it open until dismissed. Loading toasts default to `Infinity`.
   *
   * @default The store duration
   */
  readonly duration?: number;

  /**
   * Whether close buttons, Escape, and swipe may dismiss the toast.
   *
   * @default true
   */
  readonly dismissible?: boolean;

  /**
   * Optional call to action.
   *
   * @default undefined
   */
  readonly action?: ToastActionOptions;

  /**
   * Consumer payload for custom renderers.
   *
   * @default undefined
   */
  readonly data?: Data;

  /**
   * Called once when the toast is dismissed for any reason.
   *
   * @default undefined
   */
  readonly onDismiss?: (toast: ToastRecord<Data>, reason: ToastDismissReason) => void;

  /**
   * Called once when the toast closes because its duration elapsed.
   *
   * @default undefined
   */
  readonly onAutoClose?: (toast: ToastRecord<Data>) => void;
}

/** Options accepted by the typed variant helpers such as `success()`. */
export type ToastVariantOptions<Data = unknown> = Omit<ToastOptions<Data>, "type">;

/** Immutable snapshot of one toast. */
export interface ToastRecord<Data = unknown> {
  /** Store-unique id. */
  readonly id: ToastId;

  /** Short headline. */
  readonly title: string | undefined;

  /** Supporting sentence. */
  readonly description: string | undefined;

  /** Semantic variant. */
  readonly type: ToastType;

  /** Scheduling and announcement priority. */
  readonly priority: ToastPriority;

  /** Resolved auto-dismiss duration in milliseconds, possibly `Infinity`. */
  readonly duration: number;

  /** Whether user dismissal is allowed. */
  readonly dismissible: boolean;

  /** Optional call to action. */
  readonly action: ToastActionOptions | undefined;

  /** Consumer payload. */
  readonly data: Data | undefined;

  /** Whether the toast is open. Closed toasts stay until their exit completes. */
  readonly open: boolean;

  /** Stable token for styling and tests. */
  readonly state: ToastState;

  /** Why the toast closed, once it has. */
  readonly dismissReason: ToastDismissReason | null;

  /** Monotonic creation order inside the store. */
  readonly sequence: number;

  /** Incremented on every content update so announcers can re-announce. */
  readonly revision: number;
}

/** Messages for one promise toast phase: text, options, or a factory. */
export type ToastPromiseMessage<Input, Data = unknown> =
  | string
  | ToastVariantOptions<Data>
  | ((input: Input) => string | ToastVariantOptions<Data>);

/** Phase messages accepted by {@link ToastStore.promise}. */
export interface ToastPromiseOptions<Value, Data = unknown> {
  /**
   * Explicit toast id reused across the loading, success, and error phases.
   *
   * @default A store-generated id
   */
  readonly id?: ToastId;

  /** Loading phase text or options. */
  readonly loading: string | ToastVariantOptions<Data>;

  /** Success phase, optionally derived from the resolved value. */
  readonly success: ToastPromiseMessage<Value, Data>;

  /** Error phase, optionally derived from the rejection reason. */
  readonly error: ToastPromiseMessage<unknown, Data>;
}

/** Options for {@link createToastStore}. */
export interface ToastStoreOptions {
  /**
   * Default auto-dismiss duration in milliseconds.
   *
   * @default 5000
   */
  readonly duration?: number;

  /**
   * Maximum number of simultaneously visible toasts. Extra toasts wait in a queue.
   *
   * @default 3
   */
  readonly limit?: number;

  /**
   * Prefix used for generated ids.
   *
   * @default "toast"
   */
  readonly idPrefix?: string;
}

/** Typed imperative toast queue. */
export interface ToastStore<Data = unknown> {
  /** Every toast in creation order, including queued and closing toasts. */
  readonly toasts: Readonly<ShallowRef<readonly ToastRecord<Data>[]>>;

  /** Toasts currently rendered, in the order they were shown. */
  readonly visibleToasts: Readonly<ShallowRef<readonly ToastRecord<Data>[]>>;

  /** Open toasts waiting for a visible slot, in the order they will be shown. */
  readonly queuedToasts: Readonly<ShallowRef<readonly ToastRecord<Data>[]>>;

  /** Whether any pause reason is active. */
  readonly paused: Readonly<ShallowRef<boolean>>;

  /** Current store-level default duration. */
  readonly duration: Readonly<ShallowRef<number>>;

  /** Current visible limit. */
  readonly limit: Readonly<ShallowRef<number>>;

  /** Create a toast from a title or options and return its id. */
  readonly toast: (input: string | ToastOptions<Data>) => ToastId;

  /** Create a `success` toast. */
  readonly success: (input: string | ToastVariantOptions<Data>) => ToastId;

  /** Create an `error` toast. */
  readonly error: (input: string | ToastVariantOptions<Data>) => ToastId;

  /** Create a `warning` toast. */
  readonly warning: (input: string | ToastVariantOptions<Data>) => ToastId;

  /** Create an `info` toast. */
  readonly info: (input: string | ToastVariantOptions<Data>) => ToastId;

  /** Create a persistent `loading` toast. */
  readonly loading: (input: string | ToastVariantOptions<Data>) => ToastId;

  /**
   * Show a loading toast that becomes a success or error toast when the
   * promise settles. Returns the original promise unchanged.
   */
  readonly promise: <Value>(
    promise: Promise<Value>,
    options: ToastPromiseOptions<Value, Data>,
  ) => Promise<Value>;

  /** Patch an existing toast. Returns `false` when the id is unknown. */
  readonly update: (id: ToastId, patch: Omit<ToastOptions<Data>, "id">) => boolean;

  /**
   * Close one toast, or every toast when `id` is omitted. Closed toasts stay
   * in `visibleToasts` until {@link ToastStore.remove} runs after exit motion.
   */
  readonly dismiss: (id?: ToastId, reason?: ToastDismissReason) => boolean;

  /** Remove a toast immediately, skipping exit motion. */
  readonly remove: (id: ToastId) => boolean;

  /** Remove every toast immediately. */
  readonly clear: () => void;

  /** Read one toast snapshot. */
  readonly get: (id: ToastId) => ToastRecord<Data> | undefined;

  /** Pause auto-dismiss timers for a reason. */
  readonly pause: (reason?: ToastPauseReason) => void;

  /** Release one pause reason; timers resume with their remaining time. */
  readonly resume: (reason?: ToastPauseReason) => void;

  /** Update the default duration and visible limit. */
  readonly configure: (options: Omit<ToastStoreOptions, "idPrefix">) => void;

  /** Begin running timers. Providers call this after mount; SSR never does. */
  readonly start: () => void;

  /** Stop every timer, preserving remaining time. */
  readonly stop: () => void;
}

/** Slot state exposed by ToastViewport for each rendered toast. */
export interface ToastViewportSlotState<Data = unknown> {
  /** Toast snapshot to render. */
  readonly toast: ToastRecord<Data>;

  /** Zero-based position among visible toasts. */
  readonly index: number;

  /** Number of visible toasts. */
  readonly count: number;

  /** Close this toast. */
  readonly dismiss: () => boolean;
}

/** Slot state exposed by ToastRoot and its parts. */
export interface ToastSlotState<Data = unknown> {
  /** Toast snapshot. */
  readonly toast: ToastRecord<Data>;

  /** Stable open token. */
  readonly state: ToastState;

  /** Current swipe phase, or `null` when idle. */
  readonly swipe: ToastSwipeState | null;
}

/** Public instance exposed by ToastProvider. */
export interface ToastProviderExpose<Data = unknown> {
  /** Store owned or forwarded by the provider. */
  readonly store: ToastStore<Data>;
}

/** Public instance exposed by ToastViewport. */
export interface ToastViewportExpose {
  /** Rendered region element. */
  readonly element: HTMLElement | null;

  /** Move focus to the region, as the hotkey does. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by ToastRoot. */
export interface ToastRootExpose {
  /** Rendered list item. */
  readonly element: HTMLLIElement | null;

  /** Close this toast. */
  readonly dismiss: (reason?: ToastDismissReason) => boolean;
}

/** Public instance exposed by ToastAction and ToastClose. */
export interface ToastButtonExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the button. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by ToastTitle and ToastDescription. */
export interface ToastTextExpose {
  /** Rendered element. */
  readonly element: HTMLElement | null;
}
