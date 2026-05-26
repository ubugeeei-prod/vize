<script setup lang="ts">
import "./InspectorPlayground.css";
import { mdiGithub } from "@mdi/js";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import MonacoEditor from "../../shared/MonacoEditor.vue";
import CodeHighlight from "../../shared/CodeHighlight.vue";
import { PRESETS } from "../../presets";
import { useClipboard } from "../../utils/useClipboard";
import { useTheme } from "../../utils/useTheme";
import { type loadWasm, getWasm } from "../../wasm/index";
import { compileInspectorReport } from "./compareCompilers";
import { createInspectorUrl, createPullRequestUrl, readInspectorPayloadFromUrl } from "./share";
import type {
  InspectorFile,
  InspectorOptions,
  InspectorPayload,
  InspectorReport,
  InspectorTarget,
} from "./types";

const props = defineProps<{
  compiler: Awaited<ReturnType<typeof loadWasm>> | null;
}>();

const { theme } = useTheme();
const { copyToClipboard } = useClipboard();

const files = ref<InspectorFile[]>([
  {
    path: "src/App.vue",
    source: PRESETS.propsDestructure.code,
  },
]);
const selectedFileIndex = ref(0);
const target = ref<InspectorTarget>("dom");
const options = ref<InspectorOptions>({
  customRenderer: false,
  vueParserQuirks: false,
});
const report = ref<InspectorReport | null>(null);
const error = ref<string | null>(null);
const isCompiling = ref(false);
const activeOutputTab = ref<"diff" | "official" | "vize" | "payload">("diff");

const selectedFile = computed(() => files.value[selectedFileIndex.value] ?? files.value[0]!);
const source = computed({
  get: () => selectedFile.value.source,
  set: (value: string) => {
    files.value[selectedFileIndex.value] = {
      ...selectedFile.value,
      source: value,
    };
  },
});

const payload = computed<InspectorPayload>(() => ({
  version: 1,
  target: target.value,
  selectedFile: selectedFile.value.path,
  options: { ...options.value },
  files: files.value.map((file) => ({ ...file })),
}));

const payloadJson = computed(() => JSON.stringify(payload.value, null, 2));
const permalink = computed(() => createInspectorUrl(payload.value));
const pullRequestUrl = computed(() =>
  createPullRequestUrl({
    permalink: permalink.value,
    payload: payload.value,
    stats: report.value?.stats ?? { additions: 0, removals: 0, unchanged: 0 },
  }),
);
const permalinkTooLong = computed(() => permalink.value.length > 7000);
const hasChanges = computed(
  () => (report.value?.stats.additions ?? 0) > 0 || (report.value?.stats.removals ?? 0) > 0,
);

function applyPayload(nextPayload: InspectorPayload) {
  files.value = nextPayload.files.map((file, index) => ({
    path: file.path || `repro-${index + 1}.vue`,
    source: file.source,
  }));
  target.value = nextPayload.target === "ssr" ? "ssr" : "dom";
  options.value = {
    customRenderer: nextPayload.options?.customRenderer ?? false,
    vueParserQuirks: nextPayload.options?.vueParserQuirks ?? false,
  };
  const selected = nextPayload.selectedFile
    ? files.value.findIndex((file) => file.path === nextPayload.selectedFile)
    : 0;
  selectedFileIndex.value = selected >= 0 ? selected : 0;
}

async function compile() {
  const compiler = props.compiler ?? getWasm();
  if (!compiler) return;

  isCompiling.value = true;
  error.value = null;

  try {
    report.value = await compileInspectorReport({
      compiler,
      file: selectedFile.value,
      target: target.value,
      options: options.value,
    });
  } catch (compileError) {
    error.value = compileError instanceof Error ? compileError.message : String(compileError);
  } finally {
    isCompiling.value = false;
  }
}

let compileTimer: ReturnType<typeof setTimeout> | null = null;
function scheduleCompile() {
  if (compileTimer) clearTimeout(compileTimer);
  compileTimer = setTimeout(() => {
    void compile();
  }, 250);
}

function openPullRequest() {
  window.open(pullRequestUrl.value, "_blank", "noopener,noreferrer");
}

watch(
  [source, target, options, selectedFileIndex],
  () => {
    if (props.compiler ?? getWasm()) scheduleCompile();
  },
  { deep: true },
);

watch(
  () => props.compiler,
  () => {
    if (props.compiler) void compile();
  },
  { immediate: true },
);

let hasCompilerInitialized = false;
let pollInterval: ReturnType<typeof setInterval> | null = null;

function tryInitialize() {
  const compiler = getWasm();
  if (compiler && !hasCompilerInitialized) {
    hasCompilerInitialized = true;
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
    void compile();
  }
}

onMounted(() => {
  const urlPayload = readInspectorPayloadFromUrl();
  if (urlPayload) {
    applyPayload(urlPayload);
  }

  tryInitialize();
  if (!hasCompilerInitialized) {
    pollInterval = setInterval(tryInitialize, 100);
    setTimeout(() => {
      if (pollInterval) {
        clearInterval(pollInterval);
        pollInterval = null;
      }
    }, 10000);
  }
});

onUnmounted(() => {
  if (compileTimer) clearTimeout(compileTimer);
  if (pollInterval) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
});
</script>

