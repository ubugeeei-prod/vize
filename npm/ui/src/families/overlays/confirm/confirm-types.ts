import type { ComputedRef } from "vue";

/** Lifecycle state mirrored to the ConfirmProvider data contract. */
export type ConfirmState = "idle" | "pending";

/** Kind of promise the active request settles. */
export type ConfirmRequestKind = "choose" | "confirm";

/** One labelled action button rendered by ConfirmProvider. */
export interface ConfirmAction<Value extends string = string> {
  /** Value the request resolves with when this action is chosen. */
  readonly value: Value;

  /** Visible button label. */
  readonly label: string;

  /**
   * Mark the action as destructive (`data-destructive="true"`) for styling.
   *
   * @default false
   */
  readonly destructive?: boolean;
}

/** Options accepted by {@link ConfirmApi.confirm}. */
export interface ConfirmOptions<Data = unknown> {
  /** Required accessible title of the alert dialog. */
  readonly title: string;

  /**
   * Optional supporting text wired through `aria-describedby`.
   *
   * @default undefined
   */
  readonly description?: string;

  /**
   * Label of the accepting action.
   *
   * @default The provider's `confirmLabel`
   */
  readonly confirmLabel?: string;

  /**
   * Label of the cancelling action.
   *
   * @default The provider's `cancelLabel`
   */
  readonly cancelLabel?: string;

  /**
   * Mark the accepting action as destructive.
   *
   * @default false
   */
  readonly destructive?: boolean;

  /**
   * Consumer payload forwarded to custom `content` slot renderers.
   *
   * @default undefined
   */
  readonly data?: Data;
}

/** Options accepted by {@link ConfirmApi.choose}. */
export interface ConfirmChooseOptions<Value extends string, Data = unknown> {
  /** Required accessible title of the alert dialog. */
  readonly title: string;

  /**
   * Optional supporting text wired through `aria-describedby`.
   *
   * @default undefined
   */
  readonly description?: string;

  /**
   * Label of the cancelling action, which resolves `null`.
   *
   * @default The provider's `cancelLabel`
   */
  readonly cancelLabel?: string;

  /** Non-empty list of actions with unique values. */
  readonly actions: readonly [ConfirmAction<Value>, ...ConfirmAction<Value>[]];

  /**
   * Consumer payload forwarded to custom `content` slot renderers.
   *
   * @default undefined
   */
  readonly data?: Data;
}

/** Normalized request rendered by ConfirmProvider. */
export interface ConfirmRequest {
  /** Queue-local request id. */
  readonly id: string;

  /** Whether the request settles a boolean `confirm` or a `choose` value. */
  readonly kind: ConfirmRequestKind;

  /** Alert dialog title. */
  readonly title: string;

  /** Supporting text, or `null` when omitted. */
  readonly description: string | null;

  /** Resolved cancel label. */
  readonly cancelLabel: string;

  /** Actions in render order. `confirm` requests carry one action with value `"confirm"`. */
  readonly actions: readonly ConfirmAction[];

  /** Whether any action is destructive. */
  readonly destructive: boolean;

  /** Consumer payload. Narrow it in custom renderers. */
  readonly data: unknown;
}

/** Props passed to the ConfirmProvider `content` slot. */
export interface ConfirmSlotProps {
  /** Active request. */
  readonly request: ConfirmRequest;

  /** Settle the active request with one of its action values. */
  readonly resolve: (value: string) => void;

  /** Settle the active request as cancelled. */
  readonly cancel: () => void;
}

/** Imperative, promise-based confirmation API returned by {@link useConfirm}. */
export interface ConfirmApi<Data = unknown> {
  /**
   * Queue a yes/no confirmation.
   *
   * @returns `true` when the accepting action was chosen; `false` on cancel,
   * Escape, provider unmount, or {@link ConfirmApi.cancelAll}.
   */
  readonly confirm: (options: ConfirmOptions<Data>) => Promise<boolean>;

  /**
   * Queue a choice between several actions.
   *
   * @returns The chosen action value, or `null` when cancelled.
   */
  readonly choose: <const Value extends string>(
    options: ConfirmChooseOptions<Value, Data>,
  ) => Promise<Value | null>;

  /** Number of queued requests, including the one on screen. */
  readonly pending: ComputedRef<number>;

  /** Cancel every queued request. */
  readonly cancelAll: () => void;
}

/** Public instance exposed by ConfirmProvider. */
export interface ConfirmProviderExpose extends Omit<ConfirmApi, "pending"> {
  /** Number of queued requests, including the one on screen. */
  readonly pending: number;

  /** Request currently on screen, or `null`. */
  readonly active: ConfirmRequest | null;

  /** Stable state token. */
  readonly state: ConfirmState;
}
