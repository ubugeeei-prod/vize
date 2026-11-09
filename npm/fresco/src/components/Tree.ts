/**
 * Tree Component - Tree view for hierarchical data
 */

import { computed, defineComponent, h, ref, toRaw, type PropType, type VNode, watch } from "vue";
import { useInput } from "../composables/useInput.js";
import {
  adjacentEnabledIndex,
  createTypeaheadBuffer,
  firstEnabledIndex,
  lastEnabledIndex,
  typeaheadIndex,
  visibleCollectionOffset,
} from "./collection-navigation.js";
import {
  collectionValuesEqual,
  type CollectionBy,
  type CollectionValueKey,
  type RuntimeCollectionBy,
} from "./collection-values.js";

interface TreeNodeBase<Value> {
  key: string;
  label: string;
  children?: readonly TreeNode<Value>[];
  icon?: string;
  disabled?: boolean;
}

export type TreeNode<Value = string> = TreeNodeBase<Value> &
  ([Value] extends [string] ? { value?: Value } : { value: Value });

export type TreeValueKey<Value> = CollectionValueKey<Value>;
export type TreeBy<Value> = CollectionBy<Value>;

export interface TreeProps<Value = string> {
  /** Tree data */
  data: readonly TreeNode<Value>[];
  /** Expanded node keys */
  expanded?: readonly string[];
  /** Selected node value */
  selected?: Value;
  /** Compare selected values by an object key or a custom equality function */
  by?: TreeBy<Value>;
  /** Whether the tree owns keyboard input */
  focused?: boolean;
  /** Maximum visible flattened nodes before the tree window scrolls */
  maxHeight?: number;
  /** Show lines */
  showLines?: boolean;
  /** Indent size */
  indent?: number;
  /** Expanded icon */
  expandedIcon?: string;
  /** Collapsed icon */
  collapsedIcon?: string;
  /** Leaf icon */
  leafIcon?: string;
  /** Foreground color */
  fg?: string;
  /** Selected foreground color */
  selectedFg?: string;
}

export interface RuntimeTreeNode {
  key: string;
  label: string;
  value?: unknown;
  children?: readonly RuntimeTreeNode[];
  icon?: string;
  disabled?: boolean;
}

function treeNodeValue(node: RuntimeTreeNode): unknown {
  return Object.hasOwn(node, "value") ? node.value : node.key;
}

