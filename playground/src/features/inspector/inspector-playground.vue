<script setup lang="ts">
import { mdiGithub } from "@mdi/js";
import { computed, onMounted, onUnmounted, ref, useId, watch } from "vue";
import MonacoEditor from "../../shared/monaco-editor.vue";
import CodeHighlight from "../../shared/code-highlight.vue";
import type { CodeHighlightLanguage } from "../../shared/codeHighlighting";
import { createCodeHighlighter } from "../../shared/codeHighlightingClient";
import { PRESETS } from "../../presets";
import { useClipboard } from "../../utils/useClipboard";
import { useTheme } from "../../utils/useTheme";
import { withTokenOffsets } from "../../utils/withTokenOffsets";
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

const { compiler: compilerProp } = defineProps<{
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
  templateSyntax: "standard",
});
const templateSyntaxId = useId();
const report = ref<InspectorReport | null>(null);
const error = ref<string | null>(null);
const isCompiling = ref(false);
const activeOutputTab = ref<
  "compare" | "official" | "vize" | "virtual-ts" | "vir" | "graph" | "payload"
>("compare");
const diffViewMode = ref<"merged" | "split">("merged");
const highlightedDiffLines = ref<HighlightedDiffLine[]>([]);
const diffHighlighter = createCodeHighlighter();
const inspectorTargets = ["dom", "ssr", "vapor"] as const satisfies readonly InspectorTarget[];

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

function normalizeInspectorTarget(value: unknown): InspectorTarget {
  return inspectorTargets.includes(value as InspectorTarget) ? (value as InspectorTarget) : "dom";
}

function applyPayload(nextPayload: InspectorPayload) {
  files.value = nextPayload.files.map((file, index) => ({
    path: file.path || `repro-${index + 1}.vue`,
    source: file.source,
  }));
  target.value = normalizeInspectorTarget(nextPayload.target);
  options.value = {
    customRenderer: nextPayload.options?.customRenderer ?? false,
    templateSyntax: nextPayload.options?.templateSyntax ?? "standard",
  };
  const selected = nextPayload.selectedFile
    ? files.value.findIndex((file) => file.path === nextPayload.selectedFile)
    : 0;
  selectedFileIndex.value = selected >= 0 ? selected : 0;
}

