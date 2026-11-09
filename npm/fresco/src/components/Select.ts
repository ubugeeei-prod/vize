/**
 * Select Component - Dropdown/menu selection
 */

import { computed, defineComponent, h, ref, type PropType, type VNode, watch } from "vue";
import { useInput } from "../composables/useInput.js";
import {
  adjacentEnabledIndex,
  createTypeaheadBuffer,
  firstEnabledIndex,
  lastEnabledIndex,
  typeaheadIndex,
  visibleCollectionOffset,
} from "./collection-navigation.js";

export interface SelectOption {
  label: string;
  value: string;
  disabled?: boolean;
}

export interface SelectProps {
  /** Options to display */
  options: SelectOption[];
  /** Currently selected value */
  modelValue?: string;
  /** Placeholder text */
  placeholder?: string;
  /** Maximum visible options before the selection window scrolls */
  maxHeight?: number;
  /** Whether the select is focused */
  focused?: boolean;
  /** Indicator for selected item */
  indicator?: string;
  /** Indicator for unselected item */
  indicatorEmpty?: string;
  /** Foreground color */
  fg?: string;
  /** Background color */
  bg?: string;
  /** Selected item foreground color */
  selectedFg?: string;
  /** Selected item background color */
  selectedBg?: string;
}

export const Select = defineComponent({
  name: "Select",
  props: {
    options: {
      type: Array as PropType<SelectOption[]>,
      required: true,
    },
    modelValue: String,
    placeholder: {
      type: String,
      default: "Select an option",
    },
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
    bg: String,
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
    const viewportSize = computed(() => {
      if (props.maxHeight === undefined || !Number.isFinite(props.maxHeight)) return undefined;
      return props.maxHeight > 0 ? Math.max(1, Math.floor(props.maxHeight)) : undefined;
    });

    // Find initial index based on modelValue
    watch(
      () => props.modelValue,
      (value) => {
        const selectedIndex = props.options.findIndex(
          (option) => option.value === value && !option.disabled,
        );
        highlightedIndex.value =
          selectedIndex >= 0 ? selectedIndex : firstEnabledIndex(props.options);
      },
      { immediate: true },
    );

    watch(
      () => props.options,
      (options) => {
        if (
          highlightedIndex.value < 0 ||
          highlightedIndex.value >= options.length ||
          options[highlightedIndex.value]?.disabled
        ) {
          highlightedIndex.value = firstEnabledIndex(options);
        }
      },
      { deep: true },
    );

    watch(
      [highlightedIndex, viewportSize, () => props.options.length],
      ([index, windowSize]) => {
        scrollOffset.value = visibleCollectionOffset(
          index,
          scrollOffset.value,
          windowSize,
          props.options.length,
        );
      },
      { immediate: true },
    );

    const activate = () => {
      const option = props.options[highlightedIndex.value];
      if (!option || option.disabled) return;
      emit("update:modelValue", option.value);
      emit("select", option.value);
    };

    useInput({
      isActive: computed(() => props.focused),
      onArrow: (direction) => {
        if (direction !== "up" && direction !== "down") return;
        highlightedIndex.value = adjacentEnabledIndex(
          props.options,
          highlightedIndex.value,
          direction === "up" ? -1 : 1,
        );
      },
      onKey: (key) => {
        if (key === "home") highlightedIndex.value = firstEnabledIndex(props.options);
        if (key === "end") highlightedIndex.value = lastEnabledIndex(props.options);
      },
      onChar: (character) => {
        highlightedIndex.value = typeaheadIndex(
          props.options,
          highlightedIndex.value,
          typeahead.push(character),
          (option) => option.label,
        );
      },
      onSubmit: activate,
    });

    const visibleOptions = computed(() => {
      const windowSize = viewportSize.value;
      const start = windowSize === undefined ? 0 : scrollOffset.value;
      const end = windowSize === undefined ? props.options.length : start + windowSize;
      return props.options.slice(start, end).map((option, visibleIndex) => ({
        option,
        index: start + visibleIndex,
      }));
    });

    const scrollIndicator = computed(() => {
      const windowSize = viewportSize.value;
      if (windowSize === undefined || props.options.length <= windowSize) {
        return { showUp: false, showDown: false };
      }
      return {
        showUp: scrollOffset.value > 0,
        showDown: scrollOffset.value + windowSize < props.options.length,
      };
    });

    return () => {
      const children: VNode[] = [];
      if (scrollIndicator.value.showUp) {
        children.push(h("text", { key: "scroll-up", dim: true }, "  ..."));
      }

      for (const { option, index } of visibleOptions.value) {
        const isHighlighted = index === highlightedIndex.value;
        const isSelected = option.value === props.modelValue;
        const indicator = isHighlighted ? props.indicator : props.indicatorEmpty;

        children.push(
          h(
            "box",
            {
              key: option.value,
              style: { flex_direction: "row" },
            },
            [
              h(
                "text",
                {
                  fg: isHighlighted ? props.selectedFg : props.fg,
                  bg: isHighlighted ? props.selectedBg : props.bg,
                  dim: option.disabled,
                },
                `${indicator}${option.label}${isSelected ? " (selected)" : ""}`,
              ),
            ],
          ),
        );
      }

      if (scrollIndicator.value.showDown) {
        children.push(h("text", { key: "scroll-down", dim: true }, "  ..."));
      }

      return h(
        "box",
        {
          style: { flex_direction: "column" },
          fg: props.fg,
          bg: props.bg,
        },
        children,
      );
    };
  },
});
