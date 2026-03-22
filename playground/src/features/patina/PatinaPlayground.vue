<script setup lang="ts">
import "./PatinaPlayground.css";
import { ref, watch, computed, onMounted, onUnmounted, inject, type ComputedRef } from "vue";
import MonacoEditor from "../../shared/MonacoEditor.vue";
import type { WasmModule, LintResult, LintRule } from "../../wasm/index";
import { getWasm } from "../../wasm/index";
import { LINT_PRESET } from "../../shared/presets/patina";
import { mdiAlert, mdiCheckCircle, mdiCheck } from "@mdi/js";
import { formatHelp } from "../../utils/highlightCode";
import { useLocale } from "./useLocale";
import { useRuleManagement } from "./useRuleManagement";
import { useLinting } from "./useLinting";

const props = defineProps<{
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
const editorRef = ref<InstanceType<typeof MonacoEditor> | null>(null);

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
          <button class="btn-ghost" @click="source = LINT_PRESET">Reset</button>
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
            :class="['tab', { active: activeTab === 'diagnostics' }]"
            @click="activeTab = 'diagnostics'"
          >
            Diagnostics
            <span v-if="lintResult?.diagnostics.length" class="tab-badge">{{
              lintResult.diagnostics.length
            }}</span>
          </button>
          <button :class="['tab', { active: activeTab === 'rules' }]" @click="activeTab = 'rules'">
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
                  @change="setLocale(currentLocale)"
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
                v-for="(diagnostic, i) in lintResult.diagnostics"
                :key="i"
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
                    :class="[
                      'btn-action',
                      'btn-preset',
                      { active: selectedPreset === 'happy-path' },
                    ]"
                    @click="applyPreset('happy-path')"
                  >
                    Happy Path
                  </button>
                  <button
                    :class="[
                      'btn-action',
                      'btn-preset',
                      { active: selectedPreset === 'opinionated' },
                    ]"
                    @click="applyPreset('opinionated')"
                  >
                    Opinionated
                  </button>
                </div>
                <button class="btn-action" @click="enableAllRules">Enable All</button>
                <button class="btn-action" @click="disableAllRules">Disable All</button>
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
                  @change="toggleCategory(selectedCategory, $event.target.checked)"
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
                      @change="toggleRule(rule.name)"
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
