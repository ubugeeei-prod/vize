<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, inject, type ComputedRef } from "vue";
import MonacoEditor from "../../shared/monaco-editor.vue";
import ComplexityPanel from "./complexity-panel.vue";
import type { Diagnostic } from "../../shared/monaco-editor.vue";
import type { CrossFileComplexityReport, WasmModule } from "../../wasm/index";
import { getWasm } from "../../wasm/index";
import { mdiClose, mdiCheck, mdiRefresh, mdiArrowRight, mdiPlus } from "@mdi/js";
import { PRESETS } from "../../shared/presets/crossfile";
import { useResizablePanel } from "./useResizablePanel";
import { useFileManagement } from "./useFileManagement";
import { useCrossFileAnalysis, type CrossFileAnalyzerOptions } from "./useCrossFileAnalysis";
import { getFileIcon, getSeverityIcon, getTypeLabel, getTypeColor } from "./displayHelpers";

const { compiler: _compiler } = defineProps<{
  compiler: WasmModule | null;
}>();
const _injectedTheme = inject<ComputedRef<"dark" | "light">>("theme");
const theme = computed<"dark" | "light">(() => _injectedTheme?.value ?? "light");

type AnalysisProfile = "signals" | "validation" | "all";
type SeverityFilter = "all" | "error" | "warning" | "info";

const PROFILE_OPTIONS: Record<AnalysisProfile, CrossFileAnalyzerOptions> = {
  signals: {
    provideInject: true,
    componentEmits: true,
    fallthroughAttrs: true,
    reactivityTracking: true,
    uniqueIds: true,
    serverClientBoundary: true,
    setupContext: false,
    circularDependencies: false,
    componentResolution: false,
    propsValidation: false,
  },
  validation: {
    provideInject: false,
    componentEmits: false,
    fallthroughAttrs: false,
    reactivityTracking: false,
    uniqueIds: false,
    serverClientBoundary: false,
    setupContext: true,
    circularDependencies: true,
    componentResolution: true,
    propsValidation: true,
  },
  all: {
    provideInject: true,
    componentEmits: true,
    fallthroughAttrs: true,
    reactivityTracking: true,
    uniqueIds: true,
    serverClientBoundary: true,
    setupContext: true,
    circularDependencies: true,
    componentResolution: true,
    propsValidation: true,
  },
};

const analysisProfile = ref<AnalysisProfile>("all");
const severityFilter = ref<SeverityFilter>("all");
const complexityReport = ref<CrossFileComplexityReport | null>(null);

const options = ref<CrossFileAnalyzerOptions>({
  provideInject: true,
  componentEmits: true,
  fallthroughAttrs: true,
  reactivityTracking: true,
  uniqueIds: true,
  serverClientBoundary: true,
  setupContext: true,
  circularDependencies: true,
  componentResolution: true,
  propsValidation: true,
});

const {
  containerRef,
  isResizingSidebar,
  isResizingDiagnostics,
  gridStyle,
  startSidebarResize,
  startDiagnosticsResize,
} = useResizablePanel();

const {
  currentPreset,
  files,
  activeFile,
  monacoEditorRef,
  fileNames,
  croquisResults,
  crossFileIssues,
  analysisTime,
  isAnalyzing,
  selectedIssue,
  currentSource,
  editorLanguage,
  dependencyGraph,
  addFile,
  removeFile,
  resetProject,
  selectPreset,
  selectIssue,
  selectFile,
} = useFileManagement();

const { analyzeAll } = useCrossFileAnalysis({
  files,
  croquisResults,
  crossFileIssues,
  analysisTime,
  isAnalyzing,
  dependencyGraph,
  complexityReport,
  options,
});

const visibleIssues = computed(() => {
  if (severityFilter.value === "all") {
    return crossFileIssues.value;
  }
  return crossFileIssues.value.filter((issue) => issue.severity === severityFilter.value);
});

const currentDiagnostics = computed((): Diagnostic[] => {
  return visibleIssues.value
    .filter((issue) => issue.file === activeFile.value)
    .map((issue) => ({
      message: `[${issue.code}] ${issue.message}${issue.suggestion ? `\n\nTip: ${issue.suggestion}` : ""}`,
      startLine: issue.line,
      startColumn: issue.column,
      endLine: issue.endLine,
      endColumn: issue.endColumn,
      severity: issue.severity,
    }));
});

