/**
 * Tree Component - Tree view for hierarchical data
 */

import { computed, defineComponent, h, ref, type PropType, watch } from "vue";
import { useInput } from "../composables/useInput.js";
import {
  adjacentEnabledIndex,
  firstEnabledIndex,
  lastEnabledIndex,
  typeaheadIndex,
} from "./collection-navigation.js";

export interface TreeNode {
  key: string;
  label: string;
  children?: TreeNode[];
  icon?: string;
  disabled?: boolean;
}

export interface TreeProps {
  /** Tree data */
  data: TreeNode[];
  /** Expanded node keys */
  expanded?: string[];
  /** Selected node key */
  selected?: string;
  /** Whether the tree owns keyboard input */
  focused?: boolean;
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

export const Tree = defineComponent({
  name: "Tree",
  props: {
    data: {
      type: Array as PropType<TreeNode[]>,
      required: true,
    },
    expanded: {
      type: Array as PropType<string[]>,
      default: () => [],
    },
    selected: String,
    focused: {
      type: Boolean,
      default: false,
    },
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
      readonly node: TreeNode;
      readonly parentKey?: string;
      readonly prefix: string;
      readonly icon: string;
    }

    const visibleNodes = computed<VisibleTreeNode[]>(() => {
      const expanded = new Set(props.expanded);
      const visible: VisibleTreeNode[] = [];

      const append = (
        nodes: readonly TreeNode[],
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
    let typeahead = "";
    let lastTypeaheadAt = 0;

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
        ({ node }) => node.key === props.selected && !node.disabled,
      );
      const nextIndex =
        selectedIndex >= 0 ? selectedIndex : firstEnabledIndex(navigableNodes.value);
      highlightedKey.value = visibleNodes.value[nextIndex]?.node.key;
    };

    watch(
      () => props.selected,
      (selected) => {
        const selectedIndex = visibleNodes.value.findIndex(
          ({ node }) => node.key === selected && !node.disabled,
        );
        if (selectedIndex >= 0) {
          highlightedKey.value = visibleNodes.value[selectedIndex]?.node.key;
        } else syncVisibleHighlight();
      },
      { immediate: true },
    );
    watch(visibleNodes, syncVisibleHighlight, { deep: true });

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
      emit("update:selected", entry.node.key);
      emit("select", entry.node.key);
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
        const now = Date.now();
        const normalized = character.toLocaleLowerCase();
        typeahead =
          now - lastTypeaheadAt > 700 || typeahead === normalized
            ? normalized
            : typeahead + normalized;
        lastTypeaheadAt = now;
        setHighlight(
          typeaheadIndex(
            visibleNodes.value.map(({ node }) => node),
            highlightedIndex.value,
            typeahead,
            (node) => node.label,
          ),
        );
      },
      onSubmit: activate,
    });

    return () =>
      h(
        "box",
        { style: { flex_direction: "column" } },
        visibleNodes.value.map(({ node, prefix, icon }) => {
          const isSelected = node.key === props.selected;
          const isHighlighted = node.key === highlightedKey.value;
          return h(
            "text",
            {
              key: node.key,
              fg: isSelected || isHighlighted ? props.selectedFg : props.fg,
              bold: isSelected,
              underline: props.focused && isHighlighted,
              dim: node.disabled,
            },
            `${prefix}${icon} ${node.label}`,
          );
        }),
      );
  },
});
