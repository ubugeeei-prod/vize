<script setup lang="ts">
import {
  ref,
  watch,
  computed,
  onMounted,
  onUnmounted,
  inject,
  type ComputedRef,
  useTemplateRef,
} from "vue";
import MonacoEditor from "../../shared/monaco-editor.vue";
import type { WasmModule, LintResult, LintRule } from "../../wasm/index";
import { getWasm } from "../../wasm/index";
import { LINT_PRESET } from "../../shared/presets/patina";
import { mdiAlert, mdiCheckCircle, mdiCheck } from "@mdi/js";
import { formatHelp } from "../../utils/highlightCode";
import { useLocale } from "./useLocale";
import { useRuleManagement } from "./useRuleManagement";
import { useLinting } from "./useLinting";

const { compiler: _compiler } = defineProps<{
  compiler: WasmModule | null;
}>();

const _injectedTheme = inject<ComputedRef<"dark" | "light">>("theme");
const theme = computed<"dark" | "light">(() => _injectedTheme?.value ?? "light");

const source = ref(LINT_PRESET);
const lintResult = ref<LintResult | null>(null);
const rules = ref<LintRule[]>([]);
const error = ref<string | null>(null);
const activeTab = ref<"diagnostics" | "rules">("diagnostics");
const lintTime = ref<number | null>(null);
const editorRef = useTemplateRef<InstanceType<typeof MonacoEditor>>("editorRef");

// Wire in composables (lazy callbacks to resolve circular dependency)
let lintFn: () => void = () => {};

const { locales, currentLocale, loadLocaleConfig, setLocale } = useLocale(() => lintFn());

const {
  selectedPreset,
  enabledRules,
  severityOverrides,
  selectedCategory,
  searchQuery,
  categories,
  filteredRules,
  loadRuleConfig,
  initializeRuleState,
  applyPreset,
  toggleRule,
  toggleCategory,
  enableAllRules,
  disableAllRules,
  isCategoryFullyEnabled,
  isCategoryPartiallyEnabled,
} = useRuleManagement(rules, () => lintFn());

const { lint, loadRules, registerHoverProvider, disposeHoverProvider, getSeverityIcon } =
  useLinting({
    source,
    selectedPreset,
    enabledRules,
    severityOverrides,
    currentLocale,
    editorRef,
    lintResult,
    rules,
    error,
    lintTime,
    initializeRuleState,
  });

lintFn = lint;

const errorCount = computed(() => lintResult.value?.errorCount ?? 0);
const warningCount = computed(() => lintResult.value?.warningCount ?? 0);
const enabledRuleCount = computed(() => enabledRules.value.size);

const diagnostics = computed(() => {
  if (!lintResult.value?.diagnostics) return [];
  return lintResult.value.diagnostics.map((d) => ({
    message: d.message,
    help: d.help,
    startLine: d.location.start.line,
    startColumn: d.location.start.column,
    endLine: d.location.end?.line ?? d.location.start.line,
    endColumn: d.location.end?.column ?? d.location.start.column + 1,
    severity: d.severity,
  }));
});

let lintTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  source,
  () => {
    if (lintTimer) clearTimeout(lintTimer);
    lintTimer = setTimeout(lint, 300);
  },
  { immediate: true },
);

// Workaround for vite-plugin-vize prop reactivity issue
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
    loadRules();
    lint();
  }
}

onMounted(() => {
  loadLocaleConfig();
  loadRuleConfig();
  registerHoverProvider();
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
  disposeHoverProvider();
});
</script>