const issuesByFile = computed(() => {
  const grouped: Record<string, typeof visibleIssues.value> = {};
  for (const issue of visibleIssues.value) {
    if (!grouped[issue.file]) grouped[issue.file] = [];
    grouped[issue.file].push(issue);
  }
  return grouped;
});

const issuesByType = computed(() => {
  const grouped: Record<string, typeof visibleIssues.value> = {};
  for (const issue of visibleIssues.value) {
    if (!grouped[issue.type]) grouped[issue.type] = [];
    grouped[issue.type].push(issue);
  }
  return grouped;
});

const stats = computed(() => ({
  files: Object.keys(files.value).length,
  totalIssues: visibleIssues.value.length,
  errors: visibleIssues.value.filter((i) => i.severity === "error").length,
  warnings: visibleIssues.value.filter((i) => i.severity === "warning").length,
  infos: visibleIssues.value.filter((i) => i.severity === "info").length,
}));

const analyzerCount = computed(() => Object.values(options.value).filter(Boolean).length);
const selectedProfile = computed<AnalysisProfile | "custom">(() => {
  for (const profile of ["signals", "validation", "all"] as const) {
    if (optionsEqual(options.value, PROFILE_OPTIONS[profile])) {
      return profile;
    }
  }
  return "custom";
});
const croquisReadyCount = computed(
  () => Object.values(croquisResults.value).filter((result) => result !== null).length,
);
const diagnosticsEmptyLabel = computed(() =>
  crossFileIssues.value.length === 0 ? "No issues detected" : "No matching diagnostics",
);

function setAnalysisProfile(profile: AnalysisProfile) {
  analysisProfile.value = profile;
  options.value = { ...PROFILE_OPTIONS[profile] };
}

function setSeverityFilter(filter: SeverityFilter) {
  severityFilter.value = filter;
}

function optionsEqual(left: CrossFileAnalyzerOptions, right: CrossFileAnalyzerOptions) {
  return (Object.keys(right) as Array<keyof CrossFileAnalyzerOptions>).every(
    (key) => left[key] === right[key],
  );
}

let analyzeTimeout: ReturnType<typeof setTimeout> | null = null;

function debouncedAnalyze() {
  if (analyzeTimeout) clearTimeout(analyzeTimeout);
  analyzeTimeout = setTimeout(() => {
    analyzeAll();
  }, 300);
}

watch(
  [files, options],
  () => {
    debouncedAnalyze();
  },
  { deep: true },
);

function setEditorValue(source: string) {
  const el = document.querySelector(".editor-content .monaco-container") as any;
  if (el?.__monacoEditor) {
    el.__monacoEditor.setValue(source);
  }
}

function handleFileClick(event: Event) {
  const el = event.currentTarget as HTMLElement;
  const name =
    el.querySelector(".tab-name, .file-name, .dep-target")?.textContent?.trim() ||
    el.textContent?.trim() ||
    "";
  if (name && files.value[name]) {
    selectFile(name);
    setEditorValue(files.value[name]);
  }
}

function handleSelectIssue(issue: (typeof crossFileIssues.value)[0]) {
  selectIssue(issue);
  setEditorValue(files.value[issue.file] ?? "");
}

function handleSelectPreset(event: Event) {
  const el = event.currentTarget as HTMLElement;
  const name = el.querySelector(".preset-name")?.textContent?.trim();
  const preset = PRESETS.find((p) => p.name === name);
  if (preset) {
    selectPreset(preset.id, analyzeAll);
    setEditorValue(currentSource.value);
  }
}

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
    analyzeAll();
  }
}

onMounted(() => {
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
  if (pollInterval) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
});
</script>

