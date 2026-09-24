<script setup lang="ts">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { comboboxContext } from "./combobox-context.ts";

const { id = undefined } = defineProps<{
  /**
   * Consumer-owned option id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;
}>();

defineSlots<{
  /** Create-option label, e.g. `Create "{{ query }}"`. Receives the typed text and highlight state. */
  default(props: { readonly query: string; readonly active: boolean }): unknown;
}>();

const context = comboboxContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const optionId = useDeterministicId({ id: () => id, hint: "combobox-create" });
const visible = computed(() => context.canCreate.value);
const active = computed(() => visible.value && context.createActive.value);
const handlers = computed(() => ({
  role: "option" as const,
  onClick: (event: MouseEvent) => {
    if (visible.value) context.create(event);
  },
  onPointermove: () => {
    if (visible.value && !active.value) context.highlightCreateOption();
  },
}));
let registration: CollectionRegistration<string> | null = null;

watch(
  [visible, optionId],
  () => {
    registration?.unregister();
    registration = visible.value
      ? context.registerCreateOption({ element: () => element.value, id: optionId })
      : null;
  },
  { flush: "sync", immediate: true },
);

onUnmounted(() => {
  registration?.unregister();
  registration = null;
});
</script>

<template>
  <div
    :id="optionId"
    ref="element"
    v-bind="handlers"
    aria-selected="false"
    :hidden="visible ? undefined : true"
    data-vize-ui="combobox-create-item"
    part="create-item"
    :data-highlighted="active ? 'true' : undefined"
  >
    <slot v-if="visible" :query="context.query.value" :active="active" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
