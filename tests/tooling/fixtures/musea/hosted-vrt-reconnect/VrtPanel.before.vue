<script setup lang="ts">
import { ref } from "vue";
import { mdiLoading, mdiImageOutline } from "@mdi/js";
import { runVrt } from "../api";
import { isStaticGallery } from "../staticApi";
import MdiIcon from "./MdiIcon.vue";
import StaticVrtNotice from "./StaticVrtNotice.vue";
import HostedVrtConnection from "./HostedVrtConnection.vue";
import HostedVrtReports from "./HostedVrtReports.vue";
import VrtResults from "./VrtResults.vue";
import { useHostedVrt } from "../composables/useHostedVrt";
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
const hosted = useHostedVrt();

async function runTest() {
  isRunning.value = true;
  error.value = null;

  try {
    const data = isStaticGallery
      ? await hosted.run(props.artPath, updateSnapshots.value)
      : await runVrt(props.artPath, updateSnapshots.value);
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
</script>

<template>
  <div class="vrt-panel">
    <div class="vrt-header">
      <h3 class="vrt-title">Visual Regression Testing</h3>
      <div v-if="!isStaticGallery || hosted.connected.value" class="vrt-actions">
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

    <HostedVrtConnection
      v-if="isStaticGallery"
      :connected="hosted.connected.value"
      :error="hosted.connectionError.value"
      @connect="hosted.connect"
    />
    <StaticVrtNotice v-if="isStaticGallery && !hosted.connected.value" />
    <div v-else-if="error" class="vrt-error">
      <p>{{ error }}</p>
      <p v-if="!isStaticGallery" class="vrt-hint">
        Make sure Playwright is installed: <code>npm install playwright</code>
      </p>
    </div>

    <div v-else-if="!hasRun" class="vrt-empty">
      <p>Click "Run VRT" to capture and compare screenshots.</p>
      <p v-if="!isStaticGallery" class="vrt-hint">Requires Playwright to be installed.</p>
    </div>

    <template v-else>
      <HostedVrtReports v-if="hosted.reports.value" @download="hosted.download" />
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

      <VrtResults :results />
    </template>
  </div>
</template>

<style scoped>
.vrt-panel {
  padding: 0.5rem;
}

.vrt-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.vrt-title {
  font-size: 0.875rem;
  font-weight: 600;
}

.vrt-actions {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.vrt-update-label {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.75rem;
  color: var(--musea-text-muted);
  cursor: pointer;
}

.vrt-checkbox {
  width: 14px;
  height: 14px;
  cursor: pointer;
}

.vrt-run-btn {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.75rem;
  background: var(--musea-accent);
  border: none;
  border-radius: var(--musea-radius-sm);
  color: var(--musea-accent-contrast);
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--musea-transition);
}

.vrt-run-btn:hover:not(:disabled) {
  background: var(--musea-accent-hover);
}

.vrt-run-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.vrt-empty {
  padding: 2rem;
  text-align: center;
  color: var(--musea-text-muted);
  font-size: 0.875rem;
}

.vrt-hint {
  font-size: 0.75rem;
  margin-top: 0.5rem;
  opacity: 0.7;
}

.vrt-hint {
  code {
    background: var(--musea-bg-tertiary);
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    font-family: var(--musea-font-mono);
  }
}

.vrt-error {
  padding: 1rem;
  background: color-mix(in srgb, var(--musea-error) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--musea-error) 20%, transparent);
  border-radius: var(--musea-radius-sm);
  color: var(--musea-error);
  font-size: 0.8125rem;
}

.vrt-summary {
  display: flex;
  gap: 0.75rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}

.vrt-artifacts {
  display: grid;
  gap: 0.625rem;
  margin-bottom: 1rem;
  padding: 0.875rem 1rem;
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
}

.vrt-artifacts-header {
  font-size: 0.8125rem;
  font-weight: 600;
}

.vrt-artifact {
  display: grid;
  gap: 0.25rem;
}

.vrt-artifact-label {
  font-size: 0.6875rem;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--musea-text-muted);
}

.vrt-artifact-path {
  display: block;
  padding: 0.5rem 0.625rem;
  background: var(--musea-bg-primary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  color: var(--musea-text-secondary);
  font-size: 0.75rem;
  font-family: var(--musea-font-mono);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.vrt-stat {
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  padding: 0.5rem 0.75rem;
  text-align: center;
  min-width: 60px;
}

.vrt-stat-value {
  display: block;
  font-size: 1.25rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.vrt-stat-label {
  font-size: 0.625rem;
  color: var(--musea-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.vrt-stat.passed {
  .vrt-stat-value {
    color: var(--musea-success);
  }
}
.vrt-stat.failed {
  .vrt-stat-value {
    color: var(--musea-error);
  }
}
.vrt-stat.new {
  .vrt-stat-value {
    color: var(--musea-info);
  }
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.spin {
  animation: spin 1s linear infinite;
}
</style>