async function compile() {
  const compiler = compilerProp ?? getWasm();
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

function updateDiffHighlights() {
  const diff = report.value?.diff ?? [];
  highlightedDiffLines.value = renderPlainDiffLines(diff);

  if (diff.length === 0) {
    diffHighlighter.cancel();
    return;
  }

  diffHighlighter.highlight(
    diff.map(diffLineText).join("\n"),
    diffLanguage.value,
    (highlightedLines) => {
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
    },
  );
}

watch(
  [source, target, options, selectedFileIndex],
  () => {
    if (compilerProp ?? getWasm()) scheduleCompile();
  },
  { deep: true },
);

watch(
  () => compilerProp,
  () => {
    if (compilerProp) void compile();
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
  diffHighlighter.cancel();
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
        <button type="button" class="btn-ghost" @click="() => copyToClipboard(source)">Copy</button>
        <button type="button" class="btn-ghost" @click="() => copyToClipboard(permalink)">
          Permalink
        </button>
        <button type="button" class="btn-ghost" @click="() => copyToClipboard(payloadJson)">
          Payload
        </button>
      </div>
    </div>

    <div class="inspector-file-list">
      <button
        v-for="(file, index) in files"
        :key="`${file.path}-${index}`"
        type="button"
        :class="['inspector-file-tab', { active: selectedFileIndex === index }]"
        :title="file.path"
        @click="() => (selectedFileIndex = index)"
      >
        {{ file.path }}
      </button>
    </div>

    <div class="panel-header">
      <div class="inspector-controls">
        <div class="inspector-targets" aria-label="Compiler target">
          <button
            type="button"
            :class="['inspector-target', { active: target === 'dom' }]"
            @click="() => (target = 'dom')"
          >
            DOM
          </button>
          <button
            type="button"
            :class="['inspector-target', { active: target === 'ssr' }]"
            @click="() => (target = 'ssr')"
          >
            SSR
          </button>
          <button
            type="button"
            :class="['inspector-target', { active: target === 'vapor' }]"
            @click="() => (target = 'vapor')"
          >
            Vapor
          </button>
        </div>
        <label class="inspector-option">
          <input v-model="options.customRenderer" type="checkbox" />
          <span>custom renderer</span>
        </label>
        <div class="inspector-option">
          <label :for="templateSyntaxId">template syntax</label>
          <select :id="templateSyntaxId" v-model="options.templateSyntax">
            <option value="standard">standard</option>
            <option value="strict">strict</option>
            <option value="quirks">quirks</option>
          </select>
        </div>
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
          type="button"
          :class="['tab', { active: activeOutputTab === 'compare' }]"
          @click="() => (activeOutputTab = 'compare')"
        >
          Compare
        </button>
        <button
          type="button"
          :class="['tab', { active: activeOutputTab === 'official' }]"
          @click="() => (activeOutputTab = 'official')"
        >
          Vue
        </button>
        <button
          type="button"
          :class="['tab', { active: activeOutputTab === 'vize' }]"
          @click="() => (activeOutputTab = 'vize')"
        >
          Vize
        </button>
        <button
          type="button"
          :class="['tab', { active: activeOutputTab === 'virtual-ts' }]"
          @click="() => (activeOutputTab = 'virtual-ts')"
        >
          Virtual TS
        </button>
        <button
          type="button"
          :class="['tab', { active: activeOutputTab === 'vir' }]"
          @click="() => (activeOutputTab = 'vir')"
        >
          VIR
        </button>
        <button
          type="button"
          :class="['tab', { active: activeOutputTab === 'graph' }]"
          @click="() => (activeOutputTab = 'graph')"
        >
          Graph
        </button>
        <button
          type="button"
          :class="['tab', { active: activeOutputTab === 'payload' }]"
          @click="() => (activeOutputTab = 'payload')"
        >
          Payload
        </button>
        <a
          class="inspector-pr-link tab-copy-btn"
          href="https://github.com/ubugeeei-prod/vize/compare/main...compiler-inspector-repro"
          target="_blank"
          rel="noopener noreferrer"
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
        <div class="spinner"></div>
        <span>Compiling...</span>
      </div>

      <div v-else-if="error" class="wasm-error">
        <h3>Inspector Error</h3>
        <pre>{{ error }}</pre>
      </div>

      <template v-else-if="report">
        <div v-if="inspectorMessages.length > 0" class="inspector-warning-list">
          <pre v-for="warning in inspectorMessages" :key="warning" class="inspector-warning">{{
            warning
          }}</pre>
        </div>

        <div v-if="activeOutputTab === 'compare'" class="inspector-tab-panel">
          <div class="inspector-diff-toolbar" aria-label="Diff view">
            <button
              type="button"
              :class="['inspector-diff-mode', { active: diffViewMode === 'merged' }]"
              @click="() => (diffViewMode = 'merged')"
            >
              Merged
            </button>
            <button
              type="button"
              :class="['inspector-diff-mode', { active: diffViewMode === 'split' }]"
              @click="() => (diffViewMode = 'split')"
            >
              Split
            </button>
          </div>
          <div v-if="highlightedDiffLines.length === 0" class="inspector-empty-diff">
            Both compiler outputs are empty.
          </div>
          <div v-else-if="diffViewMode === 'merged'" class="inspector-diff">
            <div
              v-for="line in highlightedDiffLines"
              :key="`${line.kind}:${line.leftLine}:${line.rightLine}`"
              :class="['inspector-diff-line', line.kind]"
            >
              <span class="inspector-diff-num">{{ line.leftLine ?? "" }}</span>
              <span class="inspector-diff-num">{{ line.rightLine ?? "" }}</span>
              <span class="inspector-diff-mark">{{
                line.kind === "add" ? "+" : line.kind === "remove" ? "-" : ""
              }}</span>
              <code class="inspector-diff-code"
                ><span
                  v-for="token in withTokenOffsets(line.tokens)"
                  :key="token.offset"
                  :style="{ '--d': token.darkColor, '--l': token.lightColor }"
                  >{{ token.content }}</span
                ></code
              >
            </div>
          </div>
          <div v-else class="inspector-diff inspector-diff-split">
            <div
              v-for="row in splitDiffRows"
              :key="`${row.left.kind}:${row.left.line}:${row.right.kind}:${row.right.line}`"
              :class="['inspector-split-line', `left-${row.left.kind}`, `right-${row.right.kind}`]"
            >
              <span class="inspector-diff-num">{{ row.left.line ?? "" }}</span>
              <code :class="['inspector-diff-code', 'inspector-split-code', row.left.kind]"
                ><span
                  v-for="token in withTokenOffsets(row.left.tokens)"
                  :key="token.offset"
                  :style="{ '--d': token.darkColor, '--l': token.lightColor }"
                  >{{ token.content }}</span
                ></code
              >
              <span class="inspector-diff-num">{{ row.right.line ?? "" }}</span>
              <code :class="['inspector-diff-code', 'inspector-split-code', row.right.kind]"
                ><span
                  v-for="token in withTokenOffsets(row.right.tokens)"
                  :key="token.offset"
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
                    v-for="token in withTokenOffsets(line.tokens)"
                    :key="token.offset"
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

<style>
.inspector-file-list {
  display: flex;
  gap: 4px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
  overflow-x: auto;
}

.inspector-file-tab {
  max-width: 220px;
  padding: 5px 10px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 12px;
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

.inspector-file-tab:hover,
.inspector-file-tab.active {
  background: var(--bg-tertiary);
  border-color: var(--border-color);
  color: var(--text-primary);
}

.inspector-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.inspector-targets {
  display: flex;
  gap: 4px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 2px;
}

.inspector-target {
  border: 0;
  background: transparent;
  color: var(--text-muted);
  border-radius: 4px;
  padding: 4px 10px;
  font-size: 12px;
  cursor: pointer;
}

.inspector-target.active {
  background: var(--bg-panel);
  color: var(--text-primary);
}

.inspector-pr-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 30px;
  padding: 5px 12px;
  border: 1px solid color-mix(in srgb, var(--text-primary) 28%, transparent);
  border-radius: 6px;
  background: var(--bg-tertiary);
  color: var(--text-primary);
  font-size: 12px;
  font-weight: 600;
  text-decoration: underline;
  text-underline-offset: 3px;
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background 0.15s ease;
}

.inspector-pr-link:hover {
  border-color: var(--text-primary);
  background: var(--bg-panel);
}

.inspector-pr-link svg {
  width: 15px;
  height: 15px;
  fill: currentColor;
  flex-shrink: 0;
}

.inspector-option {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  user-select: none;
}

.inspector-option input {
  accent-color: var(--accent-rust);
}

.inspector-option select {
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-panel);
  color: var(--text-primary);
  font: inherit;
  padding: 2px 6px;
}

.inspector-tab-panel {
  min-height: 0;
}

.inspector-diff-toolbar {
  display: inline-flex;
  gap: 4px;
  margin-bottom: 8px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-tertiary);
  padding: 2px;
}

.inspector-diff-mode {
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 12px;
  padding: 4px 10px;
}

.inspector-diff-mode.active {
  background: var(--bg-panel);
  color: var(--text-primary);
}

.inspector-diff {
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow: hidden;
  background: var(--bg-secondary);
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.55;
}

.inspector-diff-line {
  display: grid;
  grid-template-columns: 46px 46px 22px minmax(0, 1fr);
  min-height: 22px;
  border-bottom: 1px solid rgba(127, 127, 127, 0.08);
}

.inspector-diff-line:last-child {
  border-bottom: 0;
}

.inspector-diff-line.add {
  background: var(--color-success-bg);
}

.inspector-diff-line.remove {
  background: var(--color-error-bg);
}

.inspector-diff-line.same {
  background: transparent;
}

.inspector-diff-num,
.inspector-diff-mark {
  padding: 2px 6px;
  color: var(--text-muted);
  text-align: right;
  user-select: none;
}

.inspector-diff-mark {
  text-align: center;
}

.inspector-diff-split {
  overflow-x: auto;
}

.inspector-split-line {
  display: grid;
  grid-template-columns: 46px minmax(280px, 1fr) 46px minmax(280px, 1fr);
  min-width: 720px;
  min-height: 22px;
  border-bottom: 1px solid rgba(127, 127, 127, 0.08);
}

.inspector-split-line:last-child {
  border-bottom: 0;
}

.inspector-split-code {
  border-left: 1px solid rgba(127, 127, 127, 0.08);
}

.inspector-split-code.remove {
  background: var(--color-error-bg);
}

.inspector-split-code.add {
  background: var(--color-success-bg);
}

.inspector-split-code.empty {
  background: color-mix(in srgb, var(--bg-tertiary) 45%, transparent);
}

.inspector-diff-code {
  padding: 2px 10px;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-word;
}

.inspector-diff-code span {
  color: var(--l, inherit);
}

body[data-theme="dark"] .inspector-diff-code span {
  color: var(--d, inherit);
}

.inspector-empty-diff {
  padding: 24px;
  color: var(--text-muted);
  text-align: center;
}

.inspector-warning-list {
  margin-bottom: 12px;
}

.inspector-warning {
  padding: 8px 10px;
  border: 1px solid color-mix(in srgb, var(--color-warning) 35%, transparent);
  border-radius: 6px;
  background: var(--color-warning-bg);
  color: var(--color-warning);
  font-family: var(--font-mono);
  font-size: 12px;
  white-space: pre-wrap;
}

.inspector-warning + .inspector-warning {
  margin-top: 6px;
}

.inspector-payload {
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-secondary);
  padding: 12px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 12px;
  white-space: pre-wrap;
  word-break: break-word;
}

