<script setup lang="ts">
import { ref, computed } from "vue";
import { mdiLoading, mdiImageOutline } from "@mdi/js";
import { runVrt } from "../api";
import { isStaticGallery } from "../staticApi";
import MdiIcon from "./MdiIcon.vue";
import StaticVrtNotice from "./StaticVrtNotice.vue";
import type { VrtResult, VrtSummary, VrtArtifacts } from "./vrtResults";

const props = defineProps<{
  artPath: string;
  defaultVariantName?: string;
}>();

const isRunning = ref(false);
const hasRun = ref(false);
const results = ref<VrtResult[]>([]);
const summary = ref<VrtSummary | null>(null);
const error = ref<string | null>(null);
const updateSnapshots = ref(false);
const artifacts = ref<VrtArtifacts | null>(null);

const groupedResults = computed(() => {
  const groups: Record<string, VrtResult[]> = {};
  for (const r of results.value) {
    const key = r.variantName;
    if (!groups[key]) groups[key] = [];
    groups[key].push(r);
  }
  return groups;
});

async function runTest() {
  isRunning.value = true;
  error.value = null;

  try {
    const data = await runVrt(props.artPath, updateSnapshots.value);
    results.value = data.results;
    summary.value = data.summary;
    artifacts.value = data.artifacts ?? null;
    hasRun.value = true;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    isRunning.value = false;
  }
}

function getStatusIcon(result: VrtResult): string {
  if (result.error) return "error";
  if (result.isNew) return "new";
  if (result.passed) return "pass";
  return "fail";
}
</script>

<template>
  <div class="vrt-panel">
    <div class="vrt-header">
      <h3 class="vrt-title">Visual Regression Testing</h3>
      <div v-if="!isStaticGallery" class="vrt-actions">
        <label class="vrt-update-label">
          <input v-model="updateSnapshots" type="checkbox" class="vrt-checkbox" />
          Update snapshots
        </label>
        <button type="button" class="vrt-run-btn" :disabled="isRunning" @click="runTest">
          <MdiIcon v-if="isRunning" class="spin" :path="mdiLoading" :size="14" />
          <MdiIcon v-else :path="mdiImageOutline" :size="14" />
          {{ isRunning ? "Running..." : "Run VRT" }}
        </button>
      </div>
    </div>

    <StaticVrtNotice v-if="isStaticGallery" />
    <div v-else-if="error" class="vrt-error">
      <p>{{ error }}</p>
      <p class="vrt-hint">Make sure Playwright is installed: <code>npm install playwright</code></p>
    </div>

    <div v-else-if="!hasRun" class="vrt-empty">
      <p>Click "Run VRT" to capture and compare screenshots.</p>
      <p class="vrt-hint">Requires Playwright to be installed.</p>
    </div>

    <template v-else>
      <div v-if="summary" class="vrt-summary">
        <div class="vrt-stat total">
          <span class="vrt-stat-value">{{ summary.total }}</span>
          <span class="vrt-stat-label">Total</span>
        </div>
        <div class="vrt-stat passed">
          <span class="vrt-stat-value">{{ summary.passed }}</span>
          <span class="vrt-stat-label">Passed</span>
        </div>
        <div v-if="summary.failed > 0" class="vrt-stat failed">
          <span class="vrt-stat-value">{{ summary.failed }}</span>
          <span class="vrt-stat-label">Failed</span>
        </div>
        <div v-if="summary.new > 0" class="vrt-stat new">
          <span class="vrt-stat-value">{{ summary.new }}</span>
          <span class="vrt-stat-label">New</span>
        </div>
      </div>

      <div v-if="artifacts" class="vrt-artifacts">
        <div class="vrt-artifacts-header">Local artifacts</div>
        <div class="vrt-artifact">
          <span class="vrt-artifact-label">Reports</span>
          <code class="vrt-artifact-path">{{ artifacts.reportDir }}</code>
        </div>
        <div class="vrt-artifact">
          <span class="vrt-artifact-label">Snapshots</span>
          <code class="vrt-artifact-path">{{ artifacts.snapshotDir }}</code>
        </div>
        <div class="vrt-artifact">
          <span class="vrt-artifact-label">HTML report</span>
          <code class="vrt-artifact-path">{{ artifacts.htmlReportPath }}</code>
        </div>
        <div class="vrt-artifact">
          <span class="vrt-artifact-label">JSON report</span>
          <code class="vrt-artifact-path">{{ artifacts.jsonReportPath }}</code>
        </div>
      </div>

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
  </div>
</template>

<style scoped src="./vrtPanel.css"></style>
