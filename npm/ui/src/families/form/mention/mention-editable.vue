<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { mentionContext } from "./mention-context.ts";

const {
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  multiline = true,
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
   * Space-separated ids that describe the field.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Announce the editor as multi-line.
   *
   * @default true
   */
  readonly multiline?: boolean;
}>();

defineSlots<{
  /** Initial editor content rendered on the server and adopted on hydration. */
  default?(): unknown;
}>();

const context = mentionContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const props = computed(() => ({
  "aria-activedescendant": context.activeDescendant.value,
  "aria-autocomplete": "list" as const,
  "aria-controls": context.open.value ? context.listboxId.value : undefined,
  "aria-describedby": ariaDescribedby,
  "aria-disabled": context.disabled.value ? ("true" as const) : undefined,
  "aria-haspopup": "listbox" as const,
  "aria-label": ariaLabel,
  "aria-labelledby": ariaLabelledby,
  "aria-multiline": multiline ? ("true" as const) : ("false" as const),
  contenteditable: context.disabled.value ? ("false" as const) : ("true" as const),
  role: "textbox" as const,
  tabindex: 0 as const,
  onBlur: (event: FocusEvent) => context.onFieldBlur(event),
  onClick: (event: MouseEvent) => context.onFieldCaret(event),
  onFocus: (event: FocusEvent) => context.onFieldCaret(event),
  onInput: (event: Event) => context.onFieldInput(event),
  onKeydown: (event: KeyboardEvent) => context.onFieldKeydown(event),
  onKeyup: (event: KeyboardEvent) => {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") context.onFieldCaret(event);
  },
}));

onMounted(() => {
  context.attachField(element.value, "editable");
});

onUnmounted(() => {
  if (context.fieldElement.value === element.value) context.attachField(null, "editable");
});

defineExpose({ element, focus: (options?: FocusOptions) => element.value?.focus(options) });
</script>

<template>
  <div
    :id="context.fieldId.value"
    ref="element"
    v-bind="props"
    data-vize-ui="mention-editable"
    part="editable"
    :data-state="context.state.value"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
