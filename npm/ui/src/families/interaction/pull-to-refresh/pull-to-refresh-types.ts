/** Gesture and refresh lifecycle published through `data-state`. */
export type PullToRefreshState = "armed" | "idle" | "pulling" | "refreshing";

/** What started a refresh. */
export type PullToRefreshSource = "api" | "gesture" | "trigger";

/** Async refresh handler; the indicator stays in `refreshing` until it settles. */
export type PullToRefreshHandler = (source: PullToRefreshSource) => unknown;

/** State exposed to PullToRefresh slots and instances. */
export interface PullToRefreshSlotState {
  /** Lifecycle state. */
  readonly state: PullToRefreshState;

  /** Current pull distance in CSS px after resistance (held at `threshold` while refreshing). */
  readonly distance: number;

  /** `distance / threshold`, clamped to 0–1. */
  readonly progress: number;

  /** Whether a refresh is running. */
  readonly refreshing: boolean;

  /** Whether the user prefers reduced motion (known after mount). */
  readonly reducedMotion: boolean;
}

/** Public instance API of PullToRefresh. */
export interface PullToRefreshExpose extends PullToRefreshSlotState {
  /** Rendered scroll container. */
  readonly root: HTMLDivElement | null;

  /** Run the refresh handler (ignored while one is running); resolves when it settles. */
  readonly refresh: () => Promise<void>;
}
