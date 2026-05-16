<script setup lang="ts">
import "./CroquisPlayground.css";
import MonacoEditor from "../../shared/MonacoEditor.vue";
import { type WasmModule, getWasm } from "../../wasm/index";
import { ANALYSIS_PRESET } from "../../shared/presets/croquis";
import { mdiCodeTags, mdiChartTimelineVariant, mdiCheck, mdiCloseCircle, mdiAlert } from "@mdi/js";
import { useCroquisAnalysis } from "./useCroquisAnalysis";
import { getSourceLabel, getSourceClass } from "./bindingHelpers";
import { getScopeColorClass } from "./scopeColors";

const props = defineProps<{
  compiler: WasmModule | null;
}>();

const {
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
  diagnostics,
  stats,
  monacoDiagnostics,
  bindingsBySource,
  virLines,
} = useCroquisAnalysis(() => props.compiler ?? getWasm());
</script>

<template>
  <div class="croquis-playground">
    <div class="panel input-panel">
      <div class="panel-header">
        <div class="header-title">
          <svg class="icon" viewBox="0 0 24 24"><path :d="mdiCodeTags" fill="currentColor" /></svg>
          <h2>Source</h2>
        </div>
        <div class="panel-actions">
          <label class="toggle-label">
            <input v-model="showScopeVisualization" type="checkbox" />
            <span>Visualize Scopes</span>
          </label>
          <button class="btn-ghost" @click="source = ANALYSIS_PRESET">Reset</button>
        </div>
      </div>
      <div class="editor-container">
        <MonacoEditor
          ref="editorRef"
          v-model="source"
          language="vue"
          :scopes="editorScopes"
          :diagnostics="monacoDiagnostics"
          :theme="theme"
        />
      </div>
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
          <button :class="['tab', { active: activeTab === 'vir' }]" @click="activeTab = 'vir'">
            VIR
          </button>
          <button :class="['tab', { active: activeTab === 'stats' }]" @click="activeTab = 'stats'">
            Stats
          </button>
          <button
            :class="['tab', { active: activeTab === 'bindings' }]"
            @click="activeTab = 'bindings'"
          >
            Bindings
          </button>
          <button
            :class="['tab', { active: activeTab === 'scopes' }]"
            @click="activeTab = 'scopes'"
          >
            Scopes
          </button>
          <button
            :class="['tab', { active: activeTab === 'diagnostics' }]"
            @click="activeTab = 'diagnostics'"
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
            <div class="vir-notice">
              VIR is a human-readable display format for debugging purposes only. It is not portable
              and should not be parsed or used as a stable interface.
            </div>
            <div class="vir-content">
              <div class="vir-line-numbers">
                <span v-for="(_, i) in virLines" :key="i" class="vir-ln">{{ i + 1 }}</span>
              </div>
              <div class="vir-code">
                <div
                  v-for="line in virLines"
                  :key="line.index"
                  :class="['vir-line', `vir-line-${line.lineType}`]"
                >
                  <template v-if="line.tokens.length > 0"
                    ><span
                      v-for="(token, ti) in line.tokens"
                      :key="ti"
                      :class="['vir-token', `vir-${token.type}`]"
                      >{{ token.text }}</span
                    ></template
                  ><template v-else><span>&#160;</span></template>
                </div>
              </div>
            </div>
          </div>

          <!-- Stats Tab -->
          <div v-else-if="activeTab === 'stats'" class="stats-output">
            <div class="stats-grid">
              <div class="stat-box">
                <div class="stat-number">{{ stats?.binding_count || 0 }}</div>
                <div class="stat-label">Bindings</div>
              </div>
              <div class="stat-box">
                <div class="stat-number">{{ stats?.macro_count || 0 }}</div>
                <div class="stat-label">Macros</div>
              </div>
              <div class="stat-box">
                <div class="stat-number">{{ stats?.scope_count || 0 }}</div>
                <div class="stat-label">Scopes</div>
              </div>
              <div class="stat-box">
                <div class="stat-number">{{ css?.v_bind_count || 0 }}</div>
                <div class="stat-label">v-bind()</div>
              </div>
            </div>

            <div class="section">
              <h3 class="section-title">Compiler Macros</h3>
              <div v-if="macros.length === 0" class="empty-state">No macros detected</div>
              <div v-else class="macro-list">
                <div
                  v-for="macro in macros"
                  :key="`${macro.name}-${macro.start}`"
                  class="macro-item"
                >
                  <span class="macro-name">{{ macro.name }}</span>
                  <code v-if="macro.type_args" class="macro-type">{{ macro.type_args }}</code>
                  <span v-if="macro.binding" class="macro-binding">→ {{ macro.binding }}</span>
                </div>
              </div>
            </div>

            <div v-if="css" class="section">
              <h3 class="section-title">CSS Analysis</h3>
              <div class="css-info">
                <span class="css-stat">{{ css.selector_count }} selectors</span>
                <span v-if="css.is_scoped" class="css-badge scoped">scoped</span>
                <span v-if="css.v_bind_count > 0" class="css-badge vbind"
                  >{{ css.v_bind_count }} v-bind</span
                >
              </div>
            </div>

            <div v-if="typeExports.length > 0" class="section">
              <h3 class="section-title">Type Exports <span class="badge hoisted">hoisted</span></h3>
              <div class="export-list">
                <div
                  v-for="te in typeExports"
                  :key="`${te.name}-${te.start}`"
                  class="export-item valid"
                >
                  <span class="export-kind">{{ te.kind }}</span>
                  <code class="export-name">{{ te.name }}</code>
                  <span class="export-badge hoisted">hoisted to module</span>
                </div>
              </div>
            </div>

            <div v-if="invalidExports.length > 0" class="section">
              <h3 class="section-title">Invalid Exports <span class="badge error">error</span></h3>
              <div class="export-list">
                <div
                  v-for="ie in invalidExports"
                  :key="`${ie.name}-${ie.start}`"
                  class="export-item invalid"
                >
                  <span class="export-kind">{{ ie.kind }}</span>
                  <code class="export-name">{{ ie.name }}</code>
                  <span class="export-badge error">not allowed in script setup</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Bindings Tab -->
          <div v-else-if="activeTab === 'bindings'" class="bindings-output">
            <div v-if="bindings.length === 0" class="empty-state">No bindings detected</div>

            <template v-else>
              <div v-for="(group, source) in bindingsBySource" :key="source" class="source-group">
                <div class="source-header">
                  <span :class="['source-indicator', getSourceClass(String(source))]"></span>
                  <span class="source-name">{{ getSourceLabel(String(source)) }}</span>
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
                v-for="(diag, i) in diagnostics"
                :key="i"
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
