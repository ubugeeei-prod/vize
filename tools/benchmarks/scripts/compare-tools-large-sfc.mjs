/**
 * Corpus generator for the large-SFC surfaces of tools/benchmarks/scripts/compare-tools.mjs.
 *
 * The `large` task compiles, lints, formats and type-checks one deliberately
 * oversized single-file component instead of many small ones, so its input is
 * a generated source rather than a copied fixture. It lives here rather than
 * beside the measurement code because it is pure text generation with no
 * dependency on the benchmark harness, and because tools/benchmarks/scripts/compare-tools.mjs is
 * long past the repository per-file line budget.
 */

export function createLargeSfcSource(blockCount) {
  const blocks = [];
  for (let i = 0; i < blockCount; i++) {
    const metricIndex = i % 64;
    blocks.push(`    <article class="metric-card metric-card-${i}" :class="{ active: selectedId === ${metricIndex} }" :data-index="${i}">
      <header>
        <p>{{ labels[${metricIndex}] }}</p>
        <h2>{{ formatMetric(metrics[${metricIndex}], ${i}) }}</h2>
      </header>
      <dl>
        <div>
          <dt>Score</dt>
          <dd>{{ metrics[${metricIndex}].score }}</dd>
        </div>
        <div>
          <dt>Status</dt>
          <dd>{{ metrics[${metricIndex}].active ? "active" : "idle" }}</dd>
        </div>
      </dl>
      <ul>
        <li v-for="point in metrics[${metricIndex}].points" :key="'${i}-' + point.id">
          <span>{{ point.label }}</span>
          <strong>{{ point.value + ${i} }}</strong>
        </li>
      </ul>
      <button type="button" @click="selectMetric(${metricIndex})">Select {{ labels[${metricIndex}] }}</button>
    </article>`);
  }

  return `<template>
  <main class="large-dashboard">
    <section class="summary">
      <h1>{{ title }}</h1>
      <p>{{ activeCount }} active metrics across {{ metrics.length }} tracked rows.</p>
    </section>
${blocks.join("\n")}
  </main>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

type Point = {
  id: string
  label: string
  value: number
}

type Metric = {
  id: number
  title: string
  score: number
  active: boolean
  points: Point[]
}

const title = ref('Large synthetic dashboard')
const selectedId = ref(0)
const metrics = ref<Metric[]>(Array.from({ length: 64 }, (_, index) => ({
  id: index,
  title: 'Metric ' + index,
  score: (index * 13) % 100,
  active: index % 3 === 0,
  points: Array.from({ length: 4 }, (__, pointIndex) => ({
    id: index + '-' + pointIndex,
    label: 'Point ' + pointIndex,
    value: index * pointIndex,
  })),
})))

const labels = computed(() => metrics.value.map((metric) => metric.title + ' / ' + metric.score))
const activeCount = computed(() => metrics.value.filter((metric) => metric.active).length)

function formatMetric(metric: Metric, offset: number): string {
  return metric.title + ' #' + offset + ' (' + metric.score + ')'
}

function selectMetric(index: number): void {
  selectedId.value = index
}
</script>

<style scoped>
.large-dashboard {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 12px;
}
.summary {
  grid-column: 1 / -1;
}
.metric-card {
  border: 1px solid #d4d4d8;
  padding: 12px;
}
.metric-card.active {
  border-color: #2563eb;
}
</style>
`;
}
