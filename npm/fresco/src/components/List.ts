/**
 * List Component - Scrollable list of items
 */

import { computed, defineComponent, h, ref, type PropType, type VNode, watch } from "vue";
import { useInput } from "../composables/useInput.js";
import {
  adjacentEnabledIndex,
  firstEnabledIndex,
  lastEnabledIndex,
  typeaheadIndex,
  visibleCollectionOffset,
} from "./collection-navigation.js";

export interface ListItem {
  key: string;
  label: string;
  disabled?: boolean;
}

export interface ListProps {
  /** List items */
  items: ListItem[];
  /** Currently selected key */
  modelValue?: string;
  /** Maximum visible items (enables scrolling) */
  maxHeight?: number;
  /** Whether the list is focused */
  focused?: boolean;
  /** Item indicator */
  indicator?: string;
  /** Empty indicator */
  indicatorEmpty?: string;
  /** Foreground color */
  fg?: string;
  /** Selected foreground color */
  selectedFg?: string;
  /** Selected background color */
  selectedBg?: string;
}

export const List = defineComponent({
  name: "List",
  props: {
    items: {
      type: Array as PropType<ListItem[]>,
      required: true,
    },
    modelValue: String,
    maxHeight: Number,
    focused: {
      type: Boolean,
      default: false,
    },
    indicator: {
      type: String,
      default: "> ",
    },
    indicatorEmpty: {
      type: String,
      default: "  ",
    },
    fg: String,
    selectedFg: {
      type: String,
      default: "cyan",
    },
    selectedBg: String,
  },
  emits: ["update:modelValue", "select"],
  setup(props, { emit }) {
    const scrollOffset = ref(0);
    const highlightedIndex = ref(-1);
    let typeahead = "";
    let lastTypeaheadAt = 0;

    const syncHighlight = () => {
      const selectedIndex = props.items.findIndex(
        (item) => item.key === props.modelValue && !item.disabled,
      );
      if (selectedIndex >= 0) {
        highlightedIndex.value = selectedIndex;
      } else if (
        highlightedIndex.value < 0 ||
        highlightedIndex.value >= props.items.length ||
        props.items[highlightedIndex.value]?.disabled
      ) {
        highlightedIndex.value = firstEnabledIndex(props.items);
      }
    };

    watch([() => props.modelValue, () => props.items], syncHighlight, {
      deep: true,
      immediate: true,
    });

    watch(
      [highlightedIndex, () => props.maxHeight, () => props.items.length],
      ([index]) => {
        scrollOffset.value = visibleCollectionOffset(
          index,
          scrollOffset.value,
          props.maxHeight,
          props.items.length,
        );
      },
      { immediate: true },
    );

    const moveBy = (direction: -1 | 1, distance = 1) => {
      let next = highlightedIndex.value;
      for (let step = 0; step < distance; step++) {
        const candidate = adjacentEnabledIndex(props.items, next, direction);
        if (candidate === next) break;
        next = candidate;
      }
      highlightedIndex.value = next;
    };

    const activate = () => {
      const item = props.items[highlightedIndex.value];
      if (!item || item.disabled) return;
      emit("update:modelValue", item.key);
      emit("select", item.key);
    };

    useInput({
      isActive: computed(() => props.focused),
      onArrow: (direction) => {
        if (direction === "up") moveBy(-1);
        if (direction === "down") moveBy(1);
      },
      onKey: (key) => {
        if (key === "home") highlightedIndex.value = firstEnabledIndex(props.items);
        if (key === "end") highlightedIndex.value = lastEnabledIndex(props.items);
        if (key === "pageup" || key === "pageUp") moveBy(-1, props.maxHeight ?? 5);
        if (key === "pagedown" || key === "pageDown") moveBy(1, props.maxHeight ?? 5);
      },
      onChar: (character) => {
        const now = Date.now();
        const normalized = character.toLocaleLowerCase();
        typeahead =
          now - lastTypeaheadAt > 700 || typeahead === normalized
            ? normalized
            : typeahead + normalized;
        lastTypeaheadAt = now;
        highlightedIndex.value = typeaheadIndex(
          props.items,
          highlightedIndex.value,
          typeahead,
          (item) => item.label,
        );
      },
      onSubmit: activate,
    });

    const visibleItems = computed(() => {
      if (!props.maxHeight) {
        return props.items;
      }
      const start = scrollOffset.value;
      const end = start + props.maxHeight;
      return props.items.slice(start, end);
    });

    const scrollIndicator = computed(() => {
      if (!props.maxHeight || props.items.length <= props.maxHeight) {
        return { showUp: false, showDown: false };
      }
      return {
        showUp: scrollOffset.value > 0,
        showDown: scrollOffset.value + props.maxHeight < props.items.length,
      };
    });

    return () => {
      const children: VNode[] = [];

      // Scroll up indicator
      if (scrollIndicator.value.showUp) {
        children.push(h("text", { key: "scroll-up", dim: true }, "  ..."));
      }

      // Visible items
      visibleItems.value.forEach((item, visibleIndex) => {
        const actualIndex = scrollOffset.value + visibleIndex;
        const isHighlighted = actualIndex === highlightedIndex.value;
        const isSelected = item.key === props.modelValue;
        const indicator = isHighlighted ? props.indicator : props.indicatorEmpty;

        children.push(
          h(
            "text",
            {
              key: item.key,
              fg: isHighlighted ? props.selectedFg : props.fg,
              bg: isHighlighted ? props.selectedBg : undefined,
              dim: item.disabled,
              bold: isSelected,
            },
            `${indicator}${item.label}`,
          ),
        );
      });

      // Scroll down indicator
      if (scrollIndicator.value.showDown) {
        children.push(h("text", { key: "scroll-down", dim: true }, "  ..."));
      }

      return h(
        "box",
        {
          style: { flex_direction: "column" },
          fg: props.fg,
        },
        children,
      );
    };
  },
});