<template>
  <div class="patina-playground">
    <div class="panel input-panel">
      <div class="panel-header">
        <div class="header-title">
          <svg class="icon" viewBox="0 0 24 24"><path :d="mdiAlert" fill="currentColor" /></svg>
          <h2>Source</h2>
        </div>
        <div class="panel-actions">
          <button type="button" class="btn-ghost" @click="() => (source = LINT_PRESET)">
            Reset
          </button>
        </div>
      </div>
      <div class="editor-container">
        <MonacoEditor ref="editorRef" v-model="source" language="vue" :diagnostics :theme />
      </div>
    </div>

    <div class="panel output-panel">
      <div class="panel-header">
        <div class="header-title">
          <svg class="icon" viewBox="0 0 24 24">
            <path :d="mdiCheckCircle" fill="currentColor" />
          </svg>
          <h2>Lint Analysis</h2>
          <span v-if="lintTime !== null" class="perf-badge"> {{ lintTime.toFixed(2) }}ms </span>
          <template v-if="lintResult">
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
            <span v-if="lintResult?.diagnostics.length" class="tab-badge">{{
              lintResult.diagnostics.length
            }}</span>
          </button>
          <button
            type="button"
            :class="['tab', { active: activeTab === 'rules' }]"
            @click="() => (activeTab = 'rules')"
          >
            Rules
            <span class="tab-count">{{ enabledRuleCount }}/{{ rules.length }}</span>
          </button>
        </div>
      </div>

      <div class="output-content">
        <div v-if="error" class="error-panel">
          <div class="error-header">Lint Error</div>
          <pre class="error-content">{{ error }}</pre>
        </div>

        <template v-else-if="lintResult">
          <!-- Diagnostics Tab -->
          <div v-if="activeTab === 'diagnostics'" class="diagnostics-output">
            <div class="output-header-bar">
              <span class="output-title">Issues</span>
              <div class="locale-selector">
                <select
                  v-model="currentLocale"
                  aria-label="Locale"
                  @change="() => setLocale(currentLocale)"
                >
                  <option v-for="locale in locales" :key="locale.code" :value="locale.code">
                    {{ locale.name }}
                  </option>
                </select>
              </div>
            </div>

            <div v-if="lintResult.diagnostics.length === 0" class="success-state">
              <svg class="success-icon" viewBox="0 0 24 24">
                <path :d="mdiCheck" fill="currentColor" />
              </svg>
              <span>No issues found</span>
            </div>

            <div v-else class="diagnostics-list">
              <div
                v-for="diagnostic in lintResult.diagnostics"
                :key="`${diagnostic.rule}:${diagnostic.location.start.offset}:${diagnostic.location.end.offset}:${diagnostic.message}`"
                :class="['diagnostic-item', `severity-${diagnostic.severity}`]"
              >
                <div class="diagnostic-header">
                  <svg class="severity-icon" viewBox="0 0 24 24">
                    <path :d="getSeverityIcon(diagnostic.severity)" fill="currentColor" />
                  </svg>
                  <code class="rule-id">{{ diagnostic.rule }}</code>
                  <span class="location-badge">
                    {{ diagnostic.location.start.line }}:{{ diagnostic.location.start.column }}
                  </span>
                </div>
                <div class="diagnostic-message">{{ diagnostic.message }}</div>
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

          <!-- Rules Tab -->
          <div v-else-if="activeTab === 'rules'" class="rules-output">
            <div class="output-header-bar">
              <span class="output-title">Rule Configuration</span>
              <div class="rules-actions">
                <div class="preset-actions" role="group" aria-label="Lint preset">
                  <button
                    type="button"
                    :class="[
                      'btn-action',
                      'btn-preset',
                      { active: selectedPreset === 'general-recommended' },
                    ]"
                    @click="() => applyPreset('general-recommended')"
                  >
                    General Recommended
                  </button>
                  <button
                    type="button"
                    :class="[
                      'btn-action',
                      'btn-preset',
                      { active: selectedPreset === 'opinionated' },
                    ]"
                    @click="() => applyPreset('opinionated')"
                  >
                    Opinionated
                  </button>
                </div>
                <button type="button" class="btn-action" @click="enableAllRules">Enable All</button>
                <button type="button" class="btn-action" @click="disableAllRules">
                  Disable All
                </button>
              </div>
            </div>

            <div class="rules-toolbar">
              <input
                v-model="searchQuery"
                type="text"
                placeholder="Search rules..."
                aria-label="Search rules"
                class="search-input"
              />
              <select
                v-model="selectedCategory"
                aria-label="Category filter"
                class="category-select"
              >
                <option v-for="cat in categories" :key="cat" :value="cat">
                  {{ cat === "all" ? "All Categories" : cat }}
                </option>
              </select>
            </div>

            <!-- Category toggle headers when filtering by category -->
            <div v-if="selectedCategory !== 'all'" class="category-toggle">
              <label class="toggle-label">
                <input
                  type="checkbox"
                  :checked="isCategoryFullyEnabled(selectedCategory)"
                  :indeterminate="isCategoryPartiallyEnabled(selectedCategory)"
                  class="rule-checkbox"
                  @change="($event) => toggleCategory(selectedCategory, $event.target.checked)"
                />
                <span class="category-label">{{ selectedCategory }}</span>
                <span class="category-count">{{ filteredRules.length }} rules</span>
              </label>
            </div>

            <div class="rules-list">
              <div
                v-for="rule in filteredRules"
                :key="rule.name"
                :class="['rule-item', { disabled: !enabledRules.has(rule.name) }]"
              >
                <div class="rule-main">
                  <label class="rule-toggle">
                    <input
                      type="checkbox"
                      :checked="enabledRules.has(rule.name)"
                      class="rule-checkbox"
                      @change="() => toggleRule(rule.name)"
                    />
                    <code class="rule-id">{{ rule.name }}</code>
                  </label>
                  <div class="rule-badges">
                    <span class="badge category-badge">{{ rule.category }}</span>
                    <span :class="['badge', 'severity-badge', rule.defaultSeverity]">
                      {{ rule.defaultSeverity }}
                    </span>
                    <span v-if="rule.fixable" class="badge fixable-badge">fix</span>
                  </div>
                </div>
                <div class="rule-description">{{ rule.description }}</div>
              </div>

              <div v-if="filteredRules.length === 0" class="empty-state">
                No rules match your search
              </div>
            </div>
          </div>
        </template>

        <div v-else class="loading-state">
          <span>Enter Vue code to see lint results</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