<template>
  <div
    ref="containerRef"
    class="cross-file-playground"
    :style="gridStyle"
    :class="{ resizing: isResizingSidebar || isResizingDiagnostics }"
  >
    <!-- Sidebar: File Tree & Dependency Graph -->
    <aside class="sidebar" aria-label="File tree and dependency graph">
      <!-- Preset Selector -->
      <div class="sidebar-section preset-section">
        <div class="section-header">
          <h3>Presets</h3>
        </div>
        <div class="preset-list">
          <button
            v-for="preset in PRESETS"
            :key="preset.id"
            type="button"
            :class="['preset-item', { active: currentPreset === preset.id }]"
            :title="preset.description"
            @click="($event) => handleSelectPreset($event)"
          >
            <svg class="preset-icon" viewBox="0 0 24 24">
              <path :d="preset.icon" fill="currentColor" />
            </svg>
            <span class="preset-name">{{ preset.name }}</span>
          </button>
        </div>
      </div>

      <div class="sidebar-section">
        <div class="section-header">
          <h3>Project Files</h3>
          <div class="section-actions">
            <button type="button" class="icon-btn" title="Add file" @click="addFile">
              <svg viewBox="0 0 24 24"><path :d="mdiPlus" fill="currentColor" /></svg>
            </button>
            <button type="button" class="icon-btn" title="Reset" @click="resetProject">
              <svg viewBox="0 0 24 24"><path :d="mdiRefresh" fill="currentColor" /></svg>
            </button>
          </div>
        </div>
        <nav class="file-tree">
          <div
            v-for="name in fileNames"
            :key="name"
            role="button"
            tabindex="0"
            :class="[
              'file-item',
              {
                active: activeFile === name,
                'has-errors': issuesByFile[name]?.some((i) => i.severity === 'error'),
                'has-warnings': issuesByFile[name]?.some((i) => i.severity === 'warning'),
              },
            ]"
            @click="($event) => handleFileClick($event)"
            @keydown.enter="($event) => handleFileClick($event)"
          >
            <svg class="file-icon" viewBox="0 0 24 24">
              <path :d="getFileIcon(name)" fill="currentColor" />
            </svg>
            <span class="file-name">{{ name }}</span>
            <span
              v-if="issuesByFile[name]?.length"
              class="file-badge"
              :class="issuesByFile[name].some((i) => i.severity === 'error') ? 'error' : 'warning'"
            >
              <span class="badge-count">{{ issuesByFile[name].length }}</span>
            </span>
            <button
              v-if="fileNames.length > 1"
              type="button"
              class="file-delete"
              @click.stop="() => removeFile(name)"
            >
              <svg viewBox="0 0 24 24"><path :d="mdiClose" fill="currentColor" /></svg>
            </button>
          </div>
        </nav>
      </div>

      <div class="sidebar-section">
        <div class="section-header">
          <h3>Dependencies</h3>
        </div>
        <div class="dependency-graph">
          <div v-for="(deps, file) in dependencyGraph" :key="file" class="dep-node">
            <span class="dep-file">{{ file }}</span>
            <div v-if="deps.length" class="dep-arrows">
              <div v-for="dep in deps" :key="dep" class="dep-edge">
                <svg class="dep-arrow" viewBox="0 0 24 24">
                  <path :d="mdiArrowRight" fill="currentColor" />
                </svg>
                <span
                  role="button"
                  tabindex="0"
                  class="dep-target"
                  @click="($event) => handleFileClick($event)"
                  @keydown.enter="($event) => handleFileClick($event)"
                  >{{ dep }}</span
                >
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="sidebar-section options-section">
        <div class="section-header">
          <h3>Scope</h3>
          <span class="analysis-mode-badge" :title="`${analyzerCount} analyzers enabled`">{{
            selectedProfile.toUpperCase()
          }}</span>
        </div>
        <div class="profile-switch" role="group" aria-label="Analysis profile">
          <button
            type="button"
            :class="['profile-btn', { active: selectedProfile === 'signals' }]"
            title="Relationship and reactivity signals"
            @click="() => setAnalysisProfile('signals')"
          >
            Signals
          </button>
          <button
            type="button"
            :class="['profile-btn', { active: selectedProfile === 'validation' }]"
            title="Project validation checks"
            @click="() => setAnalysisProfile('validation')"
          >
            Validation
          </button>
          <button
            type="button"
            :class="['profile-btn', { active: selectedProfile === 'all' }]"
            title="All project diagnostics"
            @click="() => setAnalysisProfile('all')"
          >
            All
          </button>
        </div>
        <div class="options-grid">
          <label class="option-toggle">
            <input v-model="options.provideInject" type="checkbox" />
            <span>Provide/Inject</span>
          </label>
          <label class="option-toggle">
            <input v-model="options.componentEmits" type="checkbox" />
            <span>Component Emits</span>
          </label>
          <label class="option-toggle">
            <input v-model="options.fallthroughAttrs" type="checkbox" />
            <span>Fallthrough Attrs</span>
          </label>
          <label class="option-toggle">
            <input v-model="options.reactivityTracking" type="checkbox" />
            <span>Reactivity</span>
          </label>
          <label class="option-toggle">
            <input v-model="options.uniqueIds" type="checkbox" />
            <span>Unique IDs</span>
          </label>
          <label class="option-toggle">
            <input v-model="options.serverClientBoundary" type="checkbox" />
            <span>SSR Boundary</span>
          </label>
          <label class="option-toggle">
            <input v-model="options.setupContext" type="checkbox" />
            <span>Setup Context</span>
          </label>
          <label class="option-toggle">
            <input v-model="options.circularDependencies" type="checkbox" />
            <span>Circular Deps</span>
          </label>
          <label class="option-toggle">
            <input v-model="options.componentResolution" type="checkbox" />
            <span>Component Resolution</span>
          </label>
          <label class="option-toggle">
            <input v-model="options.propsValidation" type="checkbox" />
            <span>Props Validation</span>
          </label>
        </div>
      </div>
    </aside>

    <!-- Resize Handle: Sidebar -->
    <div
      role="button"
      tabindex="0"
      class="resize-handle resize-handle-left"
      @mousedown="startSidebarResize"
      @keydown.enter="startSidebarResize"
    ></div>

    <!-- Main Content: Editor -->
    <main class="editor-pane">
      <div class="editor-header">
        <div class="editor-tabs">
          <button
            v-for="name in fileNames"
            :key="name"
            type="button"
            :class="['editor-tab', { active: activeFile === name }]"
            @click="($event) => handleFileClick($event)"
          >
            <svg class="tab-icon" viewBox="0 0 24 24">
              <path :d="getFileIcon(name)" fill="currentColor" />
            </svg>
            <span class="tab-name">{{ name }}</span>
            <span
              v-if="issuesByFile[name]?.length"
              class="tab-badge"
              :class="issuesByFile[name].some((i) => i.severity === 'error') ? 'error' : 'warning'"
            >
              <span class="badge-count">{{ issuesByFile[name].length }}</span>
            </span>
          </button>
        </div>
        <div class="editor-status">
          <span class="status-chip">Croquis {{ croquisReadyCount }}/{{ stats.files }}</span>
          <span class="status-chip">{{ analyzerCount }} checks</span>
          <span v-if="isAnalyzing" class="status-analyzing">Analyzing...</span>
          <span v-else class="status-time">{{ analysisTime.toFixed(1) }}ms</span>
        </div>
      </div>
      <div class="editor-content">
        <MonacoEditor
          ref="monacoEditorRef"
          v-model="currentSource"
          :language="editorLanguage"
          :diagnostics="currentDiagnostics"
          :theme
        />
      </div>
    </main>

    <!-- Resize Handle: Diagnostics -->
    <div
      role="button"
      tabindex="0"
      class="resize-handle resize-handle-right"
      @mousedown="startDiagnosticsResize"
      @keydown.enter="startDiagnosticsResize"
    ></div>

    <!-- Right Panel: Diagnostics -->
    <aside class="diagnostics-pane" aria-label="Diagnostics">
      <ComplexityPanel :report="complexityReport" />
      <div class="diagnostics-header">
        <h3>Diagnostics</h3>
        <div class="diagnostics-stats">
          <button
            type="button"
            :class="['filter-chip', { active: severityFilter === 'all' }]"
            @click="() => setSeverityFilter('all')"
          >
            {{ crossFileIssues.length }} all
          </button>
          <span v-if="stats.errors" class="stat-chip error">{{ stats.errors }} errors</span>
          <span v-if="stats.warnings" class="stat-chip warning">{{ stats.warnings }} warnings</span>
          <span v-if="stats.infos" class="stat-chip info">{{ stats.infos }} info</span>
        </div>
      </div>
      <div class="severity-filters" role="group" aria-label="Severity filter">
        <button
          type="button"
          :class="['filter-btn error', { active: severityFilter === 'error' }]"
          @click="() => setSeverityFilter('error')"
        >
          Errors
        </button>
        <button
          type="button"
          :class="['filter-btn warning', { active: severityFilter === 'warning' }]"
          @click="() => setSeverityFilter('warning')"
        >
          Warnings
        </button>
        <button
          type="button"
          :class="['filter-btn info', { active: severityFilter === 'info' }]"
          @click="() => setSeverityFilter('info')"
        >
          Info
        </button>
      </div>

      <div v-if="visibleIssues.length === 0" class="diagnostics-empty">
        <svg class="empty-icon" viewBox="0 0 24 24"><path :d="mdiCheck" fill="currentColor" /></svg>
        <span>{{ diagnosticsEmptyLabel }}</span>
      </div>

      <div v-else class="diagnostics-list">
        <!-- Group by type -->
        <div v-for="(issues, type) in issuesByType" :key="type" class="issue-group">
          <div class="group-header" :style="{ '--type-color': getTypeColor(type) }">
            <span class="group-badge">{{ getTypeLabel(type) }}</span>
            <span class="group-count">{{ issues.length }}</span>
          </div>
          <div class="group-issues">
            <div
              v-for="issue in issues"
              :key="issue.id"
              role="button"
              tabindex="0"
              :class="['issue-card', issue.severity, { selected: selectedIssue?.id === issue.id }]"
              @click="() => handleSelectIssue(issue)"
              @keydown.enter="() => handleSelectIssue(issue)"
            >
              <div class="issue-header">
                <svg class="severity-icon" viewBox="0 0 24 24">
                  <path :d="getSeverityIcon(issue.severity)" fill="currentColor" />
                </svg>
                <span class="issue-code">{{ issue.code }}</span>
                <span class="issue-location">{{ issue.file }}:{{ issue.line }}</span>
              </div>
              <div class="issue-message">{{ issue.message }}</div>
              <div v-if="issue.suggestion" class="issue-suggestion">
                <svg class="suggestion-icon" viewBox="0 0 24 24">
                  <path :d="mdiArrowRight" fill="currentColor" />
                </svg>
                <span class="suggestion-text">{{ issue.suggestion }}</span>
              </div>
              <div v-if="issue.relatedLocations?.length" class="issue-related">
                <div
                  v-for="rel in issue.relatedLocations"
                  :key="`${rel.file}:${rel.line}:${rel.message}`"
                  class="related-item"
                >
                  <span class="related-loc">{{ rel.file }}:{{ rel.line }}</span>
                  <span class="related-msg">{{ rel.message }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </aside>
  </div>
</template>

<style>
.cross-file-playground {
  display: grid;
  grid-template-columns: 220px 4px 1fr 4px 320px;
  grid-column: 1 / -1;
  height: 100%;
  min-height: 0;
  background: var(--bg-primary);
  font-size: 12px;
  user-select: none;
}

.cross-file-playground.resizing {
  cursor: col-resize;
}

.cross-file-playground.resizing * {
  pointer-events: none;
}

/* === Resize Handles === */
.resize-handle {
  width: 4px;
  background: var(--border-primary);
  cursor: col-resize;
  transition: background 0.15s;
  position: relative;
}

.resize-handle:hover,
.resize-handle:active {
  background: var(--accent-primary);
}

.resize-handle::after {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  width: 8px;
  left: -2px;
}

/* === Sidebar === */
.sidebar {
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-primary);
  overflow: hidden;
}

