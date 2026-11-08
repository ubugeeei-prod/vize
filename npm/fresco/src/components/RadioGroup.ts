/**
 * RadioGroup Component - Radio button group selection
 */

import { computed, defineComponent, h, ref, type PropType, watch } from "vue";
import { useInput } from "../composables/useInput.js";
import {
  adjacentEnabledIndex,
  firstEnabledIndex,
  lastEnabledIndex,
  typeaheadIndex,
} from "./collection-navigation.js";

export interface RadioOption {
  label: string;
  value: string;
  disabled?: boolean;
}

export interface RadioGroupProps {
  /** Radio options */
  options: RadioOption[];
  /** Currently selected value */
  modelValue?: string;
  /** Layout direction */
  direction?: "horizontal" | "vertical";
  /** Focused option index */
  focusedIndex?: number;
  /** Whether the radio group owns keyboard input */
  focused?: boolean;
  /** Selected indicator */
  selected?: string;
  /** Unselected indicator */
  unselected?: string;
  /** Foreground color */
  fg?: string;
  /** Selected foreground color */
  selectedFg?: string;
}

export const RadioGroup = defineComponent({
  name: "RadioGroup",
  props: {
    options: {
      type: Array as PropType<RadioOption[]>,
      required: true,
    },
    modelValue: String,
    direction: {
      type: String as PropType<"horizontal" | "vertical">,
      default: "vertical",
    },
    focusedIndex: Number,
    focused: {
      type: Boolean,
      default: false,
    },
    selected: {
      type: String,
      default: "◉",
    },
    unselected: {
      type: String,
      default: "○",
    },
    fg: String,
    selectedFg: {
      type: String,
      default: "green",
    },
  },
  emits: ["update:modelValue", "change", "update:focusedIndex"],
  setup(props, { emit }) {
    const highlightedIndex = ref(-1);
    let typeahead = "";
    let lastTypeaheadAt = 0;

    const syncHighlight = () => {
      const requested = props.focusedIndex;
      const requestedOption = requested === undefined ? undefined : props.options[requested];
      const selectedIndex = props.options.findIndex(
        (option) => option.value === props.modelValue && !option.disabled,
      );
      highlightedIndex.value =
        requestedOption && !requestedOption.disabled
          ? (requested ?? -1)
          : selectedIndex >= 0
            ? selectedIndex
            : firstEnabledIndex(props.options);
    };

    watch([() => props.focusedIndex, () => props.modelValue, () => props.options], syncHighlight, {
      deep: true,
      immediate: true,
    });

    const setHighlight = (index: number) => {
      if (index < 0 || index === highlightedIndex.value) return;
      highlightedIndex.value = index;
      emit("update:focusedIndex", index);
    };

    const activate = () => {
      const option = props.options[highlightedIndex.value];
      if (!option || option.disabled) return;
      emit("update:modelValue", option.value);
      emit("change", option.value);
    };

    useInput({
      isActive: computed(() => props.focused),
      onArrow: (direction) => {
        const delta =
          props.direction === "horizontal"
            ? direction === "left"
              ? -1
              : direction === "right"
                ? 1
                : 0
            : direction === "up"
              ? -1
              : direction === "down"
                ? 1
                : 0;
        if (delta !== 0) {
          setHighlight(
            adjacentEnabledIndex(props.options, highlightedIndex.value, delta as -1 | 1),
          );
        }
      },
      onKey: (key) => {
        if (key === "home") setHighlight(firstEnabledIndex(props.options));
        if (key === "end") setHighlight(lastEnabledIndex(props.options));
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
            props.options,
            highlightedIndex.value,
            typeahead,
            (option) => option.label,
          ),
        );
      },
      onSubmit: activate,
    });

    return () => {
      const children = props.options.map((option, index) => {
        const isSelected = option.value === props.modelValue;
        const isFocused = index === highlightedIndex.value;
        const indicator = isSelected ? props.selected : props.unselected;

        return h(
          "text",
          {
            key: option.value,
            fg: isSelected ? props.selectedFg : props.fg,
            bold: isFocused,
            dim: option.disabled,
          },
          `${indicator} ${option.label}`,
        );
      });

      return h(
        "box",
        {
          style: {
            flex_direction: props.direction === "horizontal" ? "row" : "column",
            gap: props.direction === "horizontal" ? 2 : 0,
          },
        },
        children,
      );
    };
  },
});