.inspector-vir-output {
  display: flex;
  min-height: 0;
  flex-direction: column;
}

.inspector-vir-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-bottom: 0;
  border-radius: 6px 6px 0 0;
  background: var(--bg-tertiary);
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 11px;
}

.inspector-vir-content {
  min-height: 280px;
  max-height: 58vh;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: 0 0 6px 6px;
  background: var(--bg-secondary);
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.55;
}

.inspector-vir-line {
  display: flex;
  white-space: pre;
}

.inspector-vir-ln {
  min-width: 42px;
  padding: 0 8px;
  border-right: 1px solid var(--border-color);
  color: var(--text-muted);
  text-align: right;
  user-select: none;
}

.inspector-vir-line-text {
  padding: 0 10px;
}

.vir-token {
  color: var(--text-secondary);
}

.vir-section,
.vir-section-name,
.vir-binding,
.vir-identifier {
  color: var(--text-primary);
}

.vir-macro {
  color: var(--accent-rust);
}

.vir-type,
.vir-source,
.vir-keyword,
.vir-number,
.vir-colon,
.vir-bracket,
.vir-tag,
.vir-arrow,
.vir-border {
  color: var(--text-muted);
}

.vir-line-diagnostic {
  background: var(--color-error-bg);
}

.inspector-graph-panel {
  display: flex;
  min-height: 0;
  flex-direction: column;
  gap: 10px;
}

