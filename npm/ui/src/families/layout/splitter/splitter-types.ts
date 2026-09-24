import type { Ref } from "vue";

/** Axis along which panels are laid out. `"horizontal"` places panels side by side. */
export type SplitterOrientation = "horizontal" | "vertical";

/** Reading direction used to map horizontal arrow keys and pointer deltas. */
export type SplitterDirection = "ltr" | "rtl";

/** Panel sizes in percent of the group, in panel order. A valid layout sums to 100. */
export type SplitterLayout = readonly number[];

/** Why a layout changed. */
export type SplitterResizeReason = "keyboard" | "pointer" | "programmatic";

/** State exposed by the SplitterGroup data contract. */
export type SplitterGroupState = "disabled" | "idle" | "resizing";

/** State exposed by each SplitterPanel data contract. */
export type SplitterPanelState = "collapsed" | "expanded";

/** State exposed by each SplitterHandle data contract. */
export type SplitterHandleState = "disabled" | "dragging" | "idle";

/** Size constraints of one panel, in percent of the group. */
export interface SplitterPanelConstraints {
  /** Smallest expanded size. */
  readonly minSize: number;

  /** Largest size. */
  readonly maxSize: number;

  /** Whether the panel may snap to `collapsedSize` below `minSize`. */
  readonly collapsible: boolean;

  /** Size used while collapsed. */
  readonly collapsedSize: number;
}

/** State exposed to the SplitterGroup default slot. */
export interface SplitterGroupSlotState {
  /** Resolved panel sizes in percent. */
  readonly layout: SplitterLayout;

  /** Layout axis. */
  readonly orientation: SplitterOrientation;

  /** Whether resizing is disabled. */
  readonly disabled: boolean;

  /** Whether a pointer drag is active. */
  readonly resizing: boolean;

  /** Stable state token for styling and tests. */
  readonly state: SplitterGroupState;
}

/** State exposed to the SplitterPanel default slot. */
export interface SplitterPanelSlotState {
  /** Current size in percent. */
  readonly size: number;

  /** Whether the panel is collapsed. */
  readonly collapsed: boolean;

  /** Zero-based panel position within its group. */
  readonly index: number;

  /** Stable state token. */
  readonly state: SplitterPanelState;
}

/** State exposed to the SplitterHandle default slot. */
export interface SplitterHandleSlotState {
  /** Size in percent of the panel before the handle. */
  readonly value: number;

  /** Whether this handle owns the active drag. */
  readonly dragging: boolean;

  /** Whether the handle or group is disabled. */
  readonly disabled: boolean;

  /** Layout axis of the owning group. */
  readonly orientation: SplitterOrientation;

  /** Stable state token. */
  readonly state: SplitterHandleState;
}

/** Public instance exposed by SplitterGroup. */
export interface SplitterGroupExpose {
  /** Rendered group element. */
  readonly element: HTMLDivElement | null;

  /** Group base id. */
  readonly id: string;

  /** Resolved layout. */
  readonly layout: SplitterLayout;

  /** Request a new layout. Invalid lengths are rejected and sizes are clamped to constraints. */
  readonly setLayout: (layout: SplitterLayout) => boolean;

  /** Restore `defaultLayout` or the panel defaults. */
  readonly reset: () => boolean;
}

/** Public instance exposed by SplitterPanel. */
export interface SplitterPanelExpose {
  /** Rendered panel element. */
  readonly element: HTMLDivElement | null;

  /** Panel id. */
  readonly id: string;

  /** Current size in percent. */
  readonly size: number;

  /** Whether the panel is collapsed. */
  readonly collapsed: boolean;

  /** Collapse a collapsible panel, remembering its size. */
  readonly collapse: () => boolean;

  /** Restore a collapsed panel to its remembered or minimum size. */
  readonly expand: () => boolean;

  /** Resize the panel toward `size` percent through its adjacent handle. */
  readonly resize: (size: number) => boolean;
}

/** Public instance exposed by SplitterHandle. */
export interface SplitterHandleExpose {
  /** Rendered separator element. */
  readonly element: HTMLDivElement | null;

  /** Size in percent of the panel before the handle. */
  readonly value: number;

  /** Move focus to the separator. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Synchronous key-value storage used to persist layouts, such as `localStorage` or a cookie adapter. */
export interface SplitterStorage {
  /** Read a stored value, or `null`. */
  readonly getItem: (key: string) => string | null;

  /** Write a value. */
  readonly setItem: (key: string, value: string) => void;
}

/** Options for {@link useSplitterPersistence}. */
export interface SplitterPersistenceOptions {
  /** Storage key. Use one key per group. */
  readonly key: string;

  /**
   * Storage adapter, or a factory resolved lazily on the client.
   *
   * @default globalThis.localStorage when available
   */
  readonly storage?: SplitterStorage | (() => SplitterStorage | null | undefined);

  /**
   * Read during setup instead of after mount. Only enable it for storage that is
   * readable on the server too (for example request cookies), or hydration mismatches.
   *
   * @default false
   */
  readonly immediate?: boolean;
}

/**
 * Writable layout ref returned by {@link useSplitterPersistence}. Bind it with
 * `v-model:layout`; `undefined` keeps the group on its defaults until a stored
 * layout loads.
 */
export type SplitterPersistedLayout = Ref<SplitterLayout | undefined>;
