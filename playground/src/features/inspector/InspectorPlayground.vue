<script setup lang="ts">
import "./InspectorPlayground.css";
import { mdiGithub } from "@mdi/js";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import MonacoEditor from "../../shared/MonacoEditor.vue";
import CodeHighlight from "../../shared/CodeHighlight.vue";
import { codeToThemedTokenLines, type CodeHighlightLanguage } from "../../shared/codeHighlighting";
import { PRESETS } from "../../presets";
import { useClipboard } from "../../utils/useClipboard";
import { useTheme } from "../../utils/useTheme";
import { type loadWasm, getWasm } from "../../wasm/index";
import { compileInspectorReport } from "./compareCompilers";
import {
  buildSplitDiffRows,
  diffLineText,
  renderPlainDiffLines,
  type HighlightedDiffLine,
} from "./diffRows";
import { parseVirLines } from "../croquis/useVirTokenizer";
import { createInspectorUrl, createPullRequestUrl, readInspectorPayloadFromUrl } from "./share";
import type {
  InspectorFile,
  InspectorGraphEdge,
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
const activeOutputTab = ref<
  "compare" | "official" | "vize" | "virtual-ts" | "vir" | "graph" | "payload"
>("compare");
const diffViewMode = ref<"merged" | "split">("merged");
const highlightedDiffLines = ref<HighlightedDiffLine[]>([]);
let latestDiffHighlightId = 0;

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
const inspectorMessages = computed(() => {
  if (!report.value) return [];
  return [
    ...report.value.official.warnings,
    ...report.value.vize.warnings,
    ...report.value.virtualTs.warnings,
    ...report.value.vir.warnings,
    ...(report.value.virtualTs.error ? [`Virtual TS: ${report.value.virtualTs.error}`] : []),
    ...(report.value.vir.error ? [`VIR: ${report.value.vir.error}`] : []),
    ...(report.value.graph.error ? [`Graph: ${report.value.graph.error}`] : []),
  ];
});
const totalInspectTime = computed(() =>
  report.value
    ? report.value.official.timeMs +
      report.value.vize.timeMs +
      report.value.virtualTs.timeMs +
      report.value.vir.timeMs +
      report.value.graph.timeMs
    : 0,
);
const virLines = computed(() => parseVirLines(report.value?.vir.code ?? ""));
const graphDiagnostics = computed(() => report.value?.graph.diagnostics ?? []);
const graphSummary = computed(() => {
  const graph = report.value?.graph;
  return {
    nodes: graph?.nodes.length ?? 0,
    edges: graph?.edges.length ?? 0,
    diagnostics: graph?.diagnostics.length ?? 0,
    cycles: graph?.circularDependencies.length ?? 0,
  };
});
const graphEdgesBySource = computed(() => {
  const grouped: Record<string, InspectorGraphEdge[]> = {};
  for (const edge of report.value?.graph.edges ?? []) {
    if (!grouped[edge.from]) grouped[edge.from] = [];
    grouped[edge.from].push(edge);
  }
  return grouped;
});
const diffLanguage = computed<CodeHighlightLanguage>(() =>
  report.value?.official.parser === "typescript" || report.value?.vize.parser === "typescript"
    ? "typescript"
    : "javascript",
);
const splitDiffRows = computed(() => buildSplitDiffRows(highlightedDiffLines.value));

function graphEdgesFor(path: string): InspectorGraphEdge[] {
  return graphEdgesBySource.value[path] ?? [];
}

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
      files: files.value,
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

async function updateDiffHighlights() {
  const diff = report.value?.diff ?? [];
  const renderId = ++latestDiffHighlightId;
  highlightedDiffLines.value = renderPlainDiffLines(diff);

  if (diff.length === 0) {
    return;
  }

  const highlightedLines = await codeToThemedTokenLines(
    diff.map(diffLineText).join("\n"),
    diffLanguage.value,
  );

  if (renderId !== latestDiffHighlightId) {
    return;
  }

  highlightedDiffLines.value = diff.map((line, index) => ({
    ...line,
    tokens: highlightedLines[index] ?? [
      {
        content: diffLineText(line),
        darkColor: undefined,
        lightColor: undefined,
      },
    ],
  }));
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

watch([() => report.value?.diff, diffLanguage], () => void updateDiffHighlights(), {
  immediate: true,
});

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
        Compiler Inspector
        <span v-if="report" class="compile-time"> {{ totalInspectTime.toFixed(2) }}ms </span>
      </h2>
      <div class="tabs">
        <button
          :class="['tab', { active: activeOutputTab === 'compare' }]"
          @click="activeOutputTab = 'compare'"
        >
          Compare
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
          :class="['tab', { active: activeOutputTab === 'virtual-ts' }]"
          @click="activeOutputTab = 'virtual-ts'"
        >
          Virtual TS
        </button>
        <button
          :class="['tab', { active: activeOutputTab === 'vir' }]"
          @click="activeOutputTab = 'vir'"
        >
          VIR
        </button>
        <button
          :class="['tab', { active: activeOutputTab === 'graph' }]"
          @click="activeOutputTab = 'graph'"
        >
          Graph
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
        <div v-if="inspectorMessages.length > 0" class="inspector-warning-list">
          <pre
            v-for="(warning, index) in inspectorMessages"
            :key="index"
            class="inspector-warning"
            >{{ warning }}</pre
          >
        </div>

        <div v-if="activeOutputTab === 'compare'" class="inspector-tab-panel">
          <div class="inspector-diff-toolbar" aria-label="Diff view">
            <button
              :class="['inspector-diff-mode', { active: diffViewMode === 'merged' }]"
              @click="diffViewMode = 'merged'"
            >
              Merged
            </button>
            <button
              :class="['inspector-diff-mode', { active: diffViewMode === 'split' }]"
              @click="diffViewMode = 'split'"
            >
              Split
            </button>
          </div>
          <div v-if="highlightedDiffLines.length === 0" class="inspector-empty-diff">
            Both compiler outputs are empty.
          </div>
          <div v-else-if="diffViewMode === 'merged'" class="inspector-diff">
            <div
              v-for="(line, index) in highlightedDiffLines"
              :key="index"
              :class="['inspector-diff-line', line.kind]"
            >
              <span class="inspector-diff-num">{{ line.leftLine ?? "" }}</span>
              <span class="inspector-diff-num">{{ line.rightLine ?? "" }}</span>
              <span class="inspector-diff-mark">{{
                line.kind === "add" ? "+" : line.kind === "remove" ? "-" : ""
              }}</span>
              <code class="inspector-diff-code"
                ><span
                  v-for="(token, tokenIndex) in line.tokens"
                  :key="tokenIndex"
                  :style="{ '--d': token.darkColor, '--l': token.lightColor }"
                  >{{ token.content }}</span
                ></code
              >
            </div>
          </div>
          <div v-else class="inspector-diff inspector-diff-split">
            <div
              v-for="(row, index) in splitDiffRows"
              :key="index"
              :class="['inspector-split-line', `left-${row.left.kind}`, `right-${row.right.kind}`]"
            >
              <span class="inspector-diff-num">{{ row.left.line ?? "" }}</span>
              <code :class="['inspector-diff-code', 'inspector-split-code', row.left.kind]"
                ><span
                  v-for="(token, tokenIndex) in row.left.tokens"
                  :key="tokenIndex"
                  :style="{ '--d': token.darkColor, '--l': token.lightColor }"
                  >{{ token.content }}</span
                ></code
              >
              <span class="inspector-diff-num">{{ row.right.line ?? "" }}</span>
              <code :class="['inspector-diff-code', 'inspector-split-code', row.right.kind]"
                ><span
                  v-for="(token, tokenIndex) in row.right.tokens"
                  :key="tokenIndex"
                  :style="{ '--d': token.darkColor, '--l': token.lightColor }"
                  >{{ token.content }}</span
                ></code
              >
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

        <CodeHighlight
          v-else-if="activeOutputTab === 'virtual-ts'"
          :code="
            report.virtualTs.formattedCode ||
            report.virtualTs.error ||
            report.virtualTs.code ||
            'No Virtual TS generated.'
          "
          language="typescript"
          :theme
          show-line-numbers
        />

        <div v-else-if="activeOutputTab === 'vir'" class="inspector-vir-output">
          <div class="inspector-vir-header">
            <span>VIR</span>
            <span>{{ virLines.length }} lines</span>
          </div>
          <div class="inspector-vir-content">
            <div
              v-for="line in virLines"
              :key="line.index"
              :class="['inspector-vir-line', `vir-line-${line.lineType}`]"
            >
              <span class="inspector-vir-ln">{{ line.index + 1 }}</span>
              <span class="inspector-vir-line-text"
                ><template v-if="line.tokens.length > 0"
                  ><span
                    v-for="(token, tokenIndex) in line.tokens"
                    :key="tokenIndex"
                    :class="['vir-token', `vir-${token.type}`]"
                    >{{ token.text }}</span
                  ></template
                ><template v-else><span>&#160;</span></template></span
              >
            </div>
          </div>
          <div v-if="virLines.length === 0" class="inspector-empty-diff">No VIR generated.</div>
        </div>

        <div v-else-if="activeOutputTab === 'graph'" class="inspector-graph-panel">
          <div class="inspector-graph-summary">
            <span>{{ graphSummary.nodes }} files</span>
            <span>{{ graphSummary.edges }} edges</span>
            <span>{{ graphSummary.diagnostics }} diagnostics</span>
            <span>{{ graphSummary.cycles }} cycles</span>
          </div>

          <div class="inspector-graph">
            <div
              v-for="node in report.graph.nodes"
              :key="node.path"
              :class="[
                'inspector-graph-node',
                {
                  active: node.path === selectedFile.path,
                  entry: node.isEntry,
                  issues: node.issueCount > 0,
                },
              ]"
            >
              <div class="inspector-graph-node-main">
                <span class="inspector-graph-file">{{ node.path }}</span>
                <span class="inspector-graph-kind">{{ node.kind }}</span>
                <span v-if="node.issueCount > 0" class="inspector-graph-issues">
                  {{ node.issueCount }}
                </span>
              </div>
              <div class="inspector-graph-node-meta">
                <span>{{ node.sourceLines }} lines</span>
                <span>{{ node.sourceBytes }} bytes</span>
              </div>
              <div v-if="graphEdgesFor(node.path).length > 0" class="inspector-graph-edges">
                <div
                  v-for="edge in graphEdgesFor(node.path)"
                  :key="`${edge.from}-${edge.to}-${edge.kind}-${edge.specifier}`"
                  class="inspector-graph-edge"
                >
                  <span class="inspector-graph-edge-kind">{{ edge.kind }}</span>
                  <span class="inspector-graph-arrow">-&gt;</span>
                  <span class="inspector-graph-target">{{ edge.to }}</span>
                </div>
              </div>
            </div>
          </div>

          <div v-if="graphDiagnostics.length > 0" class="inspector-graph-diagnostics">
            <div
              v-for="(diagnostic, index) in graphDiagnostics"
              :key="`${diagnostic.file}-${diagnostic.code}-${index}`"
              :class="['inspector-graph-diagnostic', diagnostic.severity]"
            >
              <span class="inspector-graph-diagnostic-code">{{ diagnostic.code }}</span>
              <span class="inspector-graph-diagnostic-file">{{ diagnostic.file }}</span>
              <span class="inspector-graph-diagnostic-message">{{ diagnostic.message }}</span>
            </div>
          </div>
        </div>

        <div v-else>
          <CodeHighlight :code="payloadJson" language="json" :theme show-line-numbers />
          <p v-if="permalinkTooLong" class="inspector-url-note">
            Permalink is long; prefer copying the payload for this batch.
          </p>
        </div>
      </template>
    </div>
  </div>
</template>