.patina-playground {
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

.count-badge {
  font-size: 0.625rem;
  padding: 0.0625rem 0.375rem;
  border-radius: 8px;
  min-width: 1.25rem;
  text-align: center;
  font-family: "JetBrains Mono", monospace;
}

.count-badge.errors {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  font-weight: 600;
}

.count-badge.warnings {
  background: var(--bg-tertiary);
  color: var(--text-muted);
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
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border-radius: 8px;
  min-width: 1rem;
  text-align: center;
  font-weight: 600;
}

.tab-count {
  font-size: 0.625rem;
  color: var(--text-muted);
  font-family: "JetBrains Mono", monospace;
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
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  overflow: hidden;
}

.error-header {
  padding: 0.5rem 0.75rem;
  background: var(--bg-tertiary);
  color: var(--text-primary);
  font-size: 0.75rem;
  font-weight: 600;
  border-bottom: 1px solid var(--border-primary);
}

.error-content {
  padding: 0.75rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: "JetBrains Mono", monospace;
}

/* Output Header Bar */
.output-header-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0;
  border-bottom: 1px solid var(--border-primary);
  margin-bottom: 0.75rem;
}

.output-title {
  font-size: 0.6875rem;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.locale-selector select {
  padding: 0.25rem 0.5rem;
  font-size: 0.625rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 3px;
  color: var(--text-primary);
  cursor: pointer;
}

/* Success State */
.success-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
  color: var(--text-muted);
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
  border-left: 2px solid var(--text-muted);
  border-radius: 0;
}

.diagnostic-item.severity-error {
  border-left-color: var(--color-error);
}

