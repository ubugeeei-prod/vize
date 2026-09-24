import type { ShallowRef } from "vue";

import type { VirtualItem } from "../../interaction/virtualizer/virtualizer.ts";

/** Serializable identity of one tree node. Keys must be unique across the whole tree. */
export type TreeKey = string | number;

/** Selection model owned by a Tree root. */
export type TreeSelectionMode = "none" | "single" | "multiple";

/** Tri-state checkbox value of one tree node. */
export type TreeCheckedState = "checked" | "mixed" | "unchecked";

/**
 * How checking a node affects related nodes.
 *
 * - `"cascade"` checks or clears every loaded descendant and derives parent state.
 * - `"independent"` toggles only the requested node.
 */
export type TreeCheckPropagation = "cascade" | "independent";

/** Lifecycle of asynchronously loaded children for one node. */
export type TreeLoadState = "error" | "idle" | "loaded" | "loading";

/** Reading direction used to map horizontal expand and collapse arrows. */
export type TreeDirection = "ltr" | "rtl";

/** State exposed by the Tree root data contract. */
export type TreeState = "disabled" | "empty" | "ready";

/** Expansion state exposed by each TreeItem data contract. */
export type TreeItemState = "collapsed" | "expanded" | "leaf";

/** Relative drop position reported by drag and keyboard reordering. */
export type TreeDropPosition = "after" | "before" | "inside";

/** Load context passed to `loadChildren`. */
export interface TreeLoadContext {
  /** Aborted when the Tree unmounts or the same node is reloaded. */
  readonly signal: AbortSignal;
}

/**
 * One visible row of the flattened tree.
 *
 * Trees render as a flat list with `aria-level`, `aria-setsize`, and
 * `aria-posinset`, which keeps DOM order equal to visual order and makes
 * virtualization possible without nested groups.
 */
export interface TreeFlatNode<T, K extends TreeKey> {
  /** Stable node key resolved through `getKey`. */
  readonly key: K;

  /** Consumer-owned node data. */
  readonly node: T;

  /** Zero-based index among visible rows. */
  readonly index: number;

  /** One-based depth used by `aria-level`. */
  readonly level: number;

  /** One-based position among siblings used by `aria-posinset`. */
  readonly posinset: number;

  /** Sibling count used by `aria-setsize`. */
  readonly setsize: number;

  /** Parent key, or `null` for root nodes. */
  readonly parentKey: K | null;

  /** Whether the node has, or may lazily load, children. */
  readonly expandable: boolean;

  /** Whether the node is currently expanded. */
  readonly expanded: boolean;
}

/** Move request emitted by pointer or keyboard reordering. */
export interface TreeMoveEvent<K extends TreeKey> {
  /** Key of the node being moved. */
  readonly key: K;

  /** Key of the node the move is relative to. */
  readonly targetKey: K;

  /** Placement relative to `targetKey`. */
  readonly position: TreeDropPosition;

  /** Input family that requested the move. */
  readonly source: "keyboard" | "pointer";

  /** Native event responsible for the move, or `null` for manual settlement. */
  readonly originalEvent: Event | null;
}

/** State exposed to the TreeRoot default slot. */
export interface TreeSlotState<T, K extends TreeKey> {
  /** Rows to render: the virtual window when a virtualizer is attached, otherwise every visible row. */
  readonly items: readonly TreeFlatNode<T, K>[];

  /** Every visible row in display order. */
  readonly visibleItems: readonly TreeFlatNode<T, K>[];

  /** Currently expanded keys. */
  readonly expanded: readonly K[];

  /** Currently selected keys. */
  readonly selected: readonly K[];

  /** Explicitly checked keys, normalized by the propagation policy. */
  readonly checked: readonly K[];

  /** Key owning roving focus, or `null` for an empty tree. */
  readonly activeKey: K | null;

  /** Selection model. */
  readonly selectionMode: TreeSelectionMode;

  /** Whether the whole tree is disabled. */
  readonly disabled: boolean;

  /** Stable state token for styling and tests. */
  readonly state: TreeState;
}

/** State exposed to TreeItem, TreeItemToggle, and TreeItemCheckbox slots. */
export interface TreeItemSlotState<T, K extends TreeKey> {
  /** Consumer-owned node data. */
  readonly node: T;

  /** Stable node key. */
  readonly key: K;

  /** One-based depth. */
  readonly level: number;

  /** Whether the node has, or may lazily load, children. */
  readonly expandable: boolean;

  /** Whether the node is expanded. */
  readonly expanded: boolean;

  /** Whether the node is selected. */
  readonly selected: boolean;

  /** Tri-state checkbox value. */
  readonly checked: TreeCheckedState;

  /** Whether the node or the tree is disabled. */
  readonly disabled: boolean;

  /** Whether the node owns roving focus. */
  readonly active: boolean;

  /** Lazy children lifecycle for this node. */
  readonly loadState: TreeLoadState;

  /** Stable expansion state token. */
  readonly state: TreeItemState;
}

/** Public instance exposed by TreeRoot. */
export interface TreeRootExpose<K extends TreeKey> {
  /** Rendered tree element. */
  readonly element: HTMLDivElement | null;

  /** Root-owned base id. */
  readonly id: string;

  /** Currently expanded keys. */
  readonly expanded: readonly K[];

  /** Currently selected keys. */
  readonly selected: readonly K[];

  /** Normalized checked keys. */
  readonly checked: readonly K[];

  /** Key owning roving focus. */
  readonly activeKey: K | null;

