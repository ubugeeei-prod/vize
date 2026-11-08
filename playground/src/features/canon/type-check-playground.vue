<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted, inject, type ComputedRef } from "vue";
import MonacoEditor from "../../shared/monaco-editor.vue";
import CodeHighlight from "../../shared/code-highlight.vue";
import ExperimentalFeatures from "../../shared/experimental-features.vue";
import { useExperimentalFeatures } from "../../shared/experimentalFeatures";
import { useCompilerInit } from "../../utils/useCompilerInit";
import { type WasmModule, getWasm } from "../../wasm/index";
import { TYPECHECK_PRESET, TYPECHECK_TYPED_PRESET } from "../../shared/presets/typecheck";
import {
  mdiCheckDecagram,
  mdiCheck,
  mdiCloseCircle,
  mdiAlert,
  mdiInformation,
  mdiCodeTags,
} from "@mdi/js";
import { useMonacoTypeCheck } from "./useMonacoTypeCheck";
import { formatHelp, formatMessage } from "./formatHelpers";

const { compiler: compilerProp } = defineProps<{
  compiler: WasmModule | null;
}>();
const _injectedTheme = inject<ComputedRef<"dark" | "light">>("theme");
const theme = computed<"dark" | "light">(() => _injectedTheme?.value ?? "light");

const source = ref(TYPECHECK_PRESET);
const { options: experimentals, loadExample } = useExperimentalFeatures("typechecker", source);
const activeTab = ref<"diagnostics" | "virtualTs" | "capabilities">("diagnostics");

// Options
const strictMode = ref(false);
const includeVirtualTs = ref(true);
const checkProps = ref(true);
const checkEmits = ref(true);
const checkTemplateBindings = ref(true);
const useMonacoTs = ref(true);

const STORAGE_KEY = "vize-canon-typecheck-options";

const {
  typeCheckResult,
  capabilities,
  error,
  checkTime,
  diagnostics,
  errorCount,
  warningCount,
  configureTypeScript,
  registerHoverProvider,
  typeCheck,
  loadCapabilities,
  dispose,
} = useMonacoTypeCheck({
  source,
  experimentals,
  compiler: () => compilerProp ?? getWasm(),
  strictMode,
  includeVirtualTs,
  checkProps,
  checkEmits,
  checkTemplateBindings,
  useMonacoTs,
});

function loadOptions() {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      const config = JSON.parse(saved);
      strictMode.value = config.strictMode ?? false;
      includeVirtualTs.value = config.includeVirtualTs ?? true;
      checkProps.value = config.checkProps ?? true;
      checkEmits.value = config.checkEmits ?? true;
      checkTemplateBindings.value = config.checkTemplateBindings ?? true;
      useMonacoTs.value = config.useMonacoTs ?? true;
    }
  } catch (e) {
    console.warn("Failed to load options:", e);
  }
}

function saveOptions() {
  try {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        strictMode: strictMode.value,
        includeVirtualTs: includeVirtualTs.value,
        checkProps: checkProps.value,
        checkEmits: checkEmits.value,
        checkTemplateBindings: checkTemplateBindings.value,
        useMonacoTs: useMonacoTs.value,
      }),
    );
  } catch (e) {
    console.warn("Failed to save options:", e);
  }
}

function getSeverityIcon(severity: "error" | "warning" | "info" | "hint"): string {
  switch (severity) {
    case "error":
      return mdiCloseCircle;
    case "warning":
      return mdiAlert;
    default:
      return mdiInformation;
  }
}

function setPreset(preset: "untyped" | "typed") {
  source.value = preset === "typed" ? TYPECHECK_TYPED_PRESET : TYPECHECK_PRESET;
}

let checkTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  source,
  () => {
    if (checkTimer) clearTimeout(checkTimer);
    checkTimer = setTimeout(typeCheck, 300);
  },
  { immediate: true },
);

watch(
  [strictMode, includeVirtualTs, checkProps, checkEmits, checkTemplateBindings, useMonacoTs],
  () => {
    saveOptions();
    typeCheck();
  },
);
watch(experimentals, typeCheck, { deep: true });

watch(
  () => compilerProp,
  () => {
    if (compilerProp) {
      typeCheck();
      loadCapabilities();
    }
  },
);

onMounted(async () => {
  loadOptions();
  await configureTypeScript();
  registerHoverProvider();
});
useCompilerInit(() => {
  typeCheck();
  loadCapabilities();
});

onUnmounted(() => {
  if (checkTimer) clearTimeout(checkTimer);
  dispose();
});
</script>