.sidebar-section {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.sidebar-section:not(:last-child) {
  border-bottom: 1px solid var(--border-primary);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--bg-tertiary);
}

.section-header h3 {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-muted);
  margin: 0;
}

.analysis-mode-badge {
  font-size: 9px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  background: var(--color-success);
  color: #fff;
  letter-spacing: 0.5px;
  cursor: help;
}

.profile-switch {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 4px;
  padding: 8px 12px 0;
}

.profile-btn {
  min-width: 0;
  height: 24px;
  padding: 0 4px;
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 10px;
  font-weight: 600;
}

.profile-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.profile-btn.active {
  border-color: var(--accent-rust);
  color: var(--accent-rust);
  background: rgba(224, 112, 72, 0.12);
}

/* Preset Selector */
.preset-section {
  flex-shrink: 0;
}

.preset-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 8px;
}

.preset-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 10px;
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  cursor: pointer;
  font-size: 10px;
  color: var(--text-secondary);
  transition: all 0.15s;
  flex: 1;
  min-width: calc(50% - 4px);
  justify-content: flex-start;
}

.preset-item:hover {
  background: var(--bg-tertiary);
  border-color: var(--text-muted);
  color: var(--text-primary);
}

.preset-item.active {
  background: rgba(224, 112, 72, 0.15);
  border-color: var(--accent-rust);
  color: var(--accent-rust);
}

