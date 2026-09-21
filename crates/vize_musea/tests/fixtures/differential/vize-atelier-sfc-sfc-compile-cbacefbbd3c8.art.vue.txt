<template>
  <div class="app-shell">
    <header class="topbar">
      <button @click="toggleMenu" class="menu-trigger">{{ menuLabel }}</button>
      <nav class="tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          :class="{ active: tab.id === activeTab }"
          @click="activeTab = tab.id"
        >
          {{ tab.label }}
        </button>
      </nav>
    </header>
    <main class="content">
      <article v-if="activePanel" class="panel">
        <h2>{{ activePanel.title }}</h2>
        <p>{{ activePanel.description }}</p>
        <ul>
          <li v-for="item in activePanel.items" :key="item.id">{{ item.label }}</li>
        </ul>
      </article>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

type PanelItem = { id: number; label: string }
type Panel = { id: string; title: string; description: string; items: PanelItem[] }

const menuLabel = ref('Open menu')
const activeTab = ref('overview')
const tabs = ref([
  { id: 'overview', label: 'Overview' },
  { id: 'usage', label: 'Usage' },
  { id: 'api', label: 'API' },
])

const panels = ref<Panel[]>([
  {
    id: 'overview',
    title: 'Overview',
    description: 'A compact compile benchmark for script setup.',
    items: [
      { id: 1, label: 'Fast parsing' },
      { id: 2, label: 'Inline template output' },
    ],
  },
  {
    id: 'usage',
    title: 'Usage',
    description: 'Exercises v-for, v-if, and event handlers.',
    items: [
      { id: 3, label: 'Stateful tabs' },
      { id: 4, label: 'Computed panel lookup' },
    ],
  },
])

const activePanel = computed(() => panels.value.find((panel) => panel.id === activeTab.value))

function toggleMenu() {
  menuLabel.value = menuLabel.value === 'Open menu' ? 'Close menu' : 'Open menu'
}
</script>

<style scoped>
.app-shell {
  display: grid;
  gap: 16px;
}

.tabs {
  display: flex;
  gap: 8px;
}

.panel {
  border: 1px solid #ddd;
  padding: 16px;
}
</style>
