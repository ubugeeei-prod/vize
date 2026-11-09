/**
 * Select Component - Dropdown/menu selection
 */

import { computed, defineComponent, h, ref, type PropType, watch } from "vue";
import { useInput } from "../composables/useInput.js";
import {
  adjacentEnabledIndex,
  createTypeaheadBuffer,
  firstEnabledIndex,
  lastEnabledIndex,
  typeaheadIndex,
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
    const highlightedIndex = ref(-1);
    const typeahead = createTypeaheadBuffer();

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

    return () => {
      const children = props.options.map((option, index) => {
        const isHighlighted = index === highlightedIndex.value;
        const isSelected = option.value === props.modelValue;
        const indicator = isHighlighted ? props.indicator : props.indicatorEmpty;

        return h(
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
        );
      });

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
