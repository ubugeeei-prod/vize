/**
 * Menu Component - Command menu/palette
 */

import { computed, defineComponent, h, ref, type PropType, type VNode, watch } from "vue";
import { useInput } from "../composables/useInput.js";
import {
  adjacentEnabledIndex,
  createTypeaheadBuffer,
  firstEnabledIndex,
  lastEnabledIndex,
  typeaheadIndex,
} from "./collection-navigation.js";

export interface MenuItem {
  key: string;
  label: string;
  shortcut?: string;
  icon?: string;
  disabled?: boolean;
  separator?: boolean;
}

export interface MenuProps {
  /** Menu items */
  items: MenuItem[];
  /** Focused item index */
  focusedIndex?: number;
  /** Whether the menu owns keyboard input */
  focused?: boolean;
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

export const Menu = defineComponent({
  name: "Menu",
  props: {
    items: {
      type: Array as PropType<MenuItem[]>,
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
    const highlightedIndex = ref(-1);
    const typeahead = createTypeaheadBuffer();

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

    const setHighlight = (index: number) => {
      if (index < 0 || index === highlightedIndex.value) return;
      highlightedIndex.value = index;
      emit("update:focusedIndex", index);
    };

    const activate = () => {
      const item = props.items[highlightedIndex.value];
      if (!item || item.disabled || item.separator) return;
      emit("select", item.key);
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
    return () => {
      const children: VNode[] = [];

      props.items.forEach((item, index) => {
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
