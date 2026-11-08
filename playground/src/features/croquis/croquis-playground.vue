<script setup lang="ts">
import CroquisSource from "./croquis-source.vue";
import { type WasmModule, getWasm } from "../../wasm/index";
import { mdiChartTimelineVariant, mdiCheck, mdiCloseCircle, mdiAlert } from "@mdi/js";
import { useCroquisAnalysis } from "./useCroquisAnalysis";
import CroquisStatsPanel from "./croquis-stats-panel.vue";
import ReactivityOverlayPanel from "./reactivity-overlay-panel.vue";
import { getSourceLabel, getSourceClass } from "./bindingHelpers";
import { getScopeColorClass } from "./scopeColors";
import { withTokenOffsets } from "../../utils/withTokenOffsets";

const { compiler: compilerProp } = defineProps<{
  compiler: WasmModule | null;
}>();

const {
  experimentals,
  loadExample,
  theme,
  source,
  error,
  activeTab,
  showScopeVisualization,
  editorRef,
  analysisTime,
  analysisResult,
  scopes,
  editorScopes,
  bindings,
  macros,
  css,
  typeExports,
  invalidExports,
  reactivityOverlay,
  diagnostics,
  stats,
  monacoDiagnostics,
  bindingsBySource,
  virLines,
} = useCroquisAnalysis(() => compilerProp ?? getWasm());
</script>

