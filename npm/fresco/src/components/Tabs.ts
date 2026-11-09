/**
 * Tabs Component - Tab navigation
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

interface TabBase {
  key: string;
  label: string;
  disabled?: boolean;
}

export type Tab<Value = string> = TabBase &
  ([Value] extends [string] ? { value?: Value } : { value: Value });

export type TabsValueKey<Value> = CollectionValueKey<Value>;
export type TabsBy<Value> = CollectionBy<Value>;

export interface TabsProps<Value = string> {
  /** Tab definitions */
  tabs: readonly Tab<Value>[];
  /** Currently active tab value */
  modelValue?: Value;
  /** Compare active values by an object key or a custom equality function */
  by?: TabsBy<Value>;
  /** Whether the tab list owns keyboard input */
  focused?: boolean;
  /** Maximum visible tabs before the tab strip scrolls */
  maxItems?: number;
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

export interface RuntimeTab extends TabBase {
  value?: unknown;
}

function tabValue(tab: RuntimeTab): unknown {
  return Object.hasOwn(tab, "value") ? tab.value : tab.key;
}

export const Tabs = defineComponent({
  name: "Tabs",
  props: {
    tabs: {
      type: Array as PropType<readonly RuntimeTab[]>,
      required: true,
    },
    modelValue: null as unknown as PropType<unknown>,
    by: [String, Number, Symbol, Function] as PropType<RuntimeCollectionBy>,
    focused: {
      type: Boolean,
      default: false,
    },
    maxItems: Number,
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
    const scrollOffset = ref(0);
    const highlightedIndex = ref(-1);
    const typeahead = createTypeaheadBuffer();
    const viewportSize = computed(() => {
      if (props.maxItems === undefined || !Number.isFinite(props.maxItems)) return undefined;
      return props.maxItems > 0 ? Math.max(1, Math.floor(props.maxItems)) : undefined;
    });

    const syncHighlight = () => {
      const activeIndex = props.tabs.findIndex(
        (tab) => collectionValuesEqual(tabValue(tab), props.modelValue, props.by) && !tab.disabled,
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

    watch([() => props.modelValue, () => props.by, () => props.tabs], syncHighlight, {
      deep: true,
      immediate: true,
    });
    watch(
      [highlightedIndex, viewportSize, () => props.tabs.length],
      ([index, windowSize]) => {
        scrollOffset.value = visibleCollectionOffset(
          index,
          scrollOffset.value,
          windowSize,
          props.tabs.length,
        );
      },
      { immediate: true },
    );

    const activate = () => {
      const tab = props.tabs[highlightedIndex.value];
      if (!tab || tab.disabled) return;
      const value = toRaw(tabValue(tab));
      emit("update:modelValue", value);
      emit("change", value);
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

    const visibleTabs = computed(() => {
      const windowSize = viewportSize.value;
      const start = windowSize === undefined ? 0 : scrollOffset.value;
      const end = windowSize === undefined ? props.tabs.length : start + windowSize;
      return props.tabs.slice(start, end).map((tab, visibleIndex) => ({
        tab,
        index: start + visibleIndex,
      }));
    });

    const scrollIndicator = computed(() => {
      const windowSize = viewportSize.value;
      if (windowSize === undefined || props.tabs.length <= windowSize) {
        return { showBefore: false, showAfter: false };
      }
      return {
        showBefore: scrollOffset.value > 0,
        showAfter: scrollOffset.value + windowSize < props.tabs.length,
      };
    });

    return () => {
      // Tab bar
      const tabItems: VNode[] = [];
      if (scrollIndicator.value.showBefore) {
        tabItems.push(h("text", { key: "scroll-before", dim: true }, "…"));
      }

      visibleTabs.value.forEach(({ tab, index }, visibleIndex) => {
        if (visibleIndex > 0) {
          tabItems.push(h("text", { key: `sep-${index}`, dim: true }, props.separator));
        }

        const isActive = collectionValuesEqual(tabValue(tab), props.modelValue, props.by);
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
              "data-tab-index": index,
            },
            tab.label,
          ),
        );
      });

      if (scrollIndicator.value.showAfter) {
        tabItems.push(h("text", { key: "scroll-after", dim: true }, "…"));
      }

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
