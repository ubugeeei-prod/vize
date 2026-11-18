<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, useTemplateRef, watch } from "vue";
import { mdiCheckCircle, mdiCloseCircle, mdiLoading, mdiPlay } from "@mdi/js";
import type { ArtVariant } from "../../src/types/index.js";
import { getPreviewUrl, vMuseaGlobals } from "../api";
import { sendMessage } from "../composables/usePostMessage";
import { safeUrl } from "../utils/safeUrl";
import MdiIcon from "./MdiIcon.vue";

const props = defineProps<{ artPath: string; variant: ArtVariant; variants: ArtVariant[] }>();
const emit = defineEmits<{ (event: "selectVariant", name: string): void }>();
const iframeRef = useTemplateRef<HTMLIFrameElement>("iframeRef");
const ready = ref(false);
const status = ref<"idle" | "running" | "passed" | "failed" | "skipped">("idle");
const duration = ref(0);
const error = ref("");
type TraceEntry = {
  kind: "step" | "action" | "assertion";
  name: string;
  status: "running" | "passed" | "failed";
  duration: number;
  target?: string;
  error?: string;
};
const trace = ref<TraceEntry[]>([]);
let requestId = "";
let timeout: ReturnType<typeof setTimeout> | undefined;
const runNonce = ref(0);

const previewUrl = computed(() => getPreviewUrl(props.artPath, props.variant.name));
const frameUrl = computed(() => {
  if (runNonce.value === 0) return previewUrl.value;
  const url = new URL(previewUrl.value, window.location.origin);
  url.searchParams.set("museaTestRun", String(runNonce.value));
  return url.href;
});
const hasPlay = computed(() => Boolean(props.variant.play));
const playLabel = computed(() =>
  props.variant.play && /^[A-Za-z_$][\w$]*$/.test(props.variant.play)
    ? props.variant.play
    : "Inline play",
);

function clearPending() {
  if (timeout) clearTimeout(timeout);
  timeout = undefined;
  requestId = "";
}

function runPlay() {
  const iframe = iframeRef.value;
  if (!iframe || !ready.value || !hasPlay.value || status.value === "running") return;
  const firstRun = status.value === "idle";
  clearPending();
  status.value = "running";
  duration.value = 0;
  error.value = "";
  trace.value = [];
  requestId = globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;
  if (firstRun) {
    sendMessage(iframe, "musea:run-play", { requestId });
  } else {
    ready.value = false;
    runNonce.value++;
  }
  timeout = setTimeout(() => {
    status.value = "failed";
    error.value = "Play function timed out after 10 seconds.";
    clearPending();
  }, 10_000);
}

function onMessage(event: MessageEvent) {
  if (event.origin !== window.location.origin || event.source !== iframeRef.value?.contentWindow)
    return;
  if (event.data?.type === "musea:ready") {
    ready.value = true;
    if (requestId && status.value === "running") {
      sendMessage(iframeRef.value!, "musea:run-play", { requestId });
    }
    return;
  }
  if (event.data?.type !== "musea:play-result" || event.data.payload?.requestId !== requestId)
    return;
  const result = event.data.payload as {
    status: "passed" | "failed" | "skipped";
    duration?: number;
    error?: string;
    report?: { trace?: TraceEntry[] };
  };
  status.value = result.status;
  duration.value = result.duration ?? 0;
  error.value = result.error ?? "";
  trace.value = result.report?.trace ?? [];
  clearPending();
}

watch(previewUrl, () => {
  ready.value = false;
  status.value = "idle";
  duration.value = 0;
  error.value = "";
  trace.value = [];
  runNonce.value = 0;
  clearPending();
});

onMounted(() => window.addEventListener("message", onMessage));
onUnmounted(() => {
  window.removeEventListener("message", onMessage);
  clearPending();
});
</script>