.preset-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.preset-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.section-actions {
  display: flex;
  gap: 4px;
}

.icon-btn svg {
  width: 14px;
  height: 14px;
}

.icon-btn {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid var(--border-primary);
  border-radius: 3px;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s;
}

.icon-btn:hover {
  background: var(--bg-primary);
  color: var(--text-primary);
  border-color: var(--text-muted);
}

/* File Tree */
.file-tree {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  cursor: pointer;
  transition: background 0.1s;
  position: relative;
}

.file-item:hover {
  background: var(--bg-tertiary);
}

.file-item.active {
  background: var(--accent-primary);
  background: rgba(224, 112, 72, 0.15);
}

.file-item.has-errors .file-icon {
  color: var(--color-error);
}
.file-item.has-warnings .file-icon {
  color: var(--color-warning);
}

.file-icon {
  width: 12px;
  height: 12px;
  color: var(--accent-rust);
  flex-shrink: 0;
}

.file-name {
  flex: 1;
  font-family: "JetBrains Mono", monospace;
  font-size: 11px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-badge {
  font-size: 8px;
  padding: 0 3px;
  border-radius: 4px;
  font-weight: 600;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  height: 12px;
}

.file-badge.error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.file-badge.warning {
  background: var(--color-warning-bg);
  color: var(--color-warning);
}

.file-delete {
  position: absolute;
  right: 8px;
  width: 16px;
  height: 16px;
  display: none;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 2px;
}

.file-delete svg {
  width: 12px;
  height: 12px;
}

.file-item:hover .file-delete {
  display: flex;
}

.file-delete:hover {
  background: var(--color-error-bg);
  color: var(--color-error);
}

/* Dependency Graph */
.dependency-graph {
  padding: 8px 12px;
  font-family: "JetBrains Mono", monospace;
  font-size: 10px;
}

.dep-node {
  margin-bottom: 8px;
}

.dep-file {
  color: var(--text-secondary);
}

.dep-arrows {
  padding-left: 12px;
  margin-top: 2px;
}

.dep-edge {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--text-muted);
}