<template>
  <div class="croquis-playground">
    <div class="panel input-panel">
      <CroquisSource
        ref="editorRef"
        v-model="source"
        v-model:visualize="showScopeVisualization"
        v-model:experimentals="experimentals"
        :scopes="editorScopes"
        :diagnostics="monacoDiagnostics"
        :theme
        @example="loadExample"
      />
    </div>

    <div class="panel output-panel">
      <div class="panel-header">
        <div class="header-title">
          <svg class="icon" viewBox="0 0 24 24">
            <path :d="mdiChartTimelineVariant" fill="currentColor" />
          </svg>
          <h2>Semantic Analysis</h2>
          <span v-if="analysisTime !== null" class="perf-badge">
            {{ analysisTime.toFixed(2) }}ms
          </span>
        </div>
        <div class="tabs">
          <button
            type="button"
            :class="['tab', { active: activeTab === 'vir' }]"
            @click="() => (activeTab = 'vir')"
          >
            VIR
          </button>
          <button
            type="button"
            :class="['tab', { active: activeTab === 'stats' }]"
            @click="() => (activeTab = 'stats')"
          >
            Stats
          </button>
          <button
            type="button"
            :class="['tab', { active: activeTab === 'reactivity' }]"
            @click="() => (activeTab = 'reactivity')"
          >
            Reactivity
            <span v-if="reactivityOverlay?.summary.lossCount" class="tab-badge">
              {{ reactivityOverlay.summary.lossCount }}
            </span>
          </button>
          <button
            type="button"
            :class="['tab', { active: activeTab === 'bindings' }]"
            @click="() => (activeTab = 'bindings')"
          >
            Bindings
          </button>
          <button
            type="button"
            :class="['tab', { active: activeTab === 'scopes' }]"
            @click="() => (activeTab = 'scopes')"
          >
            Scopes
          </button>
          <button
            type="button"
            :class="['tab', { active: activeTab === 'diagnostics' }]"
            @click="() => (activeTab = 'diagnostics')"
          >
            Diagnostics
            <span v-if="diagnostics.length > 0" class="tab-badge">{{ diagnostics.length }}</span>
          </button>
        </div>
      </div>

      <div class="output-content">
        <div v-if="error" class="error-panel">
          <div class="error-header">Analysis Error</div>
          <pre class="error-content">{{ error }}</pre>
        </div>

        <template v-else-if="analysisResult">
          <!-- VIR Tab (Primary) -->
          <div v-if="activeTab === 'vir'" class="vir-output">
            <div class="vir-header-bar">
              <span class="vir-title">VIR — Vize Intermediate Representation</span>
              <span class="vir-line-count">{{ virLines.length }} lines</span>
            </div>
            <div class="vir-content">
              <div class="vir-code">
                <div
                  v-for="line in virLines"
                  :key="line.index"
                  :class="['vir-line', `vir-line-${line.lineType}`]"
                >
                  <span class="vir-ln">{{ line.index + 1 }}</span>
                  <span class="vir-line-text"
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
            </div>
            <div class="vir-notice">
              VIR is a human-readable display format for debugging purposes only. It is not portable
              and should not be parsed or used as a stable interface.
            </div>
          </div>

          <!-- Stats Tab -->
          <CroquisStatsPanel
            v-else-if="activeTab === 'stats'"
            :stats
            :macros
            :css
            :type-exports
            :invalid-exports
          />

          <!-- Reactivity Tab -->
          <ReactivityOverlayPanel
            v-else-if="activeTab === 'reactivity'"
            :overlay="reactivityOverlay"
          />

          <!-- Bindings Tab -->
          <div v-else-if="activeTab === 'bindings'" class="bindings-output">
            <div v-if="bindings.length === 0" class="empty-state">No bindings detected</div>

            <template v-else>
              <div v-for="(group, key) in bindingsBySource" :key class="source-group">
                <div class="source-header">
                  <span :class="['source-indicator', getSourceClass(String(key))]"></span>
                  <span class="source-name">{{ getSourceLabel(String(key)) }}</span>
                  <span class="source-count">{{ group.length }}</span>
                </div>
                <div class="binding-grid">
                  <div v-for="binding in group" :key="binding.name" class="binding-item">
                    <div class="binding-main">
                      <code class="binding-name">{{ binding.name }}</code>
                      <span
                        v-if="binding.metadata?.needsValue"
                        class="needs-value"
                        title="Needs .value"
                        >.value</span
                      >
                    </div>
                    <div class="binding-meta">
                      <span class="binding-kind">{{ binding.kind }}</span>
                      <span v-if="binding.typeAnnotation" class="binding-type"
                        >: {{ binding.typeAnnotation }}</span
                      >
                    </div>
                    <div class="binding-flags">
                      <span
                        :class="['flag', binding.bindable ? 'active' : 'inactive']"
                        title="Can be referenced from template"
                        >bindable</span
                      >
                      <span
                        :class="['flag', binding.usedInTemplate ? 'active' : 'inactive']"
                        title="Actually used in template"
                        >in-template</span
                      >
                      <span :class="['flag', binding.isMutated ? 'active' : 'inactive']"
                        >mutated</span
                      >
                      <span
                        v-if="binding.fromScriptSetup"
                        class="flag setup"
                        title="From script setup"
                        >setup</span
                      >
                      <span class="refs">{{ binding.referenceCount }} refs</span>
                    </div>
                  </div>
                </div>
              </div>
            </template>
          </div>

          <!-- Scopes Tab -->
          <div v-else-if="activeTab === 'scopes'" class="scopes-output">
            <div v-if="scopes.length === 0" class="empty-state">No scopes detected</div>

            <div v-else class="scope-tree">
              <div
                v-for="scope in scopes"
                :key="scope.id"
                :class="['scope-node', getScopeColorClass(scope.kindStr || scope.kind)]"
                :style="{ marginLeft: `${(scope.depth || 0) * 20}px` }"
              >
                <div class="scope-header">
                  <span
                    :class="['scope-indicator', getScopeColorClass(scope.kindStr || scope.kind)]"
                  ></span>
                  <span class="scope-kind">{{ scope.kindStr || scope.kind }}</span>
                  <span class="scope-range">[{{ scope.start }}:{{ scope.end }}]</span>
                </div>
                <div v-if="scope.bindings.length > 0" class="scope-bindings">
                  <span v-for="name in scope.bindings" :key="name" class="scope-binding">{{
                    name
                  }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Diagnostics Tab -->
          <div v-else-if="activeTab === 'diagnostics'" class="diagnostics-output">
            <div v-if="diagnostics.length === 0" class="success-state">
              <svg class="success-icon" viewBox="0 0 24 24">
                <path :d="mdiCheck" fill="currentColor" />
              </svg>
              <span>No issues found</span>
            </div>

            <div v-else class="diagnostic-list">
              <div
                v-for="diag in diagnostics"
                :key="`${diag.start}:${diag.end}:${diag.code}:${diag.message}`"
                :class="['diagnostic-item', `severity-${diag.severity}`]"
              >
                <div class="diagnostic-header">
                  <svg class="severity-icon" viewBox="0 0 24 24">
                    <path
                      :d="diag.severity === 'error' ? mdiCloseCircle : mdiAlert"
                      fill="currentColor"
                    />
                  </svg>
                  <span class="diagnostic-message">{{ diag.message }}</span>
                </div>
                <div class="diagnostic-location">
                  <span class="location-range">{{ diag.start }}:{{ diag.end }}</span>
                  <span v-if="diag.code" class="diagnostic-code">[{{ diag.code }}]</span>
                </div>
              </div>
            </div>
          </div>
        </template>

        <div v-else class="loading-state">
          <span>Analyzing...</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
.croquis-playground {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0;
  height: 100%;
  min-height: 0;
  grid-column: 1 / -1;
  background: var(--bg-primary);
}

.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.input-panel {
  border-right: 1px solid var(--border-primary);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-primary);
  flex-shrink: 0;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.header-title .icon {
  width: 1rem;
  height: 1rem;
  color: var(--accent-rust);
}

.header-title h2 {
  font-size: 0.875rem;
  font-weight: 600;
  margin: 0;
}

.perf-badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.375rem;
  background: var(--bg-tertiary);
  color: var(--text-muted);
  border-radius: 3px;
  font-family: "JetBrains Mono", monospace;
}