<template>
  <div class="test-panel">
    <div class="test-header">
      <div>
        <p class="test-eyebrow">Interaction test</p>
        <div class="test-title-row">
          <h2>{{ variant.name }}</h2>
          <label class="variant-picker">
            <span>Variant</span>
            <select
              :value="variant.name"
              @change="emit('selectVariant', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="item in variants" :key="item.name" :value="item.name">
                {{ item.name }}{{ item.isDefault ? " · Default" : "" }}
              </option>
            </select>
          </label>
        </div>
        <p v-if="variant.play" class="test-binding">
          <code>{{ playLabel }}</code>
        </p>
      </div>
      <button
        type="button"
        class="run-button"
        :disabled="!ready || !hasPlay || status === 'running'"
        @click="runPlay"
      >
        <MdiIcon
          :class="{ spin: status === 'running' }"
          :path="status === 'running' ? mdiLoading : mdiPlay"
          :size="16"
        />
        {{ status === "running" ? "Running…" : status === "idle" ? "Run play" : "Run again" }}
      </button>
    </div>

    <div v-if="!hasPlay" class="empty-state">
      Add <code>:play="async ({ canvas }) =&gt; { … }"</code> to this variant. Use
      <code>definePlay()</code> only when the interaction is shared.
    </div>
    <div v-else-if="status !== 'idle'" class="test-result" :class="status" role="status">
      <MdiIcon v-if="status === 'passed'" :path="mdiCheckCircle" :size="18" />
      <MdiIcon v-else-if="status === 'failed'" :path="mdiCloseCircle" :size="18" />
      <MdiIcon v-else :path="mdiLoading" :size="18" :class="{ spin: status === 'running' }" />
      <div>
        <strong>{{
          status === "passed" ? "Passed" : status === "failed" ? "Failed" : "Running"
        }}</strong>
        <span v-if="duration"> · {{ duration }} ms</span>
        <pre v-if="error">{{ error }}</pre>
      </div>
    </div>

    <ol v-if="trace.length" class="test-trace" aria-label="Play trace">
      <li v-for="(entry, index) in trace" :key="`${index}-${entry.name}`" :class="entry.status">
        <MdiIcon :path="entry.status === 'passed' ? mdiCheckCircle : mdiCloseCircle" :size="15" />
        <div>
          <strong>{{ entry.name }}</strong>
          <span>
            {{ entry.kind }}<template v-if="entry.target"> · {{ entry.target }}</template>
            <template v-if="entry.duration"> · {{ entry.duration }} ms</template>
          </span>
          <small v-if="entry.error">{{ entry.error }}</small>
        </div>
      </li>
    </ol>

    <div class="test-preview">
      <iframe
        ref="iframeRef"
        v-musea-globals
        :src="safeUrl(frameUrl)"
        :title="`${variant.name} interaction test`"
        sandbox="allow-scripts allow-same-origin allow-forms"
      />
    </div>
  </div>
</template>

<style scoped>
.test-panel {
  display: grid;
  gap: 1rem;
}
.test-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}
.test-header h2 {
  margin: 0.125rem 0;
  font-size: 1rem;
}
.test-title-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.variant-picker {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  color: var(--musea-text-muted);
  font-size: 0.6875rem;
  font-weight: 600;
}
.variant-picker select {
  max-width: 12rem;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  padding: 0.3125rem 1.5rem 0.3125rem 0.5rem;
  background: var(--musea-bg-primary);
  color: var(--musea-text);
  font: inherit;
}
.test-eyebrow {
  color: var(--musea-text-muted);
  font-size: 0.6875rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}
.test-binding {
  color: var(--musea-text-muted);
  font-size: 0.75rem;
}
.run-button {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  border: 0;
  border-radius: var(--musea-radius-md);
  padding: 0.625rem 0.875rem;
  background: var(--musea-accent);
  color: white;
  cursor: pointer;
  font-weight: 600;
}
.run-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}
.empty-state,
.test-result {
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  padding: 0.875rem 1rem;
  background: var(--musea-bg-secondary);
  color: var(--musea-text-muted);
}
.test-result {
  display: flex;
  align-items: flex-start;
  gap: 0.625rem;
  color: var(--musea-text);
}
.test-result.passed {
  border-color: var(--musea-a11y-passed);
  color: var(--musea-a11y-passed);
}
.test-result.failed {
  border-color: var(--musea-a11y-critical);
  color: var(--musea-a11y-critical);
}
.test-result pre {
  margin-top: 0.5rem;
  white-space: pre-wrap;
  font:
    0.75rem/1.5 ui-monospace,
    monospace;
  color: var(--musea-text);
}
.test-trace {
  display: grid;
  gap: 0.375rem;
  margin: 0;
  padding: 0;
  list-style: none;
}
.test-trace li {
  display: grid;
  grid-template-columns: 1rem 1fr;
  gap: 0.5rem;
  align-items: start;
  border-left: 2px solid var(--musea-border);
  padding: 0.5rem 0.625rem;
  color: var(--musea-text);
}
.test-trace li.passed {
  border-left-color: var(--musea-a11y-passed);
}
.test-trace li.failed {
  border-left-color: var(--musea-a11y-critical);
}
.test-trace strong,
.test-trace span,
.test-trace small {
  display: block;
}
.test-trace strong {
  font-size: 0.75rem;
}
.test-trace span,
.test-trace small {
  margin-top: 0.125rem;
  color: var(--musea-text-muted);
  font-size: 0.6875rem;
}
.test-preview {
  min-height: 420px;
  overflow: hidden;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-lg);
  background: white;
}
.test-preview iframe {
  width: 100%;
  height: 420px;
  border: 0;
}
.spin {
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
