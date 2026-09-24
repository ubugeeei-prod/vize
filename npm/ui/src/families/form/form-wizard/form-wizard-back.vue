<script setup lang="ts">
import { formWizardContext } from "./form-wizard-context.ts";

defineSlots<{
  /** Button contents. */
  default(props: { readonly isFirst: boolean }): unknown;
}>();

const context = formWizardContext.use();

function onClick(): void {
  context.back();
}
</script>

<template>
  <button
    type="button"
    :disabled="context.isFirst.value || context.validating.value"
    :aria-controls="`${context.baseId.value}-${context.current.value}`"
    part="back"
    data-vize-ui="form-wizard-back"
    @click="onClick"
  >
    <slot :is-first="context.isFirst.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