.panel-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-ghost {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  background: transparent;
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-ghost:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  transition: all 0.15s;
}

.toggle-label:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.toggle-label input[type="checkbox"] {
  width: 14px;
  height: 14px;
  accent-color: var(--accent-primary);
}

.toggle-label span {
  white-space: nowrap;
}

.tabs {
  display: flex;
  gap: 0.125rem;
}

.tab {
  padding: 0.375rem 0.625rem;
  font-size: 0.75rem;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s;
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.tab:hover {
  color: var(--text-secondary);
  background: var(--bg-tertiary);
}

.tab.active {
  color: var(--text-primary);
  background: var(--bg-tertiary);
  font-weight: 500;
}

.tab-badge {
  font-size: 0.625rem;
  padding: 0.0625rem 0.3125rem;
  background: var(--color-error-bg);
  color: var(--color-error);
  border-radius: 8px;
  min-width: 1rem;
  text-align: center;
}

.editor-container {
  flex: 1;
  overflow: hidden;
}

.output-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* Error State */
.error-panel {
  background: var(--color-error-bg);
  border: 1px solid var(--color-error-border);
  border-radius: 6px;
  overflow: hidden;
  margin: 1rem;
}

.error-header {
  padding: 0.5rem 0.75rem;
  background: var(--color-error-bg);
  color: var(--color-error);
  font-size: 0.75rem;
  font-weight: 600;
}

.error-content {
  padding: 0.75rem;
  font-size: 0.75rem;
  color: var(--color-error);
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

/* Empty/Loading/Success States */
.empty-state,
.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  margin: 1rem;
  color: var(--text-muted);
  font-size: 0.875rem;
}

.success-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
  color: var(--color-success);
  font-size: 0.875rem;
}

.success-icon {
  width: 1.25rem;
  height: 1.25rem;
}

