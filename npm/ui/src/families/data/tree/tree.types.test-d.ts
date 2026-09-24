/** Compile-only assertions for the public Tree contract. */

import {
  Tree,
  TreeItem,
  TreeItemCheckbox,
  TreeItemToggle,
  TreeRoot,
  useTreeReorder,
  useTreeVirtualizer,
  type TreeCheckedState,
  type TreeCheckPropagation,
  type TreeDropPosition,
  type TreeFlatNode,
  type TreeItemExpose,
  type TreeItemSlotState,
  type TreeItemState,
  type TreeKey,
  type TreeLoadState,
  type TreeMoveEvent,
  type TreeReorderController,
  type TreeRootExpose,
  type TreeSelectionMode,
  type TreeSlotState,
  type TreeState,
  type TreeVirtualizer,
} from "./tree.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface FileNode {
  readonly path: string;
  readonly children?: readonly FileNode[];
}

interface OrgNode {
  readonly id: number;
  readonly reports: readonly OrgNode[];
}

type FileRootProps = Parameters<typeof TreeRoot<FileNode, string>>[0];
type FileItemProps = Parameters<typeof TreeItem<FileNode, string>>[0];

type _KeyIsSerializable = Expect<Equal<TreeKey, string | number>>;
type _SelectionModeIsLiteral = Expect<Equal<TreeSelectionMode, "multiple" | "none" | "single">>;
type _CheckedStateIsTriState = Expect<Equal<TreeCheckedState, "checked" | "mixed" | "unchecked">>;
type _PropagationIsLiteral = Expect<Equal<TreeCheckPropagation, "cascade" | "independent">>;
type _LoadStateIsLiteral = Expect<Equal<TreeLoadState, "error" | "idle" | "loaded" | "loading">>;
type _StateIsLiteral = Expect<Equal<TreeState, "disabled" | "empty" | "ready">>;
type _ItemStateIsLiteral = Expect<Equal<TreeItemState, "collapsed" | "expanded" | "leaf">>;
type _DropPositionIsLiteral = Expect<Equal<TreeDropPosition, "after" | "before" | "inside">>;
type _FlatNodeKeepsNodeType = Expect<Equal<TreeFlatNode<FileNode, string>["node"], FileNode>>;
type _FlatNodeParentKeyIsTyped = Expect<
  Equal<TreeFlatNode<OrgNode, number>["parentKey"], number | null>
>;
type _SlotItemsAreTyped = Expect<
  Equal<TreeSlotState<FileNode, string>["items"], readonly TreeFlatNode<FileNode, string>[]>
>;
type _ItemSlotNodeIsTyped = Expect<Equal<TreeItemSlotState<OrgNode, number>["node"], OrgNode>>;
type _RootExposeKeysAreTyped = Expect<Equal<TreeRootExpose<number>["selected"], readonly number[]>>;
type _ItemExposeElement = Expect<
  Equal<TreeItemExpose<FileNode, string>["element"], HTMLDivElement | null>
>;
type _MoveEventKeysAreTyped = Expect<Equal<TreeMoveEvent<number>["targetKey"], number>>;
type _ItemPropTakesTypedRow = Expect<Equal<FileItemProps["item"], TreeFlatNode<FileNode, string>>>;
type _GetKeyReturnsKey = Expect<Equal<ReturnType<FileRootProps["getKey"]>, string>>;

const rootProps: FileRootProps = {
  ariaLabel: "Files",
  checkPropagation: "cascade",
  checkable: true,
  defaultExpanded: ["src"],
  dir: "rtl",
  expandOnClick: true,
  getChildren: (node) => node.children,
  getKey: (node) => node.path,
  getTextValue: (node) => node.path,
  hasChildren: (node) => node.path.endsWith("/"),
  isDisabled: (node) => node.path.startsWith("."),
  items: [{ path: "src/" }],
  loadChildren: async (node, { signal }) => {
    void signal.aborted;
    return [{ path: `${node.path}index.ts` }];
  },
  selected: ["src"],
  selectionFollowsFocus: true,
  selectionMode: "multiple",
  typeahead: false,
  typeaheadTimeout: 300,
  "onUpdate:selected": (keys: readonly string[]) => keys,
  onAction: (key: string, node: FileNode, event: Event) => [key, node, event],
  onLoad: (key: string, children: readonly FileNode[]) => [key, children],
};

const orgProps: Parameters<typeof TreeRoot<OrgNode, number>>[0] = {
  getChildren: (node) => node.reports,
  getKey: (node) => node.id,
  items: [],
  // @ts-expect-error keys follow the inferred getKey type.
  defaultExpanded: ["1"],
};

const badSelectionMode: FileRootProps = {
  getKey: (node) => node.path,
  items: [],
  // @ts-expect-error selection mode is a closed union.
  selectionMode: "extended",
};

// @ts-expect-error tree keys must be serializable strings or numbers.
type BadKeyProps = Parameters<typeof TreeRoot<FileNode, boolean>>[0];

declare const reorder: TreeReorderController<string>;
declare const virtualizer: TreeVirtualizer;
const adapters: FileRootProps = { getKey: (node) => node.path, items: [], reorder, virtualizer };

declare const numericReorder: TreeReorderController<number>;
const mismatchedReorder: FileRootProps = {
  getKey: (node) => node.path,
  items: [],
  // @ts-expect-error the reorder adapter must use the tree's key type.
  reorder: numericReorder,
};

type ReorderFactory = typeof useTreeReorder<number>;
type _ReorderFactoryIsTyped = Expect<
  Equal<ReturnType<ReorderFactory>, TreeReorderController<number>>
>;
type _BadKeyPropsExist = BadKeyProps;
type _VirtualizerFactory = Expect<Equal<ReturnType<typeof useTreeVirtualizer>, TreeVirtualizer>>;

declare const root: TreeRootExpose<string>;
void root.expand("src");
root.focusKey("src");
root.setSelected(["src"]);
// @ts-expect-error imperative keys follow the tree key type.
root.focusKey(1);

void Tree;
void TreeItemCheckbox;
void TreeItemToggle;
void adapters;
void badSelectionMode;
void mismatchedReorder;
void orgProps;
void rootProps;