.dep-arrow {
  width: 12px;
  height: 12px;
  color: var(--accent-rust);
}

.dep-target {
  color: var(--text-secondary);
  cursor: pointer;
}

.dep-target:hover {
  color: var(--accent-rust);
  text-decoration: underline;
}

/* Options */
.options-section {
  margin-top: auto;
}

.options-grid {
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.option-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 11px;
  color: var(--text-secondary);
}

.option-toggle input {
  width: 12px;
  height: 12px;
  accent-color: var(--accent-primary);
}

.option-toggle:hover {
  color: var(--text-primary);
}

/* === Editor Pane === */
.editor-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-primary);
  padding-right: 12px;
}

.editor-tabs {
  display: flex;
  overflow-x: auto;
}

.editor-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: transparent;
  border: none;
  border-right: 1px solid var(--border-primary);
  font-size: 11px;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.1s;
}

.editor-tab:hover {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
}

.editor-tab.active {
  background: var(--bg-primary);
  color: var(--text-primary);
  border-bottom: 2px solid var(--accent-rust);
  margin-bottom: -1px;
}

.tab-icon {
  width: 12px;
  height: 12px;
  color: var(--accent-rust);
  flex-shrink: 0;
}

.tab-badge {
  font-size: 8px;
  padding: 0 3px;
  border-radius: 4px;
  font-weight: 600;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  height: 12px;
}

.tab-badge.error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.tab-badge.warning {
  background: var(--color-warning-bg);
  color: var(--color-warning);
}

.editor-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  font-family: "JetBrains Mono", monospace;
}

.status-chip {
  height: 18px;
  padding: 0 6px;
  border: 1px solid var(--border-primary);
  border-radius: 3px;
  display: inline-flex;
  align-items: center;
  color: var(--text-muted);
  background: var(--bg-primary);
}

.status-analyzing {
  color: var(--accent-rust);
}

.status-time {
  color: var(--text-muted);
}

.editor-content {
  flex: 1;
  min-height: 0;
  position: relative;
}

/* === Diagnostics Pane === */
.diagnostics-pane {
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-primary);
  overflow: hidden;
}

.diagnostics-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 10px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-primary);
}