<template>
  <div class="typecheck-playground">
    <div class="panel input-panel">
      <div class="panel-header">
        <div class="header-title">
          <svg class="icon" viewBox="0 0 24 24">
            <path :d="mdiCodeTags" fill="currentColor" />
          </svg>
          <h2>Source</h2>
        </div>
        <div class="panel-actions">
          <button type="button" class="btn-ghost" @click="() => setPreset('untyped')">
            Untyped
          </button>
          <button type="button" class="btn-ghost" @click="() => setPreset('typed')">Typed</button>
        </div>
      </div>
      <div class="experimental-controls">
        <ExperimentalFeatures v-model="experimentals" scope="typechecker" @example="loadExample" />
      </div>
      <div class="editor-container">
        <MonacoEditor v-model="source" language="vue" :diagnostics :theme />
      </div>
    </div>

    <div class="panel output-panel">
      <div class="panel-header">
        <div class="header-title">
          <svg class="icon" viewBox="0 0 24 24">
            <path :d="mdiCheckDecagram" fill="currentColor" />
          </svg>
          <h2>Type Analysis</h2>
          <span v-if="checkTime !== null" class="perf-badge"> {{ checkTime.toFixed(2) }}ms </span>
          <template v-if="typeCheckResult">
            <span v-if="errorCount > 0" class="count-badge errors">{{ errorCount }}</span>
            <span v-if="warningCount > 0" class="count-badge warnings">{{ warningCount }}</span>
          </template>
        </div>
        <div class="tabs">
          <button
            type="button"
            :class="['tab', { active: activeTab === 'diagnostics' }]"
            @click="() => (activeTab = 'diagnostics')"
          >
            Diagnostics
            <span v-if="diagnostics.length" class="tab-badge">{{ diagnostics.length }}</span>
          </button>
          <button
            type="button"
            :class="['tab', { active: activeTab === 'virtualTs' }]"
            @click="() => (activeTab = 'virtualTs')"
          >
            Virtual TS
          </button>
          <button
            type="button"
            :class="['tab', { active: activeTab === 'capabilities' }]"
            @click="() => (activeTab = 'capabilities')"
          >
            Info
          </button>
        </div>
      </div>

      <div class="output-content">
        <div v-if="error" class="error-panel">
          <div class="error-header">Type Check Error</div>
          <pre class="error-content">{{ error }}</pre>
        </div>

        <template v-else-if="typeCheckResult">
          <!-- Diagnostics Tab -->
          <div v-if="activeTab === 'diagnostics'" class="diagnostics-output">
            <div class="output-header-bar">
              <span class="output-title">Type Issues</span>
              <div class="options-toggle">
                <label class="option-label">
                  <input v-model="strictMode" type="checkbox" />
                  Strict
                </label>
              </div>
            </div>

            <div class="options-panel">
              <label class="option-label highlight">
                <input v-model="useMonacoTs" type="checkbox" />
                TypeScript (Monaco)
              </label>
              <label class="option-label">
                <input v-model="checkProps" type="checkbox" />
                Check Props
              </label>
              <label class="option-label">
                <input v-model="checkEmits" type="checkbox" />
                Check Emits
              </label>
              <label class="option-label">
                <input v-model="checkTemplateBindings" type="checkbox" />
                Check Template Bindings
              </label>
              <label class="option-label">
                <input v-model="includeVirtualTs" type="checkbox" />
                Show Virtual TS
              </label>
            </div>

            <div v-if="diagnostics.length === 0" class="success-state">
              <svg class="success-icon" viewBox="0 0 24 24">
                <path :d="mdiCheck" fill="currentColor" />
              </svg>
              <span>No type issues found</span>
            </div>

            <div v-else class="diagnostics-list">
              <div
                v-for="diagnostic in diagnostics"
                :key="`${diagnostic.startLine}:${diagnostic.startColumn}:${diagnostic.code}:${diagnostic.message}`"
                :class="['diagnostic-item', `severity-${diagnostic.severity}`]"
              >
                <div class="diagnostic-header">
                  <svg class="severity-icon" viewBox="0 0 24 24">
                    <path :d="getSeverityIcon(diagnostic.severity)" fill="currentColor" />
                  </svg>
                  <code v-if="diagnostic.code" class="error-code">TS{{ diagnostic.code }}</code>
                  <span class="location-badge">
                    {{ diagnostic.startLine }}:{{ diagnostic.startColumn }}
                  </span>
                </div>
                <!-- @vize:forget formatMessage output is pre-escaped -->
                <div class="diagnostic-message" v-html="formatMessage(diagnostic.message)"></div>
                <div v-if="diagnostic.help" class="diagnostic-help">
                  <div class="help-header">
                    <span class="help-icon">?</span>
                    <span class="help-label">Hint</span>
                  </div>
                  <!-- @vize:forget formatHelp output is pre-escaped -->
                  <div class="help-content" v-html="formatHelp(diagnostic.help)"></div>
                </div>
              </div>
            </div>
          </div>

          <!-- Virtual TS Tab -->
          <div v-else-if="activeTab === 'virtualTs'" class="virtualts-output">
            <div class="output-header-bar">
              <span class="output-title">Generated TypeScript</span>
            </div>
            <div class="virtual-ts-notice">
              Virtual TS is generated internally for type checking. It is not portable and the
              format may change without notice.
            </div>
            <div v-if="typeCheckResult.virtualTs" class="editor-container">
              <CodeHighlight
                :code="typeCheckResult.virtualTs"
                language="typescript"
                show-line-numbers
                :theme
              />
            </div>
            <div v-else class="empty-state">
              <span>Enable "Generate Virtual TS" option to see generated TypeScript</span>
            </div>
          </div>

          <!-- Capabilities Tab -->
          <div v-else-if="activeTab === 'capabilities'" class="capabilities-output">
            <div class="output-header-bar">
              <span class="output-title">Type Checker Capabilities</span>
            </div>

            <div v-if="capabilities" class="capabilities-content">
              <div class="capability-section">
                <h3>Mode</h3>
                <code class="mode-badge">{{ capabilities.mode }}</code>
                <p>{{ capabilities.description }}</p>
              </div>

              <div class="capability-section">
                <h3>Available Checks</h3>
                <div class="checks-list">
                  <div v-for="check in capabilities.checks" :key="check.name" class="check-item">
                    <code class="check-name">{{ check.name }}</code>
                    <span :class="['check-severity', check.severity]">{{ check.severity }}</span>
                    <p class="check-description">{{ check.description }}</p>
                  </div>
                </div>
              </div>

              <div class="capability-section">
                <h3>Notes</h3>
                <ul class="notes-list">
                  <li v-for="note in capabilities.notes" :key="note">
                    {{ note }}
                  </li>
                </ul>
              </div>
            </div>
          </div>
        </template>

        <div v-else class="loading-state">
          <span>Enter Vue code to see type analysis</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
