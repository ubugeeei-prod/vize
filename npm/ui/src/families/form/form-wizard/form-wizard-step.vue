<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";

import { formWizardContext } from "./form-wizard-context.ts";

const { step, label = undefined } = defineProps<{
  /**
   * Step id this panel belongs to; must be one of the wizard `steps`.
   *
   * @default required
   */
  readonly step: string;

  /**
   * Accessible name of the step panel, for example its heading text.
   *
   * @default undefined
   */
  readonly label?: string;
}>();

defineSlots<{
  /** Step contents; rendered for every step so field state survives navigation. */
  default(props: { readonly active: boolean; readonly index: number }): unknown;
}>();

const context = formWizardContext.use();
const element = useTemplateRef<HTMLElement>("element");
const active = computed(() => context.current.value === step);
const index = computed(() => context.indexOf(step));

watch(element, (panel) => context.registerPanel(step, panel), { flush: "sync", immediate: true });
onScopeDispose(() => context.registerPanel(step, null));
</script>

<template>
  <section
    :id="`${context.baseId.value}-${step}`"
    ref="element"
    role="group"
    :aria-label="label"
    :aria-current="active ? 'step' : undefined"
    :hidden="!active"
    tabindex="-1"
    part="step"
    data-vize-ui="form-wizard-step"
    :data-step="step"
    :data-state="active ? 'active' : context.isVisited(step) ? 'visited' : 'upcoming'"
  >
    <slot :active="active" :index="index" />
  </section>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
