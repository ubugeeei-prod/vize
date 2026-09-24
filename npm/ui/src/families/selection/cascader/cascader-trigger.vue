<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { cascaderContext } from "./cascader-context.ts";

const {
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Accessible name when no visible label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids of visible labels.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the cascader.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Trigger content, usually `CascaderValue`. Receives open and emptiness state. */
  default?(props: { readonly open: boolean; readonly empty: boolean }): unknown;
}>();

const context = cascaderContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const interactiveProps = computed(() => ({
  role: "combobox" as const,
  onClick: (event: MouseEvent) => {
    if (!context.disabled.value) context.setOpen(!context.open.value, event);
  },
  onKeydown: (event: KeyboardEvent) => context.onTriggerKeydown(event),
}));

onMounted(() => {
  context.triggerElement.value = element.value;
});

onUnmounted(() => {
  if (context.triggerElement.value === element.value) context.triggerElement.value = null;
});

defineExpose({ element, focus: (options?: FocusOptions) => element.value?.focus(options) });
</script>

<template>
  <button
    :id="context.triggerId.value"
    ref="element"
    v-bind="interactiveProps"
    type="button"
    :disabled="context.disabled.value"
    aria-haspopup="listbox"
    :aria-expanded="context.open.value ? 'true' : 'false'"
    :aria-controls="context.open.value ? context.columnIds.value.join(' ') : undefined"
    :aria-activedescendant="context.activeDescendant.value"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-required="context.required.value ? 'true' : undefined"
    data-vize-ui="cascader-trigger"
    part="trigger"
    :data-state="context.state.value"
    :data-placeholder="context.selectedText.value.length === 0 ? 'true' : undefined"
  >
    <slot :open="context.open.value" :empty="context.selectedText.value.length === 0" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