.typecheck-playground {
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
  color: var(--accent-blue);
}

.header-title h2 {
  font-size: 0.875rem;
  font-weight: 600;
  margin: 0;
}

.perf-badge {
  font-size: 0.625rem;
  padding: 0.125rem 0.375rem;
  background: var(--color-success-bg);
  color: var(--color-success);
  border-radius: 3px;
  font-family: "JetBrains Mono", monospace;
}

.count-badge {
  font-size: 0.625rem;
  padding: 0.0625rem 0.375rem;
  border-radius: 8px;
  min-width: 1.25rem;
  text-align: center;
  font-family: "JetBrains Mono", monospace;
}

.count-badge.errors {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.count-badge.warnings {
  background: var(--color-warning-bg);
  color: var(--color-warning);
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
  overflow-y: auto;
  padding: 1rem;
}

/* Error State */
.error-panel {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
  overflow: hidden;
}

.error-header {
  padding: 0.5rem 0.75rem;
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  font-size: 0.75rem;
  font-weight: 600;
}

.error-content {
  padding: 0.75rem;
  font-size: 0.75rem;
  color: #fca5a5;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
}

/* Output Header Bar */
.output-header-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  margin-bottom: 0.75rem;
}

.output-title {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--accent-rust);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* Options */
.options-panel {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  margin-bottom: 0.75rem;
}

.option-label {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.6875rem;
  color: var(--text-secondary);
  cursor: pointer;
}

.option-label input[type="checkbox"] {
  width: 12px;
  height: 12px;
  accent-color: var(--accent-blue);
}

.option-label.highlight {
  background: rgba(59, 130, 246, 0.15);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  border: 1px solid rgba(59, 130, 246, 0.3);
}

/* Success State */
.success-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
  color: #4ade80;
  font-size: 0.875rem;
}

.success-icon {
  width: 1.25rem;
  height: 1.25rem;
}

/* Diagnostics List */
.diagnostics-list {
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
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.375rem;
}

.severity-icon {
  width: 0.75rem;
  height: 0.75rem;
  flex-shrink: 0;
}

.severity-error .severity-icon {
  color: #ef4444;
}

.severity-warning .severity-icon {
  color: #f59e0b;
}

.severity-info .severity-icon {
  color: #60a5fa;
}

.error-code {
  font-size: 0.6875rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
}

.location-badge {
  margin-left: auto;
  font-size: 0.625rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
}