.inspector-graph-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.inspector-graph-summary span,
.inspector-graph-kind,
.inspector-graph-edge-kind,
.inspector-graph-issues {
  border: 1px solid var(--border-color);
  border-radius: 999px;
  padding: 2px 8px;
  background: var(--bg-secondary);
  color: var(--text-muted);
  font-size: 11px;
}

.inspector-graph {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 8px;
}

.inspector-graph-node {
  min-width: 0;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-secondary);
  padding: 10px;
}

.inspector-graph-node.active {
  border-color: var(--accent-rust);
}

.inspector-graph-node.issues {
  border-color: color-mix(in srgb, var(--color-warning) 42%, var(--border-color));
}

.inspector-graph-node-main {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.inspector-graph-file {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.inspector-graph-issues {
  border-color: color-mix(in srgb, var(--color-warning) 55%, transparent);
  color: var(--color-warning);
}

.inspector-graph-node-meta {
  display: flex;
  gap: 8px;
  margin-top: 6px;
  color: var(--text-muted);
  font-size: 11px;
}

.inspector-graph-edges {
  margin-top: 10px;
  border-top: 1px solid var(--border-color);
  padding-top: 8px;
}

.inspector-graph-edge {
  display: grid;
  grid-template-columns: auto 20px minmax(0, 1fr);
  align-items: center;
  gap: 4px;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 11px;
}

.inspector-graph-edge + .inspector-graph-edge {
  margin-top: 5px;
}

.inspector-graph-target {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.inspector-graph-arrow {
  color: var(--text-muted);
  text-align: center;
}

.inspector-graph-diagnostics {
  display: flex;
  max-height: 220px;
  flex-direction: column;
  gap: 6px;
  overflow: auto;
}

.inspector-graph-diagnostic {
  display: grid;
  grid-template-columns: minmax(76px, auto) minmax(90px, 0.8fr) minmax(0, 1.4fr);
  gap: 8px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-secondary);
  padding: 8px;
  color: var(--text-secondary);
  font-size: 12px;
}

.inspector-graph-diagnostic.error {
  border-color: color-mix(in srgb, var(--color-error) 45%, var(--border-color));
}

.inspector-graph-diagnostic.warning {
  border-color: color-mix(in srgb, var(--color-warning) 45%, var(--border-color));
}

.inspector-graph-diagnostic-code,
.inspector-graph-diagnostic-file {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.inspector-graph-diagnostic-message {
  min-width: 0;
}

.inspector-url-note {
  margin-top: 8px;
  color: var(--color-warning);
  font-size: 12px;
}

@media (max-width: 900px) {
  .inspector-diff-line {
    grid-template-columns: 38px 38px 20px minmax(0, 1fr);
  }

  .inspector-graph-diagnostic {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
