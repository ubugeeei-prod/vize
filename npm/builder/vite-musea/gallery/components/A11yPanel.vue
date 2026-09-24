<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import {
  mdiPlay,
  mdiLoading,
  mdiCheckCircle,
  mdiOpenInNew,
  mdiChevronDown,
  mdiChevronUp,
} from "@mdi/js";
import { useA11y, type A11yResult } from "../composables/useA11y";
import { getPreviewUrl } from "../api";
import MdiIcon from "./MdiIcon.vue";

const props = defineProps<{
  artPath: string;
  defaultVariantName?: string;
}>();

const { isKeyRunning, init, runA11y, getResult } = useA11y();

const iframeRef = ref<HTMLIFrameElement | null>(null);
const iframeReady = ref(false);
const hasRun = ref(false);
const expandedViolation = ref<string | null>(null);

const key = computed(() => `${props.artPath}:${props.defaultVariantName || "default"}`);
const result = computed<A11yResult | undefined>(() => getResult(key.value));
const isRunning = computed(() => isKeyRunning(key.value));

const previewUrl = computed(() => {
  if (!props.defaultVariantName) return "";
  return getPreviewUrl(props.artPath, props.defaultVariantName);
});

onMounted(() => {
  init();
});

function onIframeLoad() {
  iframeReady.value = true;
}

function runTest() {
  if (!iframeRef.value || !iframeReady.value) return;
  hasRun.value = true;
  runA11y(iframeRef.value, key.value);
}

function toggleViolation(id: string) {
  expandedViolation.value = expandedViolation.value === id ? null : id;
}

function safeUrl(rawUrl: string, destination: "preview" | "help"): string | undefined {
  if (!rawUrl) return undefined;
  try {
    const url = new URL(rawUrl, window.location.href);
    if (destination === "preview" && url.origin === window.location.origin) {
      return url.href;
    }
    if (destination === "help" && url.protocol === "https:") {
      return url.href;
    }
  } catch {
    // Ignore malformed URLs from a static gallery or an audit result.
  }
  return undefined;
}

function getImpactColor(impact: string): string {
  switch (impact) {
    case "critical":
      return "var(--musea-a11y-critical)";
    case "serious":
      return "var(--musea-a11y-serious)";
    case "moderate":
      return "var(--musea-a11y-moderate)";
    case "minor":
      return "var(--musea-a11y-minor)";
    default:
      return "var(--musea-a11y-unknown)";
  }
}

const summary = computed(() => {
  if (!result.value) return null;
  const violations = result.value.violations;
  return {
    total: violations.length,
    critical: violations.filter((v) => v.impact === "critical").length,
    serious: violations.filter((v) => v.impact === "serious").length,
    moderate: violations.filter((v) => v.impact === "moderate").length,
    minor: violations.filter((v) => v.impact === "minor").length,
    passes: result.value.passes,
  };
});

watch(
  key,
  (currentKey) => {
    hasRun.value = Boolean(getResult(currentKey));
    expandedViolation.value = null;
  },
  { immediate: true },
);

watch(result, (nextResult) => {
  if (nextResult) {
    hasRun.value = true;
  }
});
</script>

