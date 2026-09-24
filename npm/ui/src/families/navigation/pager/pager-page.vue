<script setup lang="ts">
import { computed } from "vue";

import { pagerContext, pagerIds } from "./pager-context.ts";

const { page } = defineProps<{
  /**
   * Page id this panel renders; must be one of the Pager `pages`.
   *
   * @default required
   */
  readonly page: string;
}>();

defineSlots<{
  /** Page contents; stay mounted so swiping shows neighbors. */
  default(props: { readonly active: boolean }): unknown;
}>();

const context = pagerContext.use();
const ids = computed(() => pagerIds(context.baseId.value, page));
const active = computed(() => context.active.value === page);
</script>

<template>
  <section
    :id="ids.panel"
    role="tabpanel"
    :aria-labelledby="ids.tab"
    :inert="!active"
    part="page"
    data-vize-ui="pager-page"
    :data-page="page"
    :data-state="active ? 'active' : 'inactive'"
  >
    <slot :active="active" />
  </section>
</template>

<style scoped>
/* Headless by design. Consumer CSS: flex: 0 0 100%; scroll-snap-align: start. */
</style>
