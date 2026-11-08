/**
 * Tabs Component - Tab navigation
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

export interface Tab {
  key: string;
  label: string;
  disabled?: boolean;
}

export interface TabsProps {
  /** Tab definitions */
  tabs: Tab[];
  /** Currently active tab key */
  modelValue?: string;
  /** Whether the tab list owns keyboard input */
  focused?: boolean;
  /** Tab bar position */
  position?: "top" | "bottom";
  /** Separator between tabs */
  separator?: string;
  /** Active tab foreground color */
  activeFg?: string;
  /** Active tab background color */
  activeBg?: string;
  /** Inactive tab foreground color */
  inactiveFg?: string;
  /** Show underline for active tab */
  underline?: boolean;
}

export const Tabs = defineComponent({
  name: "Tabs",
  props: {
    tabs: {
      type: Array as PropType<Tab[]>,
      required: true,
    },
    modelValue: String,
    focused: {
      type: Boolean,
      default: false,
    },
    position: {
      type: String as PropType<"top" | "bottom">,
      default: "top",
    },
    separator: {
      type: String,
      default: " | ",
    },
    activeFg: {
      type: String,
      default: "cyan",
    },
    activeBg: String,
    inactiveFg: String,
    underline: {
      type: Boolean,
      default: true,
    },
  },
  emits: ["update:modelValue", "change"],
  setup(props, { slots, emit }) {
    const highlightedIndex = ref(-1);
    const typeahead = createTypeaheadBuffer();

    const syncHighlight = () => {
      const activeIndex = props.tabs.findIndex(
        (tab) => tab.key === props.modelValue && !tab.disabled,
      );
      if (activeIndex >= 0) {
        highlightedIndex.value = activeIndex;
      } else if (
        highlightedIndex.value < 0 ||
        highlightedIndex.value >= props.tabs.length ||
        props.tabs[highlightedIndex.value]?.disabled
      ) {
        highlightedIndex.value = firstEnabledIndex(props.tabs);
      }
    };

    watch([() => props.modelValue, () => props.tabs], syncHighlight, {
      deep: true,
      immediate: true,
    });

    const activate = () => {
      const tab = props.tabs[highlightedIndex.value];
      if (!tab || tab.disabled) return;
      emit("update:modelValue", tab.key);
      emit("change", tab.key);
    };

    useInput({
      isActive: computed(() => props.focused),
      onArrow: (direction) => {
        if (direction !== "left" && direction !== "right") return;
        highlightedIndex.value = adjacentEnabledIndex(
          props.tabs,
          highlightedIndex.value,
          direction === "left" ? -1 : 1,
        );
      },
      onKey: (key) => {
        if (key === "home") highlightedIndex.value = firstEnabledIndex(props.tabs);
        if (key === "end") highlightedIndex.value = lastEnabledIndex(props.tabs);
      },
      onChar: (character) => {
        if (character === " ") {
          activate();
          return;
        }
        highlightedIndex.value = typeaheadIndex(
          props.tabs,
          highlightedIndex.value,
          typeahead.push(character),
          (tab) => tab.label,
        );
      },
      onSubmit: activate,
    });

    return () => {
      // Tab bar
      const tabItems: VNode[] = [];

      props.tabs.forEach((tab, index) => {
        if (index > 0) {
          tabItems.push(h("text", { key: `sep-${index}`, dim: true }, props.separator));
        }

        const isActive = tab.key === props.modelValue;
        const isHighlighted = index === highlightedIndex.value;

        tabItems.push(
          h(
            "text",
            {
              key: tab.key,
              fg: isActive || (props.focused && isHighlighted) ? props.activeFg : props.inactiveFg,
              bg: isActive || (props.focused && isHighlighted) ? props.activeBg : undefined,
              bold: isActive,
              underline: (isActive && props.underline) || (props.focused && isHighlighted),
              dim: tab.disabled,
            },
            tab.label,
          ),
        );
      });

      const tabBar = h(
        "box",
        {
          key: "tab-bar",
          style: {
            flex_direction: "row",
            padding_bottom: props.position === "top" ? 1 : 0,
            padding_top: props.position === "bottom" ? 1 : 0,
          },
        },
        tabItems,
      );

      // Content area
      const content = h(
        "box",
        {
          key: "content",
          style: { flex_grow: 1 },
        },
        slots.default?.(),
      );

      // Arrange based on position
      const children = props.position === "top" ? [tabBar, content] : [content, tabBar];

      return h(
        "box",
        {
          style: { flex_direction: "column", flex_grow: 1 },
        },
        children,
      );
    };
  },
});
