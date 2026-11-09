<script setup lang="ts" generic="Value = string">
import { computed } from "vue";

import { List as ListRuntime, type ListProps, type RuntimeListItem } from "./List.js";

const {
  items,
  modelValue,
  by,
  maxHeight,
  focused,
  indicator,
  indicatorEmpty,
  fg,
  selectedFg,
  selectedBg,
} = defineProps<ListProps<Value>>();

const emit = defineEmits<{
  /** Fired when keyboard activation requests a new controlled value. */
  "update:modelValue": [value: Value];
  /** Fired after the highlighted item is activated. */
  "select": [value: Value];
}>();

defineOptions({ name: "FrescoList" });

const runtimeProps = computed(() => {
  return {
    items: items as readonly RuntimeListItem[],
    modelValue,
    by:
      typeof by === "function"
        ? (left: unknown, right: unknown) => by(left as Value, right as Value)
        : (by as string | number | symbol | undefined),
    maxHeight,
    focused,
    indicator,
    indicatorEmpty,
    fg,
    selectedFg,
    selectedBg,
  };
});

function updateModelValue(value: unknown): void {
  emit("update:modelValue", value as Value);
}

function selectValue(value: unknown): void {
  emit("select", value as Value);
}
</script>

<template>
  <ListRuntime v-bind="runtimeProps" @update:model-value="updateModelValue" @select="selectValue" />
</template>