<template>
  <div class="a11y-panel">
    <!-- Hidden iframe for testing -->
    <iframe
      v-if="safeUrl(previewUrl, 'preview')"
      ref="iframeRef"
      :src="safeUrl(previewUrl, 'preview')"
      class="a11y-iframe"
      title="Accessibility test preview"
      sandbox="allow-scripts allow-same-origin"
      @load="onIframeLoad"
    />

    <div class="a11y-header">
      <h3 class="a11y-title">Accessibility Test</h3>
      <button
        type="button"
        class="a11y-run-btn"
        :disabled="isRunning || !iframeReady"
        @click="runTest"
      >
        <MdiIcon v-if="isRunning" class="spin" :path="mdiLoading" :size="14" />
        <MdiIcon v-else :path="mdiPlay" :size="14" />
        {{ isRunning ? "Running..." : hasRun ? "Run Again" : "Run Test" }}
      </button>
    </div>

    <div v-if="!hasRun" class="a11y-empty">
      <p>Click "Run Test" to check accessibility with axe-core.</p>
      <p class="a11y-hint">Tests WCAG 2.0/2.1 AA criteria and best practices.</p>
    </div>

    <div v-else-if="isRunning && !result" class="a11y-empty">
      <MdiIcon class="spin a11y-running-icon" :path="mdiLoading" :size="20" />
      <p>Running accessibility audit...</p>
      <p class="a11y-hint">Results will appear here as soon as the iframe responds.</p>
    </div>

    <template v-else-if="result">
      <div v-if="result.error" class="a11y-error">
        {{ result.error }}
      </div>

      <template v-else>
        <div v-if="summary" class="a11y-summary">
          <div class="a11y-stat" :class="{ 'has-issues': summary.total > 0 }">
            <span class="a11y-stat-value">{{ summary.total }}</span>
            <span class="a11y-stat-label">Violations</span>
          </div>
          <div v-if="summary.critical > 0" class="a11y-stat critical">
            <span class="a11y-stat-value">{{ summary.critical }}</span>
            <span class="a11y-stat-label">Critical</span>
          </div>
          <div v-if="summary.serious > 0" class="a11y-stat serious">
            <span class="a11y-stat-value">{{ summary.serious }}</span>
            <span class="a11y-stat-label">Serious</span>
          </div>
          <div v-if="summary.moderate > 0" class="a11y-stat moderate">
            <span class="a11y-stat-value">{{ summary.moderate }}</span>
            <span class="a11y-stat-label">Moderate</span>
          </div>
          <div v-if="summary.minor > 0" class="a11y-stat minor">
            <span class="a11y-stat-value">{{ summary.minor }}</span>
            <span class="a11y-stat-label">Minor</span>
          </div>
          <div class="a11y-stat passes">
            <span class="a11y-stat-value">{{ summary.passes }}</span>
            <span class="a11y-stat-label">Passes</span>
          </div>
        </div>

        <div v-if="result.violations.length === 0" class="a11y-success">
          <MdiIcon :path="mdiCheckCircle" :size="24" />
          <span>No accessibility violations found</span>
        </div>

        <div v-else class="a11y-violations">
          <div
            v-for="violation in result.violations"
            :key="violation.id"
            class="a11y-violation"
            :class="{ expanded: expandedViolation === violation.id }"
          >
            <button
              type="button"
              class="a11y-violation-header"
              :aria-expanded="expandedViolation === violation.id"
              @click="() => toggleViolation(violation.id)"
            >
              <span class="a11y-impact" :style="{ color: getImpactColor(violation.impact) }">
                {{ violation.impact }}
              </span>
              <span class="a11y-rule-id">{{ violation.id }}</span>
              <span class="a11y-node-count">{{ violation.nodes.length }} element(s)</span>
              <MdiIcon
                class="a11y-expand-icon"
                :path="expandedViolation === violation.id ? mdiChevronUp : mdiChevronDown"
                :size="14"
              />
            </button>
            <div v-if="expandedViolation === violation.id" class="a11y-violation-detail">
              <p class="a11y-description">{{ violation.description }}</p>
              <a
                v-if="safeUrl(violation.helpUrl, 'help')"
                :href="safeUrl(violation.helpUrl, 'help')"
                target="_blank"
                rel="noopener noreferrer"
                class="a11y-help-link"
              >
                Learn more
                <MdiIcon :path="mdiOpenInNew" :size="12" />
              </a>
              <div class="a11y-nodes">
                <div v-for="node in violation.nodes" :key="node.target.join(' ')" class="a11y-node">
                  <pre class="a11y-node-html">{{ node.html }}</pre>
                  <p v-if="node.failureSummary" class="a11y-node-summary">
                    {{ node.failureSummary }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </template>
    </template>
  </div>
</template>

<style scoped>
.a11y-panel {
  padding: 0.5rem;
}

.a11y-iframe {
  position: absolute;
  width: 1px;
  height: 1px;
  opacity: 0;
  pointer-events: none;
}

.a11y-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
}