export const Tree = defineComponent({
  name: "Tree",
  props: {
    data: {
      type: Array as PropType<readonly RuntimeTreeNode[]>,
      required: true,
    },
    expanded: {
      type: Array as PropType<readonly string[]>,
      default: () => [],
    },
    selected: null as unknown as PropType<unknown>,
    by: [String, Number, Symbol, Function] as PropType<RuntimeCollectionBy>,
    focused: {
      type: Boolean,
      default: false,
    },
    maxHeight: Number,
    showLines: {
      type: Boolean,
      default: true,
    },
    indent: {
      type: Number,
      default: 2,
    },
    expandedIcon: {
      type: String,
      default: "▼",
    },
    collapsedIcon: {
      type: String,
      default: "▶",
    },
    leafIcon: {
      type: String,
      default: "•",
    },
    fg: String,
    selectedFg: {
      type: String,
      default: "cyan",
    },
  },
  emits: ["select", "toggle", "update:selected", "update:expanded"],
  setup(props, { emit }) {
    interface VisibleTreeNode {
      readonly node: RuntimeTreeNode;
      readonly parentKey?: string;
      readonly prefix: string;
      readonly icon: string;
    }

    const visibleNodes = computed<VisibleTreeNode[]>(() => {
      const expanded = new Set(props.expanded);
      const visible: VisibleTreeNode[] = [];

      const append = (
        nodes: readonly RuntimeTreeNode[],
        depth: number,
        prefix: string,
        parentKey?: string,
      ) => {
        nodes.forEach((node, index) => {
          const isLast = index === nodes.length - 1;
          const hasChildren = (node.children?.length ?? 0) > 0;
          const isExpanded = expanded.has(node.key);
          const line = props.showLines && depth > 0 ? (isLast ? "└─" : "├─") : "";
          const icon =
            node.icon ??
            (hasChildren
              ? isExpanded
                ? props.expandedIcon
                : props.collapsedIcon
              : props.leafIcon);

          visible.push({ node, parentKey, prefix: `${prefix}${line}`, icon });

          if (!hasChildren || !isExpanded) return;
          const branch = props.showLines && depth > 0 ? (isLast ? "  " : "│ ") : "";
          append(
            node.children ?? [],
            depth + 1,
            `${prefix}${branch}${" ".repeat(props.indent)}`,
            node.key,
          );
        });
      };

      append(props.data, 0, "");
      return visible;
    });

    const highlightedKey = ref<string>();
    const scrollOffset = ref(0);
    const typeahead = createTypeaheadBuffer();
    const viewportSize = computed(() => {
      if (props.maxHeight === undefined || !Number.isFinite(props.maxHeight)) return undefined;
      return props.maxHeight > 0 ? Math.max(1, Math.floor(props.maxHeight)) : undefined;
    });

    const navigableNodes = computed(() =>
      visibleNodes.value.map(({ node }) => ({ disabled: node.disabled })),
    );
    const highlightedIndex = computed(() =>
      visibleNodes.value.findIndex(({ node }) => node.key === highlightedKey.value),
    );

    const syncVisibleHighlight = () => {
      const currentIndex = highlightedIndex.value;
      if (currentIndex >= 0 && !visibleNodes.value[currentIndex]?.node.disabled) return;

      const selectedIndex = visibleNodes.value.findIndex(
        ({ node }) =>
          collectionValuesEqual(treeNodeValue(node), props.selected, props.by) && !node.disabled,
      );
      const nextIndex =
        selectedIndex >= 0 ? selectedIndex : firstEnabledIndex(navigableNodes.value);
      highlightedKey.value = visibleNodes.value[nextIndex]?.node.key;
    };

    watch(
      [() => props.selected, () => props.by],
      ([selected, by]) => {
        const selectedIndex = visibleNodes.value.findIndex(
          ({ node }) => collectionValuesEqual(treeNodeValue(node), selected, by) && !node.disabled,
        );
        if (selectedIndex >= 0) {
          highlightedKey.value = visibleNodes.value[selectedIndex]?.node.key;
        } else syncVisibleHighlight();
      },
      { immediate: true },
    );
    watch(visibleNodes, syncVisibleHighlight, { deep: true });
    watch(
      [highlightedIndex, viewportSize, () => visibleNodes.value.length],
      ([index, windowSize]) => {
        scrollOffset.value = visibleCollectionOffset(
          index,
          scrollOffset.value,
          windowSize,
          visibleNodes.value.length,
        );
      },
      { immediate: true },
    );

    const setHighlight = (index: number) => {
      const entry = visibleNodes.value[index];
      if (!entry || entry.node.disabled) return;
      highlightedKey.value = entry.node.key;
    };

    const setExpanded = (key: string, expanded: boolean) => {
      const next = new Set(props.expanded);
      if (expanded) next.add(key);
      else next.delete(key);
      emit("update:expanded", [...next]);
      emit("toggle", key, expanded);
    };

    const activate = () => {
      const entry = visibleNodes.value[highlightedIndex.value];
      if (!entry || entry.node.disabled) return;
      const value = toRaw(treeNodeValue(entry.node));
      emit("update:selected", value);
      emit("select", value);
    };

    const moveToParent = (parentKey: string | undefined) => {
      let key = parentKey;
      while (key) {
        const index = visibleNodes.value.findIndex(({ node }) => node.key === key);
        const entry = visibleNodes.value[index];
        if (entry && !entry.node.disabled) {
          setHighlight(index);
          return;
        }
        key = entry?.parentKey;
      }
    };

    useInput({
      isActive: computed(() => props.focused),
      onArrow: (direction) => {
        const currentIndex = highlightedIndex.value;
        if (direction === "up" || direction === "down") {
          setHighlight(
            adjacentEnabledIndex(navigableNodes.value, currentIndex, direction === "up" ? -1 : 1),
          );
          return;
        }

        const entry = visibleNodes.value[currentIndex];
        if (!entry) return;
        const hasChildren = (entry.node.children?.length ?? 0) > 0;
        const isExpanded = props.expanded.includes(entry.node.key);
        if (direction === "right" && hasChildren) {
          if (!isExpanded) {
            setExpanded(entry.node.key, true);
          } else {
            const childIndex = visibleNodes.value.findIndex(
              (candidate) => candidate.parentKey === entry.node.key && !candidate.node.disabled,
            );
            setHighlight(childIndex);
          }
        } else if (direction === "left") {
          if (hasChildren && isExpanded) setExpanded(entry.node.key, false);
          else moveToParent(entry.parentKey);
        }
      },
      onKey: (key) => {
        if (key === "home") setHighlight(firstEnabledIndex(navigableNodes.value));
        if (key === "end") setHighlight(lastEnabledIndex(navigableNodes.value));
      },
      onChar: (character) => {
        if (character === " ") {
          activate();
          return;
        }
        setHighlight(
          typeaheadIndex(
            visibleNodes.value.map(({ node }) => node),
            highlightedIndex.value,
            typeahead.push(character),
            (node) => node.label,
          ),
        );
      },
      onSubmit: activate,
    });

    const viewportNodes = computed(() => {
      const windowSize = viewportSize.value;
      const start = windowSize === undefined ? 0 : scrollOffset.value;
      const end = windowSize === undefined ? visibleNodes.value.length : start + windowSize;
      return visibleNodes.value.slice(start, end);
    });

    const scrollIndicator = computed(() => {
      const windowSize = viewportSize.value;
      if (windowSize === undefined || visibleNodes.value.length <= windowSize) {
        return { showUp: false, showDown: false };
      }
      return {
        showUp: scrollOffset.value > 0,
        showDown: scrollOffset.value + windowSize < visibleNodes.value.length,
      };
    });

    return () => {
      const children: VNode[] = [];
      if (scrollIndicator.value.showUp) {
        children.push(h("text", { key: "scroll-up", dim: true }, "  ..."));
      }

      for (const { node, prefix, icon } of viewportNodes.value) {
        const isSelected = collectionValuesEqual(treeNodeValue(node), props.selected, props.by);
        const isHighlighted = node.key === highlightedKey.value;
        children.push(
          h(
            "text",
            {
              key: node.key,
              fg: isSelected || isHighlighted ? props.selectedFg : props.fg,
              bold: isSelected,
              underline: props.focused && isHighlighted,
              dim: node.disabled,
            },
            `${prefix}${icon} ${node.label}`,
          ),
        );
      }

      if (scrollIndicator.value.showDown) {
        children.push(h("text", { key: "scroll-down", dim: true }, "  ..."));
      }

      return h("box", { style: { flex_direction: "column" } }, children);
    };
  },
});
