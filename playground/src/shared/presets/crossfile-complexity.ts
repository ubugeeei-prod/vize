// CrossFile preset: Component-tree complexity
// Nested template flow, scoped slots, props, provide/inject, and reactive graph signals

import type { Preset } from "./crossfile";

export const COMPLEXITY_PRESET: Omit<Preset, "icon"> = {
  id: "component-complexity",
  name: "Complexity",
  description: "Component-tree template and reactivity complexity",
  files: {
    "App.vue": `<script setup lang="ts">
import { computed, provide, ref } from 'vue'
import DashboardShell from './DashboardShell.vue'

const ready = ref(true)
const featureFlags = ref({ cohort: 'beta', realtime: true })
const sections = ref([
  { id: 'revenue', title: 'Revenue', enabled: true, metrics: [12, 18, 31] },
  { id: 'retention', title: 'Retention', enabled: true, metrics: [74, 82, 88] },
])
const activeSections = computed(() => sections.value.filter((section) => section.enabled))

provide('featureFlags', featureFlags)
</script>

<template>
  <DashboardShell
    v-if="ready && activeSections.length > 0"
    v-for="section in activeSections"
    :key="section.id"
    :section="section"
    :flags="featureFlags"
  >
    <template #metric="{ metric, index }">
      <strong v-if="featureFlags.realtime || index === 0">{{ metric }}</strong>
    </template>
  </DashboardShell>
</template>`,

    "DashboardShell.vue": `<script setup lang="ts">
import DashboardPanel from './DashboardPanel.vue'

const props = defineProps<{
  section: { id: string; title: string; metrics: number[] }
  flags: { cohort: string; realtime: boolean }
}>()
</script>

<template>
  <DashboardPanel
    v-if="props.flags.realtime"
    :title="props.section.title"
    :metrics="props.section.metrics"
    :cohort="props.flags.cohort"
  >
    <template #metric="{ metric, index }">
      <slot name="metric" :metric="metric" :index="index" />
    </template>
  </DashboardPanel>
</template>`,

    "DashboardPanel.vue": `<script setup lang="ts">
import { computed, inject } from 'vue'
import MetricList from './MetricList.vue'

const props = defineProps<{
  title: string
  metrics: number[]
  cohort: string
}>()

const flags = inject<{ realtime: boolean }>('featureFlags')
const visibleMetrics = computed(() => props.metrics.filter((metric) => metric > 10))
</script>

<template>
  <section v-if="flags?.realtime && visibleMetrics.length">
    <h2>{{ title }} / {{ cohort }}</h2>
    <MetricList :items="visibleMetrics">
      <template #default="{ item, index }">
        <slot name="metric" :metric="item" :index="index" />
      </template>
    </MetricList>
  </section>
</template>`,

    "MetricList.vue": `<script setup lang="ts">
defineProps<{
  items: number[]
}>()
</script>

<template>
  <ol>
    <li v-for="(item, index) in items" :key="index">
      <slot :item="item" :index="index" />
    </li>
  </ol>
</template>`,
  },
};
