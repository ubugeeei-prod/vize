<script setup lang="ts">
import "./CroquisPlayground.css";
import CroquisSource from "./CroquisSource.vue";
import { type WasmModule, getWasm } from "../../wasm/index";
import { mdiChartTimelineVariant, mdiCheck, mdiCloseCircle, mdiAlert } from "@mdi/js";
import { useCroquisAnalysis } from "./useCroquisAnalysis";
import CroquisStatsPanel from "./CroquisStatsPanel.vue";
import ReactivityOverlayPanel from "./ReactivityOverlayPanel.vue";
import { getSourceLabel, getSourceClass } from "./bindingHelpers";
import { getScopeColorClass } from "./scopeColors";
import { withTokenOffsets } from "../../utils/withTokenOffsets";

const props = defineProps<{
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
} = useCroquisAnalysis(() => props.compiler ?? getWasm());
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
