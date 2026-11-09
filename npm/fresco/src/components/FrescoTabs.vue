<script setup lang="ts" generic="Value = string">
import { computed } from "vue";

import { Tabs as TabsRuntime, type RuntimeTab, type TabsProps } from "./Tabs.js";

const {
  tabs,
  modelValue,
  by,
  focused,
  maxItems,
  position,
  separator,
  activeFg,
  activeBg,
  inactiveFg,
  underline,
} = defineProps<TabsProps<Value>>();

const emit = defineEmits<{
  /** Fired when keyboard activation requests a new controlled value. */
  "update:modelValue": [value: Value];
  /** Fired after the highlighted tab is activated. */
  "change": [value: Value];
}>();

defineSlots<{ default?: () => unknown }>();

defineOptions({ name: "FrescoTabs" });

const runtimeProps = computed(() => {
  return {
    tabs: tabs as readonly RuntimeTab[],
    modelValue,
    by:
      typeof by === "function"
        ? (left: unknown, right: unknown) => by(left as Value, right as Value)
        : (by as string | number | symbol | undefined),
    focused,
    maxItems,
    position,
    separator,
    activeFg,
    activeBg,
    inactiveFg,
    underline,
  };
});

function updateModelValue(value: unknown): void {
  emit("update:modelValue", value as Value);
}

function changeValue(value: unknown): void {
  emit("change", value as Value);
}
</script>

<template>
  <TabsRuntime v-bind="runtimeProps" @update:model-value="updateModelValue" @change="changeValue">
    <slot />
  </TabsRuntime>
</template>