/* Stats Tab */
.stats-output,
.bindings-output,
.scopes-output,
.diagnostics-output {
  padding: 1rem;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 0.75rem;
  margin-bottom: 1.5rem;
}

.stat-box {
  padding: 1rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  text-align: center;
}

.stat-number {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--accent-rust);
  font-family: "JetBrains Mono", monospace;
}

.stat-label {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-top: 0.25rem;
}

.section {
  margin-bottom: 1.5rem;
}

.section-title {
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  margin-bottom: 0.75rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border-primary);
}

.macro-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.macro-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
}

.macro-name {
  font-weight: 600;
  font-family: "JetBrains Mono", monospace;
  color: var(--accent-rust);
}

.macro-type {
  font-size: 0.75rem;
  color: var(--color-info);
  background: var(--color-info-bg);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
}

.macro-binding {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.css-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.css-stat {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.css-badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.5rem;
  border-radius: 3px;
}

.css-badge.scoped {
  background: var(--color-purple-bg);
  color: var(--color-purple);
}

.css-badge.vbind {
  background: var(--color-teal-bg);
  color: var(--color-teal);
}

/* Export sections */
.export-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.export-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
}

.export-item.valid {
  background: var(--color-success-bg);
}

.export-item.invalid {
  background: var(--color-error-bg);
}

.export-kind {
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  background: var(--surface-elevated);
  color: var(--text-muted);
}

.export-name {
  font-family: var(--font-mono);
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-primary);
}

.export-badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.5rem;
  border-radius: 3px;
  margin-left: auto;
}

.export-badge.hoisted {
  background: var(--color-success-bg);
  color: var(--color-success);
}

.export-badge.error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  margin-left: 0.5rem;
  vertical-align: middle;
}

.badge.hoisted {
  background: var(--color-success-bg);
  color: var(--color-success);
}

.badge.error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

/* Bindings Tab */
.source-group {
  margin-bottom: 1.25rem;
}

.source-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
  padding-bottom: 0.375rem;
  border-bottom: 1px solid var(--border-primary);
}

.source-indicator {
  width: 10px;
  height: 10px;
  border-radius: 2px;
}

.src-props {
  background: #a78bfa;
}
.src-emits {
  background: #f472b6;
}
.src-model {
  background: #fb923c;
}
.src-slots {
  background: #34d399;
}
.src-ref {
  background: #4ade80;
}
.src-reactive {
  background: #f87171;
}
.src-computed {
  background: #2dd4bf;
}
.src-import {
  background: #60a5fa;
}
.src-local {
  background: #94a3b8;
}
.src-function {
  background: #fbbf24;
}
.src-class {
  background: #818cf8;
}
.src-default {
  background: #6b7280;
}

.source-name {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-primary);
}

.source-count {
  font-size: 0.625rem;
  padding: 0.0625rem 0.375rem;
  background: var(--bg-tertiary);
  border-radius: 8px;
  color: var(--text-muted);
}

