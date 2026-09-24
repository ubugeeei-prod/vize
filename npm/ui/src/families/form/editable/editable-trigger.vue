<script setup lang="ts">
import { computed } from "vue";

import { editableContext } from "./editable-context.ts";
import type { EditableTriggerAction } from "./editable-types.ts";

const { action, ariaLabel = undefined } = defineProps<{
  /**
   * What activation does: enter edit mode, submit the draft, or cancel it.
   *
   * @default required
   */
  readonly action: EditableTriggerAction;

  /**
   * Accessible name when the contents are icon-only.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Trigger contents with the current editing state. */
  default?(props: { readonly editing: boolean }): unknown;
}>();

const context = editableContext.use();
// Edit triggers show in preview mode; submit and cancel triggers show while editing.
const shown = computed(() => (action === "edit" ? !context.editing.value : context.editing.value));

function onPointerdown(event: PointerEvent): void {
  // Keep focus in the input so blur-submit does not race submit/cancel clicks.
  if (event.button === 0 && action !== "edit") event.preventDefault();
}

function onClick(): void {
  if (action === "edit") context.edit();
  else if (action === "submit") context.submit();
  else context.cancel();
}
</script>

<template>
  <button
    type="button"
    :hidden="!shown"
    :disabled="!context.interactive.value"
    :aria-label="ariaLabel"
    :aria-controls="context.inputId.value"
    :part="`${action}-trigger`"
    data-vize-ui="editable-trigger"
    :data-action="action"
    @pointerdown="onPointerdown"
    @click="onClick"
  >
    <slot :editing="context.editing.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
