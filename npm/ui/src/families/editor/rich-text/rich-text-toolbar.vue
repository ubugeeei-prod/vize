<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { richTextContext } from "./rich-text-context.ts";
import { useRichTextToolbar } from "./rich-text-toolbar-runtime.ts";

const { ariaLabel = "Formatting" } = defineProps<{
  /**
   * Accessible name of the toolbar.
   *
   * @default "Formatting"
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** RichTextToolbarButton children (and separators or groups). */
  default?(): unknown;
}>();

const editor = richTextContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const toolbar = useRichTextToolbar(editor, element);
const toolbarProps = computed(() => ({
  role: "toolbar",
  "aria-orientation": "horizontal" as const,
  onKeydown: toolbar.onKeydown,
}));

defineExpose({ element });
</script>

<template>
  <div
    ref="element"
    v-bind="toolbarProps"
    :aria-label="ariaLabel"
    :aria-controls="editor.contentId.value"
    data-vize-ui="rich-text-toolbar"
    part="toolbar"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
