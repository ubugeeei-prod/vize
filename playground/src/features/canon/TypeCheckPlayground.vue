<script setup lang="ts">
import "./TypeCheckPlayground.css";
import { ref, watch, computed, onMounted, onUnmounted, inject, type ComputedRef } from "vue";
import MonacoEditor from "../../shared/MonacoEditor.vue";
import CodeHighlight from "../../shared/CodeHighlight.vue";
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

const props = defineProps<{
  compiler: WasmModule | null;
}>();
const _injectedTheme = inject<ComputedRef<"dark" | "light">>("theme");
const theme = computed<"dark" | "light">(() => _injectedTheme?.value ?? "light");

const source = ref(TYPECHECK_PRESET);
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
  compiler: () => props.compiler ?? getWasm(),
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

watch(
  () => props.compiler,
  () => {
    if (props.compiler) {
      typeCheck();
      loadCapabilities();
    }
  },
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
    typeCheck();
    loadCapabilities();
  }
}

onMounted(async () => {
  loadOptions();
  await configureTypeScript();
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
          <button class="btn-ghost" @click="setPreset('untyped')">Untyped</button>
          <button class="btn-ghost" @click="setPreset('typed')">Typed</button>
        </div>
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
            :class="['tab', { active: activeTab === 'diagnostics' }]"
            @click="activeTab = 'diagnostics'"
          >
            Diagnostics
            <span v-if="diagnostics.length" class="tab-badge">{{ diagnostics.length }}</span>
          </button>
          <button
            :class="['tab', { active: activeTab === 'virtualTs' }]"
            @click="activeTab = 'virtualTs'"
          >
            Virtual TS
          </button>
          <button
            :class="['tab', { active: activeTab === 'capabilities' }]"
            @click="activeTab = 'capabilities'"
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
                v-for="(diagnostic, i) in diagnostics"
                :key="i"
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
                  <li v-for="(note, i) in capabilities.notes" :key="i">
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