.diagnostics-header h3 {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-muted);
  margin: 0;
}

.diagnostics-stats {
  display: flex;
  gap: 6px;
  align-items: center;
}

.stat-chip {
  font-size: 9px;
  padding: 0 3px;
  border-radius: 3px;
  font-weight: 600;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  height: 14px;
}

.stat-chip.error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.stat-chip.warning {
  background: var(--color-warning-bg);
  color: var(--color-warning);
}

.stat-chip.info {
  background: var(--color-info-bg);
  color: var(--color-info);
}

.filter-chip {
  height: 16px;
  padding: 0 6px;
  border: 1px solid var(--border-primary);
  border-radius: 3px;
  background: var(--bg-primary);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 9px;
  font-weight: 600;
}

.filter-chip.active {
  border-color: var(--accent-rust);
  color: var(--accent-rust);
}

.severity-filters {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 4px;
  padding: 6px 8px;
  border-bottom: 1px solid var(--border-primary);
}

.filter-btn {
  min-width: 0;
  height: 22px;
  padding: 0 4px;
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 10px;
  font-weight: 600;
}

.filter-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.filter-btn.active.error {
  border-color: var(--color-error);
  color: var(--color-error);
  background: var(--color-error-bg);
}

.filter-btn.active.warning {
  border-color: var(--color-warning);
  color: var(--color-warning);
  background: var(--color-warning-bg);
}

.filter-btn.active.info {
  border-color: var(--color-info);
  color: var(--color-info);
  background: var(--color-info-bg);
}

.diagnostics-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 32px;
  color: var(--color-success);
}

.empty-icon {
  width: 24px;
  height: 24px;
}

.diagnostics-list {
  flex: 1;
  overflow-y: auto;
  padding: 6px;
}

.issue-group {
  margin-bottom: 8px;
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  margin-bottom: 4px;
  background: var(--bg-tertiary);
  border-radius: 4px;
}

.group-badge {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-secondary);
}

.group-count {
  font-size: 10px;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

.group-issues {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.issue-card {
  padding: 5px 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.1s;
}

.issue-card:hover {
  background: var(--bg-tertiary);
}

.issue-card.selected {
  border-color: var(--accent-rust);
  background: rgba(224, 112, 72, 0.1);
}

.issue-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}

.severity-icon {
  width: 12px;
  height: 12px;
  flex-shrink: 0;
}

.issue-card.error .severity-icon {
  color: var(--color-error);
}
.issue-card.warning .severity-icon {
  color: var(--color-warning);
}
.issue-card.info .severity-icon {
  color: var(--color-info);
}

.issue-code {
  font-size: 9px;
  font-family: "JetBrains Mono", monospace;
  padding: 1px 4px;
  background: var(--bg-secondary);
  border-radius: 2px;
  color: var(--text-muted);
}

.issue-location {
  margin-left: auto;
  font-size: 9px;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
}

.issue-message {
  font-size: 11px;
  color: var(--text-primary);
  line-height: 1.4;
}

.issue-suggestion {
  margin-top: 4px;
  padding: 4px 6px;
  font-size: 10px;
  color: var(--color-success);
  background: var(--color-success-bg);
  border-radius: 3px;
  display: flex;
  gap: 6px;
}

.suggestion-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.issue-related {
  margin-top: 6px;
  padding-top: 6px;
  border-top: 1px solid var(--border-primary);
}

.related-item {
  display: flex;
  gap: 8px;
  font-size: 10px;
  color: var(--text-muted);
  margin-bottom: 2px;
}

.related-loc {
  font-family: "JetBrains Mono", monospace;
  color: var(--text-secondary);
}

/* === Responsive === */
@media (max-width: 1200px) {
  .cross-file-playground {
    grid-template-columns: 180px 1fr 280px;
  }
}

@media (max-width: 900px) {
  .cross-file-playground {
    grid-template-columns: 1fr;
    grid-template-rows: auto 1fr auto;
  }

  .sidebar {
    flex-direction: row;
    border-right: none;
    border-bottom: 1px solid var(--border-primary);
    overflow-x: auto;
  }

  .sidebar-section {
    flex-direction: row;
    min-width: max-content;
  }

  .diagnostics-pane {
    border-left: none;
    border-top: 1px solid var(--border-primary);
    max-height: 300px;
  }
}
</style>
