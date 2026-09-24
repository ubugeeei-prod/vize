<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { mentionContext } from "./mention-context.ts";

const {
  as = "textarea",
  name = undefined,
  placeholder = undefined,
  rows = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Native text control to render.
   *
   * @default "textarea"
   */
  readonly as?: "input" | "textarea";

  /**
   * Form control name; the field submits its plain text natively.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Hint text shown while empty.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Visible text lines for the textarea.
   *
   * @default undefined
   */
  readonly rows?: number;

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
}>();

const context = mentionContext.use();
const textarea = useTemplateRef<HTMLTextAreaElement>("textarea");
const input = useTemplateRef<HTMLInputElement>("input");
const handlers = computed(() => ({
  onBlur: (event: FocusEvent) => context.onFieldBlur(event),
  onClick: (event: MouseEvent) => context.onFieldCaret(event),
  onFocus: (event: FocusEvent) => context.onFieldCaret(event),
  onInput: (event: Event) => context.onFieldInput(event),
  onKeydown: (event: KeyboardEvent) => context.onFieldKeydown(event),
  onKeyup: (event: KeyboardEvent) => {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") context.onFieldCaret(event);
  },
}));
const ariaProps = computed(() => ({
  "aria-activedescendant": context.activeDescendant.value,
  "aria-autocomplete": "list" as const,
  "aria-controls": context.open.value ? context.listboxId.value : undefined,
  "aria-describedby": ariaDescribedby,
  "aria-label": ariaLabel,
  "aria-labelledby": ariaLabelledby,
}));

function element(): HTMLTextAreaElement | HTMLInputElement | null {
  return as === "input" ? input.value : textarea.value;
}

onMounted(() => {
  context.attachField(element(), "text");
});

onUnmounted(() => {
  if (context.fieldElement.value === element()) context.attachField(null, "text");
});

defineExpose({ element, focus: (options?: FocusOptions) => element()?.focus(options) });
</script>

<template>
  <input
    v-if="as === 'input'"
    :id="context.fieldId.value"
    ref="input"
    v-bind="{ ...handlers, ...ariaProps }"
    type="text"
    role="combobox"
    autocomplete="off"
    :aria-expanded="context.open.value ? 'true' : 'false'"
    :name
    :placeholder
    :value="context.text.value"
    :disabled="context.disabled.value"
    data-vize-ui="mention-input"
    part="input"
    :data-state="context.state.value"
  />
  <textarea
    v-else
    :id="context.fieldId.value"
    ref="textarea"
    v-bind="{ ...handlers, ...ariaProps }"
    aria-haspopup="listbox"
    :name
    :rows
    :placeholder
    :value="context.text.value"
    :disabled="context.disabled.value"
    data-vize-ui="mention-input"
    part="input"
    :data-state="context.state.value"
  ></textarea>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
