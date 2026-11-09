<script setup lang="ts" generic="Value = string">
import { computed } from "vue";

import { Menu as MenuRuntime, type MenuProps, type RuntimeMenuItem } from "./Menu.js";

const {
  items,
  focusedIndex,
  focused,
  maxHeight,
  border,
  width,
  fg,
  focusedFg,
  focusedBg,
  shortcutFg,
} = defineProps<MenuProps<Value>>();

const emit = defineEmits<{
  /** Fired after the highlighted command is activated. */
  "select": [value: Value];
  /** Fired when keyboard navigation requests a new focused item index. */
  "update:focusedIndex": [index: number];
}>();

defineOptions({ name: "FrescoMenu" });

const runtimeProps = computed(() => ({
  items: items as readonly RuntimeMenuItem[],
  focusedIndex,
  focused,
  maxHeight,
  border,
  width,
  fg,
  focusedFg,
  focusedBg,
  shortcutFg,
}));

function selectValue(value: unknown): void {
  emit("select", value as Value);
}

function updateFocusedIndex(index: number): void {
  emit("update:focusedIndex", index);
}
</script>

<template>
  <MenuRuntime
    v-bind="runtimeProps"
    @select="selectValue"
    @update:focused-index="updateFocusedIndex"
  />
</template>
