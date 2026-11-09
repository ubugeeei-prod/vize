<script setup lang="ts" generic="Value = string">
import { computed } from "vue";

import { Select as SelectRuntime, type SelectOption, type SelectProps } from "./Select.js";

const {
  options,
  modelValue,
  by,
  placeholder,
  maxHeight,
  focused,
  indicator,
  indicatorEmpty,
  fg,
  bg,
  selectedFg,
  selectedBg,
} = defineProps<SelectProps<Value>>();

const emit = defineEmits<{
  /** Fired when keyboard activation requests a new controlled value. */
  "update:modelValue": [value: Value];
  /** Fired after the highlighted option is activated. */
  "select": [value: Value];
}>();

defineOptions({ name: "FrescoSelect" });

const runtimeProps = computed(() => {
  return {
    options: options as readonly SelectOption<unknown>[],
    modelValue,
    by:
      typeof by === "function"
        ? (left: unknown, right: unknown) => by(left as Value, right as Value)
        : (by as string | number | symbol | undefined),
    placeholder,
    maxHeight,
    focused,
    indicator,
    indicatorEmpty,
    fg,
    bg,
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
  <SelectRuntime
    v-bind="runtimeProps"
    @update:model-value="updateModelValue"
    @select="selectValue"
  />
</template>