  /** Move focus to the active, selected, or first row. */
  readonly focus: (options?: FocusOptions) => void;

  /** Make a visible key active and focus it, scrolling a virtual window when needed. */
  readonly focusKey: (key: K, options?: FocusOptions) => boolean;

  /** Expand one node, loading lazy children first when needed. */
  readonly expand: (key: K) => Promise<boolean>;

  /** Collapse one node. */
  readonly collapse: (key: K) => boolean;

  /** Toggle one node's expansion. */
  readonly toggle: (key: K) => Promise<boolean>;

  /** Expand every node whose children are already resolved. */
  readonly expandAll: () => boolean;

  /** Collapse every node. */
  readonly collapseAll: () => boolean;

  /** Replace the selected keys. Single mode keeps at most the first key. */
  readonly setSelected: (keys: readonly K[]) => boolean;

  /** Toggle one node's checkbox with the configured propagation. */
  readonly toggleChecked: (key: K) => boolean;

  /** Resolve the tri-state checkbox value of any known node. */
  readonly getCheckedState: (key: K) => TreeCheckedState;

  /** Reload lazy children for one node. */
  readonly reload: (key: K) => Promise<boolean>;
}

/** Public instance exposed by TreeItem. */
export interface TreeItemExpose<T, K extends TreeKey> extends TreeItemSlotState<T, K> {
  /** Rendered treeitem element. */
  readonly element: HTMLDivElement | null;

  /** Deterministic treeitem id. */
  readonly id: string;

  /** Make this row active and focus it. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Options for {@link useTreeVirtualizer}. */
export interface TreeVirtualizerOptions {
  /**
   * Exact row size in CSS pixels. Takes precedence over `estimateItemSize`.
   *
   * @default undefined
   */
  readonly itemSize?: number;

  /**
   * Estimated row size used until rows are measured.
   *
   * @default 32
   */
  readonly estimateItemSize?: number;

  /**
   * Extra rows rendered before and after the visible window.
   *
   * @default 4
   */
  readonly overscan?: number;

  /**
   * Viewport size assumed during SSR and before the tree element is measured.
   *
   * @default { width: 0, height: 320 }
   */
  readonly initialRect?: { readonly width: number; readonly height: number };
}

/** Row source a TreeRoot connects to its virtualizer. */
export interface TreeVirtualizerSource {
  /** Number of visible rows. */
  readonly count: () => number;

  /** Stable key of the visible row at `index`. */
  readonly getKey: (index: number) => TreeKey;
}

/** Windowing adapter created by {@link useTreeVirtualizer} and passed to TreeRoot. */
export interface TreeVirtualizer {
  /** Rows inside the rendered window. */
  readonly virtualItems: Readonly<ShallowRef<readonly VirtualItem[]>>;

  /** Total scrollable height in CSS pixels. */
  readonly totalSize: Readonly<ShallowRef<number>>;

  /** Main-axis start offset of a visible row, or `0` when outside the window. */
  readonly getItemStart: (index: number) => number;

  /** Connect the owning TreeRoot's rows. Called by TreeRoot. */
  readonly connect: (source: TreeVirtualizerSource) => void;

  /** Attach the scroll viewport. Called by TreeRoot. */
  readonly setViewport: (element: Element | null) => void;

  /** Measure one rendered row. Called by TreeItem. */
  readonly measureElement: (element: Element | null, index: number) => void;

  /** Scroll a row into the window. */
  readonly scrollToIndex: (index: number) => void;
}

/** Options for {@link useTreeReorder}. */
export interface TreeReorderOptions<K extends TreeKey> {
  /** Called when a pointer drop or `Alt+Arrow` keyboard move requests a new placement. */
  readonly onMove: (event: TreeMoveEvent<K>) => void;

  /**
   * Reject a move before it is emitted. Returning `false` cancels it.
   *
   * @default undefined
   */
  readonly canMove?: (event: TreeMoveEvent<K>) => boolean;

  /**
   * Suppress new drags and keyboard moves.
   *
   * @default false
   */
  readonly disabled?: () => boolean;
}

/** Registration for one draggable row. */
export interface TreeReorderItemRegistration {
  /** Whether this row owns the active pointer drag. */
  readonly isDragging: Readonly<ShallowRef<boolean>>;

  /** Pointer handlers merged onto the treeitem. */
  readonly pointerProps: {
    readonly onDragstart: (event: DragEvent) => void;
    readonly onMousedown: (event: MouseEvent) => void;
    readonly onPointerdown: (event: PointerEvent) => void;
    readonly onTouchstart: (event: TouchEvent) => void;
  };

  /** Remove the row from the drag controller. */
  readonly dispose: () => void;
}

/** Drag reorder adapter created by {@link useTreeReorder} and passed to TreeRoot. */
export interface TreeReorderController<K extends TreeKey> {
  /** Whether a pointer drag is active. */
  readonly isDragging: Readonly<ShallowRef<boolean>>;

  /** Projected drop target during a pointer drag. */
  readonly dropTarget: Readonly<
    ShallowRef<{ readonly key: K; readonly position: TreeDropPosition } | null>
  >;

  /** Register one rendered row. Called by TreeItem. */
  readonly registerItem: (input: {
    readonly key: K;
    readonly element: () => Element | null;
    readonly label: () => string;
    readonly disabled: () => boolean;
  }) => TreeReorderItemRegistration;

  /** Request a move through the same validation as pointer drops. */
  readonly move: (event: TreeMoveEvent<K>) => boolean;

  /** Cancel an active pointer drag. */
  readonly cancel: () => boolean;
}
