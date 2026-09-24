<script setup lang="ts">
import { formWizardContext } from "./form-wizard-context.ts";

defineSlots<{
  /** Button contents; `isLast` lets consumers switch between "Next" and "Finish". */
  default(props: { readonly isLast: boolean; readonly validating: boolean }): unknown;
}>();

const context = formWizardContext.use();

function onClick(): void {
  void context.next();
}
</script>

<template>
  <button
    type="button"
    :disabled="context.validating.value"
    :aria-controls="`${context.baseId.value}-${context.current.value}`"
    part="next"
    data-vize-ui="form-wizard-next"
    :data-last="context.isLast.value ? 'true' : 'false'"
    @click="onClick"
  >
    <slot :is-last="context.isLast.value" :validating="context.validating.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
