/**
 * Menu Component - Command menu/palette
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

interface MenuItemBase {
  key: string;
  label: string;
  shortcut?: string;
  icon?: string;
  disabled?: boolean;
  separator?: boolean;
}

export type MenuItem<Value = string> = MenuItemBase &
  ([Value] extends [string] ? { value?: Value } : { value: Value });

export interface MenuProps<Value = string> {
  /** Menu items */
  items: readonly MenuItem<Value>[];
  /** Focused item index */
  focusedIndex?: number;
  /** Whether the menu owns keyboard input */
  focused?: boolean;
  /** Maximum visible items before the command window scrolls */
  maxHeight?: number;
  /** Show border */
  border?: "single" | "double" | "rounded" | "none";
  /** Width */
  width?: number;
  /** Foreground color */
  fg?: string;
  /** Focused foreground color */
  focusedFg?: string;
  /** Focused background color */
  focusedBg?: string;
  /** Shortcut foreground color */
  shortcutFg?: string;
}

export interface RuntimeMenuItem extends MenuItemBase {
  value?: unknown;
}

function menuItemValue(item: RuntimeMenuItem): unknown {
  return Object.hasOwn(item, "value") ? item.value : item.key;
}

export const Menu = defineComponent({
  name: "Menu",
  props: {
    items: {
      type: Array as PropType<readonly RuntimeMenuItem[]>,
      required: true,
    },
    focusedIndex: {
      type: Number,
      default: 0,
    },
    focused: {
      type: Boolean,
      default: false,
    },
    maxHeight: Number,
    border: {
      type: String as PropType<MenuProps["border"]>,
      default: "single",
    },
    width: Number,
    fg: String,
    focusedFg: {
      type: String,
      default: "black",
    },
    focusedBg: {
      type: String,
      default: "cyan",
    },
    shortcutFg: {
      type: String,
      default: "gray",
    },
  },
  emits: ["select", "update:focusedIndex"],
  setup(props, { emit }) {
    const scrollOffset = ref(0);
    const highlightedIndex = ref(-1);
    const typeahead = createTypeaheadBuffer();
    const viewportSize = computed(() => {
      if (props.maxHeight === undefined || !Number.isFinite(props.maxHeight)) return undefined;
      return props.maxHeight > 0 ? Math.max(1, Math.floor(props.maxHeight)) : undefined;
    });

    const syncHighlight = () => {
      const requested = props.focusedIndex;
      const requestedItem = props.items[requested];
      highlightedIndex.value =
        requestedItem && !requestedItem.disabled && !requestedItem.separator
          ? requested
          : firstEnabledIndex(
              props.items.map((item) => ({ disabled: item.disabled || item.separator })),
            );
    };

    watch([() => props.focusedIndex, () => props.items], syncHighlight, {
      deep: true,
      immediate: true,
    });

    watch(
      [highlightedIndex, viewportSize, () => props.items.length],
      ([index, windowSize]) => {
        scrollOffset.value = visibleCollectionOffset(
          index,
          scrollOffset.value,
          windowSize,
          props.items.length,
        );
      },
      { immediate: true },
    );

    const setHighlight = (index: number) => {
      if (index < 0 || index === highlightedIndex.value) return;
      highlightedIndex.value = index;
      emit("update:focusedIndex", index);
    };

    const activate = () => {
      const item = props.items[highlightedIndex.value];
      if (!item || item.disabled || item.separator) return;
      emit("select", toRaw(menuItemValue(item)));
    };

    useInput({
      isActive: computed(() => props.focused),
      onArrow: (direction) => {
        if (direction !== "up" && direction !== "down") return;
        setHighlight(
          adjacentEnabledIndex(
            props.items.map((item) => ({ disabled: item.disabled || item.separator })),
            highlightedIndex.value,
            direction === "up" ? -1 : 1,
          ),
        );
      },
      onKey: (key) => {
        const navigableItems = props.items.map((item) => ({
          disabled: item.disabled || item.separator,
        }));
        if (key === "home") setHighlight(firstEnabledIndex(navigableItems));
        if (key === "end") setHighlight(lastEnabledIndex(navigableItems));
      },
      onChar: (character) => {
        setHighlight(
          typeaheadIndex(
            props.items.map((item) => ({
              ...item,
              disabled: item.disabled || item.separator,
            })),
            highlightedIndex.value,
            typeahead.push(character),
            (item) => item.label,
          ),
        );
      },
      onSubmit: activate,
    });

    const visibleItems = computed(() => {
      const windowSize = viewportSize.value;
      const start = windowSize === undefined ? 0 : scrollOffset.value;
      const end = windowSize === undefined ? props.items.length : start + windowSize;
      return props.items.slice(start, end).map((item, visibleIndex) => ({
        item,
        index: start + visibleIndex,
      }));
    });

    const scrollIndicator = computed(() => {
      const windowSize = viewportSize.value;
      if (windowSize === undefined || props.items.length <= windowSize) {
        return { showUp: false, showDown: false };
      }
      return {
        showUp: scrollOffset.value > 0,
        showDown: scrollOffset.value + windowSize < props.items.length,
      };
    });

    return () => {
      const children: VNode[] = [];

      if (scrollIndicator.value.showUp) {
        children.push(h("text", { key: "scroll-up", dim: true }, "  ..."));
      }

      visibleItems.value.forEach(({ item, index }) => {
        if (item.separator) {
          children.push(
            h("text", { key: `sep-${index}`, dim: true }, "─".repeat(props.width ?? 20)),
          );
          return;
        }

        const isFocused = index === highlightedIndex.value;
        const itemContent: VNode[] = [];

        // Icon
        if (item.icon) {
          itemContent.push(h("text", { key: "icon" }, `${item.icon} `));
        }

        // Label
        itemContent.push(
          h(
            "text",
            {
              key: "label",
              fg: isFocused ? props.focusedFg : props.fg,
              bg: isFocused ? props.focusedBg : undefined,
              dim: item.disabled,
              style: { flex_grow: 1 },
            },
            item.label,
          ),
        );

        // Shortcut
        if (item.shortcut) {
          itemContent.push(
            h(
              "text",
              {
                key: "shortcut",
                fg: props.shortcutFg,
                dim: true,
              },
              `  ${item.shortcut}`,
            ),
          );
        }

        children.push(
          h(
            "box",
            {
              key: item.key,
              style: {
                flex_direction: "row",
                padding_left: 1,
                padding_right: 1,
              },
              bg: isFocused ? props.focusedBg : undefined,
            },
            itemContent,
          ),
        );
      });

      if (scrollIndicator.value.showDown) {
        children.push(h("text", { key: "scroll-down", dim: true }, "  ..."));
      }

      return h(
        "box",
        {
          border: props.border === "none" ? undefined : props.border,
          style: {
            flex_direction: "column",
            width: props.width ? String(props.width) : undefined,
          },
        },
        children,
      );
    };
  },
});