.diagnostic-message {
  font-size: 0.8125rem;
  color: var(--text-primary);
  line-height: 1.4;
}

.diagnostic-message .msg-type {
  font-family: "JetBrains Mono", monospace;
  font-size: 0.85em;
  color: #79c0ff;
  background: rgba(121, 192, 255, 0.1);
  padding: 0.1rem 0.3rem;
  border-radius: 3px;
}

.diagnostic-help {
  margin-top: 0.75rem;
  padding: 0.75rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  font-size: 0.85rem;
}

.help-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid rgba(96, 165, 250, 0.15);
}

.help-icon {
  font-size: 1rem;
}

.help-label {
  font-weight: 600;
  color: #60a5fa;
  font-size: 0.9rem;
}

.help-content {
  color: var(--text-primary);
  line-height: 1.6;
}

.help-content strong {
  color: #f59e0b;
  font-weight: 600;
}

.help-content .help-code {
  margin: 0.5rem 0;
  padding: 1rem 1.125rem;
  background: var(--code-bg);
  border: 1px solid var(--code-border);
  border-radius: 6px;
  overflow-x: auto;
  font-family: "JetBrains Mono", "Fira Code", monospace;
  font-size: 0.8125rem;
  line-height: 1.55;
}

.help-content .help-code code {
  color: var(--code-foreground);
  background: none;
  padding: 0;
}

.help-content .help-inline-code {
  background: var(--code-bg);
  border: 1px solid var(--code-border);
  padding: 0.18rem 0.42rem;
  border-radius: 3px;
  font-family: "JetBrains Mono", "Fira Code", monospace;
  font-size: 0.85em;
  color: var(--hl-keyword);
}

/* Syntax highlighting colors */
.help-content .hl-keyword {
  color: var(--hl-keyword);
}

.help-content .hl-vue-api {
  color: var(--hl-function);
}

.help-content .hl-string {
  color: var(--hl-string);
}

.help-content .hl-comment {
  color: var(--hl-comment);
  font-style: italic;
}

.help-content .hl-tag {
  color: var(--hl-tag);
}

.help-content .hl-directive {
  color: var(--hl-directive);
}

.help-content .hl-delimiter {
  color: var(--hl-delimiter);
}

.help-content .hl-type {
  color: var(--hl-type);
}

.help-content .hl-number {
  color: var(--hl-number);
}

.help-content .hl-property {
  color: var(--hl-property);
}

.help-content .hl-value {
  color: var(--hl-value);
}

.severity-error .severity-icon {
  color: #ef4444;
}

.severity-warning .severity-icon {
  color: #f59e0b;
}

.severity-info .severity-icon {
  color: #60a5fa;
}

.diagnostic-code {
  font-size: 0.6875rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
}

.location-badge {
  margin-left: auto;
  font-size: 0.625rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-muted);
}

/* Virtual TS Output */
.virtualts-output {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.virtual-ts-notice {
  padding: 0.5rem 0.75rem;
  background: var(--color-notice-bg);
  border: 1px solid var(--color-notice-border);
  border-radius: 4px;
  font-size: 0.6875rem;
  color: var(--color-notice);
  line-height: 1.4;
  margin-bottom: 0.5rem;
}

.virtualts-output .editor-container {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.virtualts-output .code-highlight {
  min-height: 100%;
}

/* Capabilities Output */
.capabilities-content {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.capability-section h3 {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-secondary);
  margin: 0 0 0.5rem 0;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.capability-section p {
  font-size: 0.8125rem;
  color: var(--text-muted);
  margin: 0.25rem 0 0 0;
}

.mode-badge {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  font-size: 0.75rem;
  color: #60a5fa;
}

.checks-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.check-item {
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
}

.check-name {
  font-size: 0.75rem;
  font-family: "JetBrains Mono", monospace;
  color: var(--text-primary);
}

.check-severity {
  margin-left: 0.5rem;
  font-size: 0.625rem;
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  text-transform: uppercase;
}

.check-severity.error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.check-severity.warning {
  background: var(--color-warning-bg);
  color: var(--color-warning);
}

.check-description {
  font-size: 0.6875rem;
  color: var(--text-muted);
  margin: 0.25rem 0 0 0;
}

.notes-list {
  margin: 0;
  padding-left: 1rem;
  color: var(--text-muted);
  font-size: 0.75rem;
}

.notes-list li {
  margin: 0.25rem 0;
}

.empty-state,
.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  color: var(--text-muted);
  font-size: 0.875rem;
}

/* Mobile responsive */
@media (max-width: 768px) {
  .typecheck-playground {
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

  .options-panel {
    flex-direction: column;
  }
}
</style>
