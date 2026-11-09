/**
 * List Component - Scrollable list of items
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

interface ListItemBase {
  key: string;
  label: string;
  disabled?: boolean;
}

export type ListItem<Value = string> = ListItemBase &
  ([Value] extends [string] ? { value?: Value } : { value: Value });

export type ListValueKey<Value> = CollectionValueKey<Value>;
export type ListBy<Value> = CollectionBy<Value>;

export interface ListProps<Value = string> {
  /** List items */
  items: readonly ListItem<Value>[];
  /** Currently selected value */
  modelValue?: Value;
  /** Compare values by an object key or a custom equality function */
  by?: ListBy<Value>;
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

export interface RuntimeListItem extends ListItemBase {
  value?: unknown;
}

function listItemValue(item: RuntimeListItem): unknown {
  return Object.hasOwn(item, "value") ? item.value : item.key;
}

export const List = defineComponent({
  name: "List",
  props: {
    items: {
      type: Array as PropType<readonly RuntimeListItem[]>,
      required: true,
    },
    modelValue: null as unknown as PropType<unknown>,
    by: [String, Number, Symbol, Function] as PropType<RuntimeCollectionBy>,
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
    const typeahead = createTypeaheadBuffer();

    const syncHighlight = () => {
      const selectedIndex = props.items.findIndex(
        (item) =>
          collectionValuesEqual(listItemValue(item), props.modelValue, props.by) && !item.disabled,
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

    watch([() => props.modelValue, () => props.items, () => props.by], syncHighlight, {
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

    const moveBy = (direction: -1 | 1, distance = 1, wrap = true) => {
      let next = highlightedIndex.value;
      for (let step = 0; step < distance; step++) {
        const candidate = adjacentEnabledIndex(props.items, next, direction);
        if (candidate === next) break;
        if (!wrap && (direction > 0 ? candidate < next : candidate > next)) break;
        next = candidate;
      }
      highlightedIndex.value = next;
    };

    const activate = () => {
      const item = props.items[highlightedIndex.value];
      if (!item || item.disabled) return;
      const value = toRaw(listItemValue(item));
      emit("update:modelValue", value);
      emit("select", value);
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
        if (key === "pageup" || key === "pageUp") moveBy(-1, props.maxHeight ?? 5, false);
        if (key === "pagedown" || key === "pageDown") moveBy(1, props.maxHeight ?? 5, false);
      },
      onChar: (character) => {
        highlightedIndex.value = typeaheadIndex(
          props.items,
          highlightedIndex.value,
          typeahead.push(character),
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
        const isSelected = collectionValuesEqual(listItemValue(item), props.modelValue, props.by);
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