.diagnostic-item.severity-warning {
  border-left-color: var(--color-warning);
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
  color: var(--color-error);
}

.severity-warning .severity-icon {
  color: var(--color-warning);
}

.rule-id {
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

.diagnostic-help {
  margin-top: 0.75rem;
  padding: 0.75rem;
  background: var(--bg-tertiary);
  border-left: 2px solid var(--text-muted);
  border-radius: 0;
  font-size: 0.85rem;
}

.help-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid var(--border-primary);
}

.help-icon {
  font-size: 1rem;
}

.help-label {
  font-weight: 600;
  color: var(--text-primary);
  font-size: 0.8125rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.help-content {
  color: var(--text-primary);
  line-height: 1.6;
}

.help-content strong {
  color: var(--text-primary);
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

/* Syntax highlighting colors — warm earthy tones, theme-adaptive */
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

/* Rules Output */
.rules-output {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.rules-toolbar {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.rules-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.preset-actions {
  display: flex;
  gap: 0.375rem;
  margin-right: 0.5rem;
}

.btn-action {
  padding: 0.25rem 0.5rem;
  font-size: 0.625rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 3px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-action:hover {
  background: var(--bg-secondary);
  color: var(--text-primary);
  border-color: var(--accent-rust);
}

.btn-preset.active {
  background: color-mix(in srgb, var(--accent-rust) 12%, var(--bg-tertiary));
  color: var(--text-primary);
  border-color: var(--accent-rust);
}

.search-input {
  flex: 1;
  padding: 0.375rem 0.625rem;
  font-size: 0.75rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-primary);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-input:focus {
  outline: none;
  border-color: var(--accent-rust);
}

.category-select {
  padding: 0.375rem 0.625rem;
  font-size: 0.75rem;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  color: var(--text-primary);
  cursor: pointer;
}

.category-select:focus {
  outline: none;
  border-color: var(--accent-rust);
}

.category-toggle {
  padding: 0.625rem 0.75rem;
  margin-bottom: 0.75rem;
  background: var(--bg-tertiary);
  border-radius: 4px;
  border: 1px solid var(--border-primary);
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.category-label {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--text-primary);
}

.category-count {
  font-size: 0.6875rem;
  color: var(--text-muted);
}

/* Rules List */
.rules-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.rule-item {
  padding: 0.625rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  transition: all 0.15s;
}

.rule-item:hover {
  border-color: var(--border-secondary);
}

.rule-item.disabled {
  opacity: 0.5;
}

.rule-main {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  margin-bottom: 0.25rem;
}

.rule-toggle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.rule-checkbox {
  width: 14px;
  height: 14px;
  accent-color: var(--accent-rust);
  cursor: pointer;
}

.rules-output .rule-id {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-primary);
  background: transparent;
  padding: 0;
}

.rule-item.disabled .rule-id {
  text-decoration: line-through;
}

.rule-badges {
  display: flex;
  gap: 0.375rem;
  flex-wrap: wrap;
}

.badge {
  display: inline-block;
  padding: 0.0625rem 0.375rem;
  font-size: 0.5625rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  border-radius: 2px;
}

.category-badge {
  background: var(--bg-tertiary);
  color: var(--text-muted);
}

.severity-badge {
  background: var(--bg-tertiary);
}

.severity-badge.error {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  font-weight: 600;
}

.severity-badge.warning {
  background: var(--bg-tertiary);
  color: var(--text-muted);
}

.fixable-badge {
  background: var(--bg-tertiary);
  color: var(--text-muted);
}

.rule-description {
  font-size: 0.6875rem;
  color: var(--text-muted);
  line-height: 1.4;
  padding-left: 1.375rem;
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
  .patina-playground {
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

  .rules-toolbar {
    flex-direction: column;
  }

  .preset-actions {
    width: 100%;
    margin-right: 0;
  }

  .rule-main {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.375rem;
  }

  .rule-description {
    padding-left: 0;
  }
}
</style>
