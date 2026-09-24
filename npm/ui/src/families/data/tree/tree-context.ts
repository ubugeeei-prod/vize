import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  TreeCheckedState,
  TreeDirection,
  TreeDropPosition,
  TreeLoadState,
  TreeKey,
  TreeReorderItemRegistration,
  TreeSelectionMode,
} from "./tree-types.ts";

/** Row element registration owned by one mounted TreeItem. */
export interface TreeItemElementInput {
  readonly key: TreeKey;
  readonly element: Readonly<ShallowRef<HTMLDivElement | null>>;
}

/**
 * Key-level state and actions shared by the Tree compound parts.
 *
 * The context is deliberately key-based and non-generic: TreeItem receives its
 * typed row through props, while the root owns every key-to-node lookup.
 */
export interface TreeContextValue {
  readonly id: ComputedRef<string>;
  readonly disabled: ComputedRef<boolean>;
  readonly checkable: ComputedRef<boolean>;
  readonly selectionMode: ComputedRef<TreeSelectionMode>;
  readonly dir: ComputedRef<TreeDirection>;
  readonly activeKey: ComputedRef<TreeKey | null>;
  readonly virtualized: ComputedRef<boolean>;
  readonly reorderable: ComputedRef<boolean>;
  readonly getItemId: (key: TreeKey) => string;
  readonly isSelected: (key: TreeKey) => boolean;
  readonly isDisabled: (key: TreeKey) => boolean;
  readonly getCheckedState: (key: TreeKey) => TreeCheckedState;
  readonly getLoadState: (key: TreeKey) => TreeLoadState;
  readonly getDropPosition: (key: TreeKey) => TreeDropPosition | null;
  readonly registerElement: (input: TreeItemElementInput) => () => void;
  readonly measureElement: (element: Element | null, index: number) => void;
  readonly registerReorderItem: (
    key: TreeKey,
    element: () => Element | null,
    label: () => string,
  ) => TreeReorderItemRegistration | null;
  readonly onItemClick: (key: TreeKey, event: MouseEvent) => void;
  readonly onItemFocus: (key: TreeKey) => void;
  readonly toggleExpanded: (key: TreeKey, event: Event | null) => void;
  readonly toggleChecked: (key: TreeKey, event: Event | null) => boolean;
  readonly focusKey: (key: TreeKey, options?: FocusOptions) => boolean;
}

export const treeContext = createContext<TreeContextValue>("Tree");

/** Row-level context shared by TreeItem and its toggle and checkbox parts. */
export interface TreeItemContextValue {
  readonly key: ComputedRef<TreeKey>;
  readonly expandable: ComputedRef<boolean>;
  readonly expanded: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly checked: ComputedRef<TreeCheckedState>;
  readonly loadState: ComputedRef<TreeLoadState>;
}

export const treeItemContext = createContext<TreeItemContextValue>("TreeItem");