.binding-grid {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.binding-item {
  display: grid;
  grid-template-columns: 1fr auto auto;
  gap: 0.75rem;
  align-items: center;
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  font-size: 0.75rem;
}

.binding-main {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.binding-name {
  font-weight: 600;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-primary);
}

.needs-value {
  font-size: 0.625rem;
  color: var(--color-success);
  opacity: 0.7;
}

.binding-meta {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  color: var(--text-muted);
}

.binding-kind {
  font-family: "JetBrains Mono", monospace;
}

.binding-type {
  color: var(--color-info);
}

.binding-flags {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.flag {
  font-size: 0.5rem;
  text-transform: uppercase;
  padding: 0.0625rem 0.25rem;
  border-radius: 2px;
}

.flag.active {
  background: var(--color-success-bg);
  color: var(--color-success);
}

.flag.inactive {
  background: var(--bg-tertiary);
  color: var(--text-muted);
  opacity: 0.5;
}

.flag.setup {
  background: var(--color-purple-bg);
  color: var(--color-purple);
}

.refs {
  font-size: 0.625rem;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

/* Scopes Tab */
.scope-tree {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.scope-node {
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
}

.scope-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.scope-indicator {
  width: 8px;
  height: 8px;
  border-radius: 2px;
  flex-shrink: 0;
}

.scope-kind {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-primary);
}

.scope-range {
  font-size: 0.625rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
}

.scope-bindings {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  margin-top: 0.375rem;
  padding-left: 1.5rem;
}

.scope-binding {
  font-size: 0.625rem;
  padding: 0.0625rem 0.375rem;
  background: var(--bg-tertiary);
  border-radius: 3px;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-secondary);
}

/* Scope kind colors */
/* Template scopes */
.scope-module {
  background: rgba(167, 139, 250, 0.08);
}
.scope-module.scope-indicator {
  background: #a78bfa;
}

.scope-function {
  background: rgba(251, 191, 36, 0.08);
}
.scope-function.scope-indicator {
  background: #fbbf24;
}

.scope-block {
  background: rgba(148, 163, 184, 0.08);
}
.scope-block.scope-indicator {
  background: #94a3b8;
}

.scope-vfor {
  background: rgba(74, 222, 128, 0.08);
}
.scope-vfor.scope-indicator {
  background: #4ade80;
}

.scope-vslot {
  background: rgba(45, 212, 191, 0.08);
}
.scope-vslot.scope-indicator {
  background: #2dd4bf;
}

.scope-event-handler {
  background: rgba(244, 114, 182, 0.08);
}
.scope-event-handler.scope-indicator {
  background: #f472b6;
}

.scope-callback {
  background: rgba(251, 146, 60, 0.08);
}
.scope-callback.scope-indicator {
  background: #fb923c;
}

/* Script scopes */
.scope-script-setup {
  background: rgba(96, 165, 250, 0.08);
}
.scope-script-setup.scope-indicator {
  background: #60a5fa;
}

.scope-non-script-setup {
  background: rgba(129, 140, 248, 0.08);
}
.scope-non-script-setup.scope-indicator {
  background: #818cf8;
}

/* SSR scopes */
.scope-universal {
  background: rgba(34, 211, 238, 0.08);
}
.scope-universal.scope-indicator {
  background: #22d3ee;
}

.scope-client-only {
  background: rgba(248, 113, 113, 0.08);
}
.scope-client-only.scope-indicator {
  background: #f87171;
}

/* JS Global scopes (runtime-specific) */
.scope-js-global-universal {
  background: rgba(253, 224, 71, 0.08);
}
.scope-js-global-universal.scope-indicator {
  background: #fde047;
}

.scope-js-global-browser {
  background: rgba(251, 146, 60, 0.08);
}
.scope-js-global-browser.scope-indicator {
  background: #fb923c;
}

.scope-js-global-node {
  background: rgba(74, 222, 128, 0.08);
}
.scope-js-global-node.scope-indicator {
  background: #4ade80;
}

.scope-js-global-deno {
  background: rgba(96, 165, 250, 0.08);
}
.scope-js-global-deno.scope-indicator {
  background: #60a5fa;
}

.scope-js-global-bun {
  background: rgba(244, 114, 182, 0.08);
}
.scope-js-global-bun.scope-indicator {
  background: #f472b6;
}

/* Vue global */
.scope-vue-global {
  background: rgba(52, 211, 153, 0.08);
}
.scope-vue-global.scope-indicator {
  background: #34d399;
}

.scope-external-module {
  background: rgba(192, 132, 252, 0.08);
}
.scope-external-module.scope-indicator {
  background: #c084fc;
}

.scope-default {
  background: var(--bg-secondary);
}
.scope-default.scope-indicator {
  background: var(--text-muted);
}

/* VIR Tab */
.vir-output {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: 1rem;
}

.vir-header-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.375rem 0.75rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 4px 4px 0 0;
  border-bottom: none;
}

.vir-title {
  font-size: 0.6875rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.vir-line-count {
  font-size: 0.625rem;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
}

.vir-notice {
  padding: 0.5rem 0.75rem;
  background: var(--color-notice-bg);
  border: 1px solid var(--color-notice-border);
  font-size: 0.6875rem;
  color: var(--color-notice);
  line-height: 1.4;
}

.vir-content {
  flex: 1;
  min-height: 0;
  margin: 0;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-bottom: none;
  font-size: 0.8125rem;
  font-family: "JetBrains Mono", monospace;
  overflow: auto;
  line-height: 1.6;
}

.vir-ln {
  padding: 0 0.75rem;
  text-align: right;
  color: var(--text-muted);
  font-size: 0.8125rem;
  min-width: 2.5rem;
  opacity: 0.6;
  height: 1.6em;
  line-height: 1.6;
  display: inline-block;
  background: rgba(0, 0, 0, 0.15);
  border-right: 1px solid var(--border-primary);
  user-select: none;
}

.vir-code {
  flex: 1;
  padding: 0.75rem 0;
}

.vir-line {
  display: flex;
  white-space: pre;
  height: 1.6em;
  line-height: 1.6;
}

.vir-line-text {
  padding: 0 1rem;
}

/* VIR Token Types - Subtle syntax highlighting */
.vir-token {
  color: var(--text-secondary);
}

.vir-border {
  color: var(--text-muted);
  opacity: 0.5;
}

.vir-section {
  color: var(--text-primary);
  font-weight: 600;
}

.vir-section-name {
  color: var(--text-primary);
  font-weight: 600;
}

.vir-macro {
  color: var(--color-purple);
}

.vir-type {
  color: var(--text-muted);
  font-style: italic;
}

.vir-binding {
  color: var(--text-primary);
}

.vir-identifier {
  color: var(--text-primary);
}

.vir-tag {
  color: var(--text-muted);
}

.vir-source {
  color: var(--text-muted);
  font-style: italic;
}

.vir-arrow {
  color: var(--text-muted);
}

.vir-number {
  color: var(--text-muted);
}

.vir-diagnostic {
  font-weight: 500;
}

.vir-line-diagnostic .vir-diagnostic {
  color: var(--color-error);
}

.vir-keyword {
  color: var(--text-muted);
}

.vir-colon {
  color: var(--text-muted);
}

.vir-bracket {
  color: var(--text-muted);
}

.vir-plain {
  color: var(--text-secondary);
}

/* Line type background hints - removed for cleaner look */
.vir-line-section {
}

.vir-line-macro {
}

.vir-line-binding {
}

.vir-line-diagnostic {
  background: var(--color-error-bg);
}

/* Diagnostics Tab */
.diagnostic-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.diagnostic-item {
  padding: 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
}

.diagnostic-header {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
}

.severity-icon {
  width: 0.875rem;
  height: 0.875rem;
  flex-shrink: 0;
}

.severity-error .severity-icon {
  color: var(--color-error);
}
.severity-warning .severity-icon {
  color: var(--color-warning);
}

.diagnostic-message {
  flex: 1;
  font-size: 0.875rem;
  color: var(--text-primary);
}

.diagnostic-location {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.375rem;
  padding-left: 1.25rem;
}

.location-range {
  font-size: 0.75rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
}

.diagnostic-code {
  font-size: 0.625rem;
  color: var(--text-secondary);
}

/* Mobile responsive */
@media (max-width: 768px) {
  .croquis-playground {
    grid-template-columns: 1fr;
    grid-template-rows: minmax(300px, 1fr) minmax(300px, 1fr);
    height: auto;
    min-height: 100%;
  }

  .panel {
    min-height: 300px;
  }

  .input-panel {
    border-right: none;
    border-bottom: 1px solid var(--border-primary);
  }

  .panel-header {
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .tabs {
    flex-wrap: wrap;
    width: 100%;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
