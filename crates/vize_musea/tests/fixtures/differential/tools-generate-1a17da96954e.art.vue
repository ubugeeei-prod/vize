<template>
  <section class="typed-resource">
    <header>
      <h1>{{ headline }}</h1>
      <p>{{ benchmarkToken }}</p>
      <button type="button" @click="cycleState">Cycle</button>
    </header>
    <article v-for="resource in decoratedResources" :key="resource.id" :data-kind="resource.state.kind">
      <h2>{{ resource.name }}</h2>
      <p>{{ resource.summary }}</p>
      <meter :value="resource.score" min="0" max="100"></meter>
      <button type="button" @click="select(resource)">Select</button>
      <template v-if="resource.state.kind === 'failed'">
        <strong>{{ resource.state.reason }}</strong>
      </template>
      <template v-else-if="resource.state.kind === 'ready'">
        <span>{{ resource.state.deployedAt }}</span>
      </template>
      <template v-else>
        <span>{{ resource.state.percent }}%</span>
      </template>
    </article>
    <footer>{{ selected?.name ?? 'None selected' }} / {{ totals.ready }} ready</footer>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

type ResourceState =
  | { kind: 'ready'; deployedAt: string }
  | { kind: 'pending'; percent: number }
  | { kind: 'failed'; reason: string }

type Resource<TMeta extends Record<string, unknown>> = {
  id: number
  name: string
  weight: number
  state: ResourceState
  meta: TMeta
}

type DecoratedResource<TMeta extends Record<string, unknown>> = Resource<TMeta> & {
  score: number
  summary: string
}

const emit = defineEmits<{
  selected: [payload: DecoratedResource<{ owner: string; tags: string[] }>]
}>()

const benchmarkToken = 'ts-heavy-__BENCH_ID__' as const
const headline = computed(() => 'Typed resources ' + benchmarkToken)
const selected = ref<DecoratedResource<{ owner: string; tags: string[] }> | null>(null)
const resources = ref<Resource<{ owner: string; tags: string[] }>[]>([
  { id: 1, name: 'Compiler', weight: 0.85, state: { kind: 'ready', deployedAt: '2026-01-01' }, meta: { owner: 'Core', tags: ['sfc', 'fast'] } },
  { id: 2, name: 'Checker', weight: 0.65, state: { kind: 'pending', percent: 42 }, meta: { owner: 'Types', tags: ['ts', 'diagnostics'] } },
  { id: 3, name: 'Linter', weight: 0.45, state: { kind: 'failed', reason: 'rule mismatch' }, meta: { owner: 'Lint', tags: ['rules'] } },
])

function scoreFor<TMeta extends Record<string, unknown>>(resource: Resource<TMeta>): number {
  const stateScore: Record<ResourceState['kind'], number> = { ready: 100, pending: 60, failed: 15 }
  return Math.round(stateScore[resource.state.kind] * resource.weight)
}

const decoratedResources = computed(() =>
  resources.value.map((resource): DecoratedResource<{ owner: string; tags: string[] }> => ({
    ...resource,
    score: scoreFor(resource),
    summary: resource.meta.owner + ' / ' + resource.meta.tags.join(', '),
  })),
)

const totals = computed(() => decoratedResources.value.reduce(
  (acc, resource) => {
    acc[resource.state.kind] += 1
    return acc
  },
  { ready: 0, pending: 0, failed: 0 } as Record<ResourceState['kind'], number>,
))

function select(resource: DecoratedResource<{ owner: string; tags: string[] }>): void {
  selected.value = resource
  emit('selected', resource)
}

function cycleState(): void {
  resources.value = resources.value.map((resource) => ({
    ...resource,
    state: resource.state.kind === 'ready'
      ? { kind: 'pending', percent: 20 }
      : resource.state.kind === 'pending'
        ? { kind: 'failed', reason: 'synthetic transition' }
        : { kind: 'ready', deployedAt: '2026-01-02' },
  }))
}
</script>

<style scoped>
.typed-resource { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 16px; }
.typed-resource header, .typed-resource footer { grid-column: 1 / -1; }
.typed-resource article { border: 1px solid #d1d5db; padding: 12px; }
</style>