.a11y-title {
  font-size: 0.875rem;
  font-weight: 600;
}

.a11y-run-btn {
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

.a11y-run-btn:hover:not(:disabled) {
  background: var(--musea-accent-hover);
}

.a11y-run-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.a11y-empty {
  padding: 2rem;
  text-align: center;
  color: var(--musea-text-muted);
  font-size: 0.875rem;
}

.a11y-hint {
  font-size: 0.75rem;
  margin-top: 0.5rem;
  opacity: 0.7;
}

.a11y-running-icon {
  margin-bottom: 0.75rem;
}

.a11y-error {
  padding: 1rem;
  background: color-mix(in srgb, var(--musea-a11y-critical) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--musea-a11y-critical) 20%, transparent);
  border-radius: var(--musea-radius-sm);
  color: var(--musea-a11y-critical);
  font-size: 0.8125rem;
}

.a11y-summary {
  display: flex;
  gap: 0.75rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}

.a11y-stat {
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  padding: 0.5rem 0.75rem;
  text-align: center;
  min-width: 60px;
}

.a11y-stat-value {
  display: block;
  font-size: 1.25rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.a11y-stat-label {
  font-size: 0.625rem;
  color: var(--musea-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.a11y-stat {
  &.has-issues .a11y-stat-value,
  &.critical .a11y-stat-value {
    color: var(--musea-a11y-critical);
  }

  &.serious .a11y-stat-value {
    color: var(--musea-a11y-serious);
  }

  &.moderate .a11y-stat-value {
    color: var(--musea-a11y-moderate);
  }

  &.minor .a11y-stat-value {
    color: var(--musea-a11y-minor);
  }

  &.passes .a11y-stat-value {
    color: var(--musea-a11y-passed);
  }
}

.a11y-success {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem;
  background: color-mix(in srgb, var(--musea-a11y-passed) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--musea-a11y-passed) 20%, transparent);
  border-radius: var(--musea-radius-sm);
  color: var(--musea-a11y-passed);
  font-weight: 500;
}

.a11y-violations {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.a11y-violation {
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  overflow: hidden;
}

.a11y-violation-header {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border: 0;
  background: transparent;
  color: inherit;
  cursor: pointer;
  font-family: inherit;
  font-size: 0.75rem;
  text-align: start;
}

.a11y-violation-header:hover {
  background: var(--musea-bg-tertiary);
}

.a11y-impact {
  font-weight: 700;
  text-transform: uppercase;
  font-size: 0.625rem;
}

.a11y-rule-id {
  font-family: var(--musea-font-mono);
  color: var(--musea-text-secondary);
}

.a11y-node-count {
  color: var(--musea-text-muted);
  margin-inline-start: auto;
}

.a11y-expand-icon {
  color: var(--musea-text-muted);
}

.a11y-violation-detail {
  padding: 0.75rem;
  border-top: 1px solid var(--musea-border);
  background: var(--musea-bg-tertiary);
}

.a11y-description {
  font-size: 0.8125rem;
  color: var(--musea-text-secondary);
  margin-bottom: 0.5rem;
}

.a11y-help-link {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 0.75rem;
  color: var(--musea-accent);
  text-decoration: none;
}

.a11y-help-link:hover {
  text-decoration: underline;
}

.a11y-nodes {
  margin-top: 0.75rem;
}

.a11y-node {
  background: var(--musea-bg-primary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  padding: 0.5rem;
  margin-top: 0.5rem;
}

.a11y-node-html {
  font-family: var(--musea-font-mono);
  font-size: 0.6875rem;
  color: var(--musea-text-secondary);
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}

.a11y-node-summary {
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
  margin-top: 0.375rem;
  white-space: pre-wrap;
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
