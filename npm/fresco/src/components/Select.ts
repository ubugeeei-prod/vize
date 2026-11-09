/**
 * Select Component - Dropdown/menu selection
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

export interface SelectOption<Value = string> {
  /** Stable render identity when the value is not a Vue vnode key. */
  key?: string | number | symbol;
  label: string;
  value: Value;
  disabled?: boolean;
}

export type SelectValueKey<Value> = Value extends object
  ? Extract<keyof Value, string | number | symbol>
  : never;

export type SelectBy<Value> = SelectValueKey<Value> | ((left: Value, right: Value) => boolean);

export interface SelectProps<Value = string> {
  /** Options to display */
  options: readonly SelectOption<Value>[];
  /** Currently selected value */
  modelValue?: Value;
  /** Compare values by an object key or a custom equality function */
  by?: SelectBy<Value>;
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

type RuntimeSelectBy = string | number | symbol | ((left: unknown, right: unknown) => boolean);

function selectValuesEqual(
  left: unknown,
  right: unknown,
  by: RuntimeSelectBy | undefined,
): boolean {
  if (typeof by === "function") return by(left, right);
  if (
    by !== undefined &&
    typeof left === "object" &&
    left !== null &&
    typeof right === "object" &&
    right !== null
  ) {
    return Object.is(Reflect.get(left, by), Reflect.get(right, by));
  }
  return Object.is(left, right);
}

function optionKey(option: SelectOption<unknown>, index: number): string | number | symbol {
  if (option.key !== undefined) return option.key;
  const value = option.value;
  return typeof value === "string" || typeof value === "number" || typeof value === "symbol"
    ? value
    : index;
}

export const Select = defineComponent({
  name: "Select",
  props: {
    options: {
      type: Array as PropType<readonly SelectOption<unknown>[]>,
      required: true,
    },
    modelValue: null as unknown as PropType<unknown>,
    by: [String, Number, Symbol, Function] as PropType<RuntimeSelectBy>,
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
      [() => props.modelValue, () => props.by],
      (value) => {
        const selectedIndex = props.options.findIndex(
          (option) => selectValuesEqual(option.value, value[0], value[1]) && !option.disabled,
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
      const value = toRaw(option.value);
      emit("update:modelValue", value);
      emit("select", value);
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
        const isSelected = selectValuesEqual(option.value, props.modelValue, props.by);
        const indicator = isHighlighted ? props.indicator : props.indicatorEmpty;

        children.push(
          h(
            "box",
            {
              key: optionKey(option, index),
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
