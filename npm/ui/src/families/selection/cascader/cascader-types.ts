import type { Placement, PositionerStrategy } from "../../overlays/positioner/positioner.ts";
import type { CascaderBy, CascaderExpandTrigger, CascaderModelValue } from "./cascader-model.ts";

export type {
  CascaderBy,
  CascaderEquality,
  CascaderExpandTrigger,
  CascaderModelValue,
  CascaderValueKey,
} from "./cascader-model.ts";

/** Open state mirrored to the Cascader data contract. */
export type CascaderState = "closed" | "open";

/** State token for each option. */
export type CascaderItemState = "checked" | "partial" | "unchecked";

/** Context handed to `loadChildren`. */
export interface CascaderLoadContext {
  /** Aborted when another branch at the same level supersedes this load, or on unmount. */
  readonly signal: AbortSignal;
}

/** Placement accepted by `CascaderContent`. */
export type CascaderPlacement = Placement;

/** CSS strategy accepted by `CascaderContent`. */
export type CascaderPositionerStrategy = PositionerStrategy;

/** Public props accepted by `CascaderRoot`. */
export interface CascaderRootProps<T, Multiple extends boolean = false> {
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /** Top-level options. @default required */
  readonly options: readonly T[];

  /**
   * Controlled selection: one path, or a list of leaf paths when `multiple` is `true`.
   *
   * @default undefined
   */
  readonly modelValue?: CascaderModelValue<T, Multiple>;

  /**
   * Initial selection for uncontrolled use and the value restored by reset.
   *
   * @default undefined
   */
  readonly defaultValue?: CascaderModelValue<T, Multiple>;

  /**
   * Select several leaf paths. The literal type decides the model type.
   *
   * @default false
   */
  readonly multiple?: Multiple;

  /**
   * Child accessor. Defaults to an array-valued `children` property.
   *
   * @default undefined
   */
  readonly getChildren?: (node: T) => readonly T[] | undefined;

  /**
   * Leaf predicate. Defaults to "has no children and nothing to load".
   *
   * @default undefined
   */
  readonly isLeaf?: (node: T) => boolean;

  /**
   * Lazily load a branch's children the first time it expands.
   *
   * @default undefined
   */
  readonly loadChildren?: (node: T, context: CascaderLoadContext) => Promise<readonly T[]>;

  /**
   * Compare nodes by a property key or with a custom equality function.
   *
   * @default undefined
   */
  readonly by?: CascaderBy<T>;

  /**
   * Human-readable text for a node, used by the value display, typeahead, and search.
   *
   * @default undefined
   */
  readonly itemText?: (node: T) => string;

  /**
   * Disable individual nodes.
   *
   * @default undefined
   */
  readonly itemDisabled?: (node: T) => boolean;

  /**
   * Serialize one node for form submission; path segments are joined by `separator`.
   *
   * @default undefined
   */
  readonly formValue?: (node: T) => string;

  /**
   * Allow choosing branch options, not only leaves (single mode).
   *
   * @default false
   */
  readonly changeOnSelect?: boolean;

  /**
   * Expand branches on click or on pointer hover.
   *
   * @default "click"
   */
  readonly expandTrigger?: CascaderExpandTrigger;

  /**
   * Separator between path segments in display text and form values.
   *
   * @default " / "
   */
  readonly separator?: string;

  /**
   * Controlled search text; a non-empty query publishes `searchResults`.
   *
   * @default undefined
   */
  readonly search?: string;

  /**
   * Controlled open state. `undefined` selects uncontrolled behavior.
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
   * Disable the trigger and every option.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Mark a selection as required for assistive technology.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Form field name; each selected path submits one hidden input.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Id of the owning form when rendered outside it.
   *
   * @default undefined
   */
  readonly form?: string;

  /**
   * Text shown by `CascaderValue` while nothing is selected.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Idle time before buffered typeahead starts a new query.
   *
   * @default 500
   */
  readonly typeaheadTimeout?: number;
}

/** One rendered level of the cascade. */
export interface CascaderColumnState<T> {
  /** Zero-based depth. */
  readonly level: number;

  /** Branch whose children this column lists, or `null` for the top level. */
  readonly parent: T | null;

  /** Options of this level (empty while loading). */
  readonly options: readonly T[];

  /** Whether `loadChildren` is pending for `parent`. */
  readonly loading: boolean;
}

/** State exposed to the `CascaderRoot` slot. */
export interface CascaderSlotState<T> {
  /** Columns to render: the top level plus one per expanded branch. */
  readonly columns: readonly CascaderColumnState<T>[];

  /** Selected paths (at most one in single mode). */
  readonly selected: readonly (readonly T[])[];

  /** Display text for each selected path. */
  readonly selectedText: readonly string[];

  /** Paths matching `search`, or empty when the query is empty. */
  readonly searchResults: readonly (readonly T[])[];

  /** Choose a path, e.g. a search result. */
  readonly selectPath: (path: readonly T[]) => boolean;

  /** Whether the popup is open. */
  readonly open: boolean;

  /** Whether the cascader is disabled. */
  readonly disabled: boolean;

  /** Stable state token. */
  readonly state: CascaderState;
}

/** State exposed to `CascaderValue` slots. */
export interface CascaderValueSlotState {
  /** Display text for each selected path. */
  readonly selectedText: readonly string[];

  /** Placeholder text, when configured. */
  readonly placeholder: string | undefined;

  /** Whether nothing is selected. */
  readonly empty: boolean;
}

/** State exposed to `CascaderContent` slots. */
export interface CascaderContentSlotState {
  /** Whether the popup is open. */
  readonly open: boolean;

  /** Resolved placement after collision handling. */
  readonly placement: Placement;
}

/** State exposed to `CascaderColumn` slots. */
export interface CascaderColumnSlotState {
  /** Zero-based depth. */
  readonly level: number;

  /** Whether this column's children are loading. */
  readonly loading: boolean;
}

/** State exposed to `CascaderItem` slots. */
export interface CascaderItemSlotState<T> {
  /** Node value. */
  readonly value: T;

  /** Zero-based depth. */
  readonly level: number;

  /** Whether the node has (or may load) children. */
  readonly branch: boolean;

  /** Whether the node's child column is open. */
  readonly expanded: boolean;

  /** Whether the node's children are loading. */
  readonly loading: boolean;

  /** Whether the node is highlighted. */
  readonly active: boolean;

  /** Whether the node ends a selected path. */
  readonly selected: boolean;

  /** Whether the node lies on a selected path without ending it. */
  readonly partial: boolean;

  /** Whether the node is disabled. */
  readonly disabled: boolean;

  /** Stable state token. */
  readonly state: CascaderItemState;
}

/** Public instance exposed by `CascaderRoot`. */
export interface CascaderRootExpose<T> {
  /** Selected paths. */
  readonly selected: readonly (readonly T[])[];

  /** Whether the popup is open. */
  readonly open: boolean;

  /** Request a specific open state. */
  readonly setOpen: (open: boolean) => boolean;

  /** Choose a path (toggle in multiple mode). */
  readonly selectPath: (path: readonly T[]) => boolean;

  /** Expand the branch at the end of `path`, loading it when needed. */
  readonly expandPath: (path: readonly T[]) => void;

  /** Clear the selection. */
  readonly clear: () => boolean;

  /** Restore `defaultValue`. */
  readonly reset: () => boolean;

  /** Focus the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}
