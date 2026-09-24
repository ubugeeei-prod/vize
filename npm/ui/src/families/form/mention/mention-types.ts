import type { Placement } from "../../overlays/positioner/positioner.ts";
import type { MentionInsertTransform, MentionMatch, MentionTrigger } from "./mention-core.ts";

export type {
  MentionEdit,
  MentionInsertTransform,
  MentionMatch,
  MentionTrigger,
} from "./mention-core.ts";

/** Open state mirrored to the Mention data contract. */
export type MentionState = "closed" | "open";

/** Status of the async item loader. */
export type MentionLoadStatus = "error" | "idle" | "loading" | "success";

/** Context handed to `loadItems`. */
export interface MentionLoadContext {
  /** Aborted when a newer query supersedes this request, the popup closes, or the root unmounts. */
  readonly signal: AbortSignal;
}

/** Async item source called with the active query and trigger. */
export type MentionLoader<T> = (
  query: string,
  trigger: MentionTrigger,
  context: MentionLoadContext,
) => Promise<readonly T[]> | readonly T[];

/** Filter deciding whether an item matches the active query. */
export type MentionFilter<T> = (
  item: T,
  query: string,
  trigger: MentionTrigger,
  text: string,
) => boolean;

/** Public props accepted by `MentionRoot`. */
export interface MentionRootProps<T> {
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled field text for `MentionInput`. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: string;

  /**
   * Initial field text for uncontrolled use.
   *
   * @default ""
   */
  readonly defaultValue?: string;

  /**
   * Candidate items filtered by the active query and exposed as `filteredItems`.
   *
   * @default undefined
   */
  readonly items?: readonly T[];

  /**
   * Async item source; its results replace `items`.
   *
   * @default undefined
   */
  readonly loadItems?: MentionLoader<T>;

  /**
   * Debounce in milliseconds before `loadItems` runs for a changed query.
   *
   * @default 150
   */
  readonly debounce?: number;

  /**
   * Trigger characters and their query rules.
   *
   * @default [{ char: "@" }]
   */
  readonly triggers?: readonly MentionTrigger[];

  /**
   * Human-readable text for an item, used by filtering and the default insertion.
   *
   * @default undefined
   */
  readonly itemText?: (item: T) => string;

  /**
   * Filter for `items`; `false` keeps every item (server-side filtering).
   *
   * @default undefined
   */
  readonly filter?: MentionFilter<T> | false;

  /**
   * Text inserted for a chosen item. Defaults to trigger + item text + one space.
   *
   * @default undefined
   */
  readonly insert?: MentionInsertTransform<T>;

  /**
   * Controlled popup open state. The popup only shows while a trigger token is active.
   *
   * @default undefined
   */
  readonly open?: boolean;

  /**
   * Initial open state for uncontrolled use.
   *
   * @default false
   */
  readonly defaultOpen?: boolean;

  /**
   * Wrap arrow-key navigation at the first and last item.
   *
   * @default false
   */
  readonly loop?: boolean;

  /**
   * Disable trigger detection and editing.
   *
   * @default false
   */
  readonly disabled?: boolean;
}

/** State exposed to `MentionRoot` slots. */
export interface MentionSlotState<T> {
  /** Current field text. */
  readonly text: string;

  /** Active query, or `""` when no token is active. */
  readonly query: string;

  /** Active trigger, or `null`. */
  readonly trigger: MentionTrigger | null;

  /** Active token, or `null`. */
  readonly match: MentionMatch | null;

  /** Items (or loaded items) after filtering. */
  readonly filteredItems: readonly T[];

  /** Whether the popup is open. */
  readonly open: boolean;

  /** Whether `loadItems` is pending. */
  readonly loading: boolean;

  /** Async loader status. */
  readonly status: MentionLoadStatus;

  /** Last loader error, if any. */
  readonly error: unknown;

  /** Stable state token. */
  readonly state: MentionState;
}

/** State exposed to `MentionItem` slots. */
export interface MentionItemSlotState<T> {
  /** Item value. */
  readonly value: T;

  /** Whether this item is highlighted. */
  readonly active: boolean;

  /** Whether this item is disabled. */
  readonly disabled: boolean;
}

/** State exposed to `MentionContent` slots. */
export interface MentionContentSlotState {
  /** Whether the popup is open. */
  readonly open: boolean;

  /** Resolved placement. */
  readonly placement: Placement;

  /** Active query. */
  readonly query: string;
}

/** Public instance exposed by `MentionRoot`. */
export interface MentionRootExpose<T> {
  /** Current field text. */
  readonly text: string;

  /** Active token, or `null`. */
  readonly match: MentionMatch | null;

  /** Whether the popup is open. */
  readonly open: boolean;

  /** Insert `item` for the active token and report whether text changed. */
  readonly select: (item: T) => boolean;

  /** Close the popup until the caret enters a different token. */
  readonly dismiss: () => void;

  /** Re-read text and caret from the field and re-detect the token. */
  readonly refresh: () => void;

  /** Focus the field. */
  readonly focus: (options?: FocusOptions) => void;
}
