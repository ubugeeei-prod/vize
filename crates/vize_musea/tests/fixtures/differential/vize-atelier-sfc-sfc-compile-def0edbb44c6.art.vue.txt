<template>
  <section class="dashboard">
    <header class="dashboard-header">
      <div>
        <p class="eyebrow">{{ eyebrow }}</p>
        <h1>{{ title }}</h1>
      </div>
      <button @click="refresh" :disabled="loading" class="refresh">
        {{ loading ? 'Refreshing…' : 'Refresh' }}
      </button>
    </header>

    <div class="summary-grid">
      <article
        v-for="metric in metrics"
        :key="metric.id"
        class="metric-card"
        :style="{ borderColor: metric.color }"
      >
        <span class="metric-label">{{ metric.label }}</span>
        <strong class="metric-value">{{ metric.value }}</strong>
        <em class="metric-change">{{ metric.change }}</em>
      </article>
    </div>

    <section v-if="visibleProjects.length" class="projects">
      <article v-for="project in visibleProjects" :key="project.id" class="project-card">
        <header>
          <h2>{{ project.name }}</h2>
          <span>{{ project.owner }}</span>
        </header>
        <p>{{ project.summary }}</p>
        <ul>
          <li v-for="tag in project.tags" :key="tag">{{ tag }}</li>
        </ul>
      </article>
    </section>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

interface Metric {
  id: number
  label: string
  value: string
  change: string
  color: string
}

interface Project {
  id: number
  name: string
  owner: string
  summary: string
  tags: string[]
  featured: boolean
}

const eyebrow = ref('Operations')
const title = ref('System overview')
const loading = ref(false)
const showFeaturedOnly = ref(false)

const metrics = ref<Metric[]>([
  { id: 1, label: 'Deployments', value: '128', change: '+12%', color: '#22c55e' },
  { id: 2, label: 'Latency', value: '84ms', change: '-9%', color: '#3b82f6' },
  { id: 3, label: 'Errors', value: '3', change: '-42%', color: '#ef4444' },
])

const projects = ref<Project[]>([
  {
    id: 1,
    name: 'Atlas',
    owner: 'Platform',
    summary: 'Core dashboard application',
    tags: ['dashboard', 'vue', 'rust'],
    featured: true,
  },
  {
    id: 2,
    name: 'Comet',
    owner: 'Growth',
    summary: 'Acquisition experiments and reporting',
    tags: ['growth', 'analytics'],
    featured: false,
  },
  {
    id: 3,
    name: 'Beacon',
    owner: 'Infra',
    summary: 'Realtime service health explorer',
    tags: ['infra', 'realtime'],
    featured: true,
  },
])

const visibleProjects = computed(() =>
  showFeaturedOnly.value
    ? projects.value.filter((project) => project.featured)
    : projects.value,
)

function refresh() {
  loading.value = !loading.value
}
</script>

<style scoped>
.dashboard {
  display: grid;
  gap: 20px;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
}

.metric-card,
.project-card {
  border: 1px solid #d4d4d8;
  border-radius: 16px;
  padding: 16px;
}
</style>
