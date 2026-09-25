<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";

import { pagerContext, pagerIds } from "./pager-context.ts";

const { page } = defineProps<{
  /**
   * Page this segment selects; must be one of the Pager `pages`.
   *
   * @default required
   */
  readonly page: string;
}>();

defineSlots<{
  /** Segment label with its selection state. */
  default(props: { readonly selected: boolean }): unknown;
}>();

const context = pagerContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const ids = computed(() => pagerIds(context.baseId.value, page));
const selected = computed(() => context.active.value === page);

watch(element, (target) => context.registerTab(page, target), { flush: "sync", immediate: true });
onScopeDispose(() => context.registerTab(page, null));

function onClick(): void {
  context.select(page, "tab");
}

// Roving tab stop: only the selected segment is in the tab order.
const tabProps = computed(() => ({
  role: "tab",
  tabindex: selected.value ? 0 : -1,
  "aria-selected": selected.value ? ("true" as const) : ("false" as const),
  "aria-controls": ids.value.panel,
}));
</script>

<template>
  <button
    :id="ids.tab"
    ref="element"
    type="button"
    v-bind="tabProps"
    part="tab"
    data-vize-ui="pager-tab"
    :data-page="page"
    :data-state="selected ? 'active' : 'inactive'"
    @click="onClick"
  >
    <slot :selected="selected" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