<template>
  <div class="panel input-panel">
    <div class="panel-header">
      <h2>Inspector Source</h2>
      <div class="panel-actions">
        <button class="btn-ghost" @click="copyToClipboard(source)">Copy</button>
        <button class="btn-ghost" @click="copyToClipboard(permalink)">Permalink</button>
        <button class="btn-ghost" @click="copyToClipboard(payloadJson)">Payload</button>
      </div>
    </div>

    <div class="inspector-file-list">
      <button
        v-for="(file, index) in files"
        :key="`${file.path}-${index}`"
        :class="['inspector-file-tab', { active: selectedFileIndex === index }]"
        :title="file.path"
        @click="selectedFileIndex = index"
      >
        {{ file.path }}
      </button>
    </div>

    <div class="panel-header">
      <div class="inspector-controls">
        <div class="inspector-targets" aria-label="Compiler target">
          <button
            :class="['inspector-target', { active: target === 'dom' }]"
            @click="target = 'dom'"
          >
            DOM
          </button>
          <button
            :class="['inspector-target', { active: target === 'ssr' }]"
            @click="target = 'ssr'"
          >
            SSR
          </button>
        </div>
        <label class="inspector-option">
          <input v-model="options.customRenderer" type="checkbox" />
          <span>custom renderer</span>
        </label>
        <label class="inspector-option">
          <input v-model="options.vueParserQuirks" type="checkbox" />
          <span>Vue parser quirks</span>
        </label>
      </div>
    </div>

    <div class="editor-container">
      <MonacoEditor v-model="source" language="vue" :theme />
    </div>
  </div>

  <div class="panel output-panel">
    <div class="panel-header">
      <h2>
        Compiler Diff
        <span v-if="report" class="compile-time">
          {{ (report.official.timeMs + report.vize.timeMs).toFixed(2) }}ms
        </span>
      </h2>
      <div class="tabs">
        <button
          :class="['tab', { active: activeOutputTab === 'diff' }]"
          @click="activeOutputTab = 'diff'"
        >
          Diff
        </button>
        <button
          :class="['tab', { active: activeOutputTab === 'official' }]"
          @click="activeOutputTab = 'official'"
        >
          Vue
        </button>
        <button
          :class="['tab', { active: activeOutputTab === 'vize' }]"
          @click="activeOutputTab = 'vize'"
        >
          Vize
        </button>
        <button
          :class="['tab', { active: activeOutputTab === 'payload' }]"
          @click="activeOutputTab = 'payload'"
        >
          Payload
        </button>
        <a
          class="inspector-pr-link tab-copy-btn"
          href="https://github.com/ubugeeei/vize/compare/main...compiler-inspector-repro"
          target="_blank"
          rel="noreferrer"
          @click.prevent="openPullRequest"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path :d="mdiGithub" />
          </svg>
          <span>Create PR</span>
        </a>
      </div>
    </div>

    <div class="output-content">
      <div v-if="isCompiling" class="compiling">
        <div class="spinner" />
        <span>Compiling...</span>
      </div>

      <div v-else-if="error" class="wasm-error">
        <h3>Inspector Error</h3>
        <pre>{{ error }}</pre>
      </div>

      <template v-else-if="report">
        <div class="inspector-summary">
          <div class="inspector-stat">
            <span class="inspector-stat-value">{{ report.target.toUpperCase() }}</span>
            <span class="inspector-stat-label">target</span>
          </div>
          <div class="inspector-stat">
            <span class="inspector-stat-value">{{ hasChanges ? "diff" : "same" }}</span>
            <span class="inspector-stat-label">status</span>
          </div>
          <div class="inspector-stat">
            <span class="inspector-stat-value">+{{ report.stats.additions }}</span>
            <span class="inspector-stat-label">Vize-only lines</span>
          </div>
          <div class="inspector-stat">
            <span class="inspector-stat-value">-{{ report.stats.removals }}</span>
            <span class="inspector-stat-label">Vue-only lines</span>
          </div>
        </div>

        <div
          v-if="report.official.warnings.length > 0 || report.vize.warnings.length > 0"
          class="inspector-warning-list"
        >
          <pre
            v-for="(warning, index) in [...report.official.warnings, ...report.vize.warnings]"
            :key="index"
            class="inspector-warning"
            >{{ warning }}</pre
          >
        </div>

        <div v-if="activeOutputTab === 'diff'" class="inspector-tab-panel">
          <div v-if="report.diff.length === 0" class="inspector-empty-diff">
            Both compilers produced empty output.
          </div>
          <div v-else class="inspector-diff">
            <div
              v-for="(line, index) in report.diff"
              :key="index"
              :class="['inspector-diff-line', line.kind]"
            >
              <span class="inspector-diff-num">{{ line.leftLine ?? "" }}</span>
              <span class="inspector-diff-num">{{ line.rightLine ?? "" }}</span>
              <span class="inspector-diff-mark">{{
                line.kind === "add" ? "+" : line.kind === "remove" ? "-" : ""
              }}</span>
              <code class="inspector-diff-code">{{ line.text || " " }}</code>
            </div>
          </div>
        </div>

        <CodeHighlight
          v-else-if="activeOutputTab === 'official'"
          :code="report.official.formattedCode || report.official.error || report.official.code"
          :language="report.official.parser === 'typescript' ? 'typescript' : 'javascript'"
          :theme
          show-line-numbers
        />

        <CodeHighlight
          v-else-if="activeOutputTab === 'vize'"
          :code="report.vize.formattedCode || report.vize.error || report.vize.code"
          :language="report.vize.parser === 'typescript' ? 'typescript' : 'javascript'"
          :theme
          show-line-numbers
        />

        <div v-else>
          <pre class="inspector-payload">{{ payloadJson }}</pre>
          <p v-if="permalinkTooLong" class="inspector-url-note">
            Permalink is long; prefer copying the payload for this batch.
          </p>
        </div>
      </template>
    </div>
  </div>
</template>
