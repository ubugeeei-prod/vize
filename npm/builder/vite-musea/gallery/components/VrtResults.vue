<script setup lang="ts">
import { computed } from "vue";
import type { VrtResult } from "./vrtResults";

const props = defineProps<{ results: VrtResult[] }>();

const groupedResults = computed(() => {
  const groups: Record<string, VrtResult[]> = Object.create(null);
  for (const r of props.results) {
    const key = r.variantName;
    if (!groups[key]) groups[key] = [];
    groups[key].push(r);
  }
  return groups;
});

function getStatusIcon(result: VrtResult): string {
  if (result.error) return "error";
  if (result.isNew) return "new";
  if (result.passed) return "pass";
  return "fail";
}
</script>

<template>
  <div class="vrt-results">
    <div
      v-for="(variantResults, variantName) in groupedResults"
      :key="variantName"
      class="vrt-variant"
    >
      <div class="vrt-variant-name">{{ variantName }}</div>
      <div class="vrt-viewports">
        <div
          v-for="result in variantResults"
          :key="result.viewport"
          class="vrt-viewport"
          :class="getStatusIcon(result)"
        >
          <span class="vrt-viewport-name">{{ result.viewport }}</span>
          <div class="vrt-viewport-body">
            <figure v-for="(url, kind) in result.images" :key="url">
              <figcaption>{{ kind }}</figcaption>
              <img :src="url" :alt="`${result.variantName} ${kind}`" class="vrt-hosted-image" />
            </figure>
            <span class="vrt-status" :class="getStatusIcon(result)">
              <template v-if="result.error">Error</template>
              <template v-else-if="result.isNew">New baseline</template>
              <template v-else-if="result.passed">Pass</template>
              <template v-else> Diff {{ result.diffPercentage?.toFixed(2) }}% </template>
            </span>
            <code
              v-if="result.diffPath || result.currentPath || result.snapshotPath"
              class="vrt-result-path"
            >
              {{ result.diffPath || result.currentPath || result.snapshotPath }}
            </code>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.vrt-status.pass {
  color: var(--musea-success);
}

.vrt-status.fail,
.vrt-status.error {
  color: var(--musea-error);
}

.vrt-status.new {
  color: var(--musea-info);
}

.vrt-results {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.vrt-variant {
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  padding: 0.75rem;
}

.vrt-variant-name {
  font-weight: 600;
  font-size: 0.8125rem;
  margin-bottom: 0.5rem;
}

.vrt-viewports {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.vrt-viewport {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  padding: 0.25rem 0.5rem;
  background: var(--musea-bg-tertiary);
  border-radius: var(--musea-radius-sm);
  font-size: 0.75rem;
}

.vrt-viewport-name {
  color: var(--musea-text-secondary);
}

.vrt-viewport-body {
  display: grid;
  gap: 0.25rem;
}

.vrt-status {
  font-weight: 600;
  font-size: 0.6875rem;
}

.vrt-result-path {
  color: var(--musea-text-muted);
  font-size: 0.6875rem;
  font-family: var(--musea-font-mono);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.vrt-hosted-image {
  max-width: 240px;
  height: auto;
}
</style>
