<script setup lang="ts" generic="Value = string">
import { computed } from "vue";

import { Tree as TreeRuntime, type RuntimeTreeNode, type TreeProps } from "./Tree.js";

const {
  data,
  expanded,
  selected,
  by,
  focused,
  maxHeight,
  showLines,
  indent,
  expandedIcon,
  collapsedIcon,
  leafIcon,
  fg,
  selectedFg,
} = defineProps<TreeProps<Value>>();

const emit = defineEmits<{
  /** Fired after the highlighted node is activated. */
  "select": [value: Value];
  /** Fired when keyboard activation requests a new controlled selection. */
  "update:selected": [value: Value];
  /** Fired when one expansion key changes. */
  "toggle": [key: string, expanded: boolean];
  /** Fired when expansion requests a new controlled key list. */
  "update:expanded": [keys: string[]];
}>();

defineOptions({ name: "FrescoTree" });

const runtimeProps = computed(() => {
  return {
    data: data as readonly RuntimeTreeNode[],
    expanded,
    selected,
    by:
      typeof by === "function"
        ? (left: unknown, right: unknown) => by(left as Value, right as Value)
        : (by as string | number | symbol | undefined),
    focused,
    maxHeight,
    showLines,
    indent,
    expandedIcon,
    collapsedIcon,
    leafIcon,
    fg,
    selectedFg,
  };
});

function selectValue(value: unknown): void {
  emit("select", value as Value);
}

function updateSelected(value: unknown): void {
  emit("update:selected", value as Value);
}

function toggle(key: string, expanded: boolean): void {
  emit("toggle", key, expanded);
}

function updateExpanded(keys: string[]): void {
  emit("update:expanded", keys);
}
</script>

<template>
  <TreeRuntime
    v-bind="runtimeProps"
    @select="selectValue"
    @update:selected="updateSelected"
    @toggle="toggle"
    @update:expanded="updateExpanded"
  />
</template>
