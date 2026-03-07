<script setup lang="ts">
import { computed, watch, onMounted, onUnmounted } from "vue";
import MonacoEditor from "../../shared/MonacoEditor.vue";
import CodeHighlight from "../../shared/CodeHighlight.vue";
import { useTheme } from "../../utils/useTheme";
import { useClipboard } from "../../utils/useClipboard";
import { useAtelierCompiler } from "./useAtelierCompiler";
import { CODE_OUTPUT_LABELS } from "./codeOutputs";
import {
  useTokenAnalysis,
  getTokenTypeIcon,
  getTokenTypeLabel,
  getTokenTypeColor,
} from "./useTokenAnalysis";
import { getBindingIcon, getBindingLabel } from "./bindingHelpers";
import { type loadWasm, getWasm } from "../../wasm/index";

const props = defineProps<{
  compiler: Awaited<ReturnType<typeof loadWasm>> | null;
}>();

const { theme } = useTheme();
const { copyToClipboard } = useClipboard();

const {
  inputMode,
  source,
  output,
  sfcResult,
  error,
  activeTab,
  isCompiling,
  selectedPreset,
  compileTime,
  cssResult,
  cssOptions,
  formattedCss,
  codeViewMode,
  codeOutputTarget,
  codeOutputVersion,
  activeCodeOutput,
  astHideLoc,
  astHideSource,
  astCollapsed,
  editorLanguage,
  astJson,
  bindingsSummary,
  groupedBindings,
  compile,
  handlePresetChange,
} = useAtelierCompiler(() => props.compiler ?? getWasm());

const { lexicalTokens, tokensByType, tokenStats } = useTokenAnalysis(source);

const activeCodeOutputText = computed(() => {
  const code =
    activeCodeOutput.value.isTypeScript && codeViewMode.value === "js"
      ? activeCodeOutput.value.formattedJsCode
      : activeCodeOutput.value.formattedCode || activeCodeOutput.value.code;
  return activeCodeOutput.value.error || code;
});

const activeCodeHighlightKey = computed(
  () => `${codeOutputVersion.value}:${codeOutputTarget.value}:${codeViewMode.value}`,
);

watch(
  () => props.compiler,
  () => {
    if (props.compiler) compile();
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
    compile();
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
  <div class="panel input-panel">
    <div class="panel-header">
      <h2>{{ inputMode === "sfc" ? "SFC (.vue)" : "Template" }}</h2>
      <div class="panel-actions">
        <button class="btn-ghost" @click="handlePresetChange(selectedPreset)">Reset</button>
        <button class="btn-ghost" @click="copyToClipboard(source)">Copy</button>
      </div>
    </div>
    <div class="editor-container">
      <MonacoEditor v-model="source" :language="editorLanguage" :theme />
    </div>
  </div>

  <div class="panel output-panel">
    <div class="panel-header">
      <h2>
        Output
        <span v-if="compileTime !== null" class="compile-time">{{ compileTime.toFixed(4) }}ms</span>
      </h2>
      <div class="tabs">
        <button :class="['tab', { active: activeTab === 'code' }]" @click="activeTab = 'code'">
          Code
        </button>
        <button :class="['tab', { active: activeTab === 'ast' }]" @click="activeTab = 'ast'">
          AST
        </button>
        <button
          v-if="inputMode === 'sfc'"
          :class="['tab', { active: activeTab === 'bindings' }]"
          @click="activeTab = 'bindings'"
        >
          Bindings
        </button>
        <button :class="['tab', { active: activeTab === 'tokens' }]" @click="activeTab = 'tokens'">
          Tokens ({{ tokenStats.total }})
        </button>
        <button
          :class="['tab', { active: activeTab === 'helpers' }]"
          @click="activeTab = 'helpers'"
        >
          Helpers
        </button>
        <template v-if="inputMode === 'sfc'">
          <button :class="['tab', { active: activeTab === 'sfc' }]" @click="activeTab = 'sfc'">
            SFC
          </button>
          <button :class="['tab', { active: activeTab === 'css' }]" @click="activeTab = 'css'">
            CSS
          </button>
        </template>
      </div>
    </div>

    <div class="output-content">
      <div v-if="isCompiling" class="compiling">
        <div class="spinner" />
        <span>Compiling...</span>
      </div>

      <div v-else-if="error" class="wasm-error">
        <h3>Compilation Error</h3>
        <pre>{{ error }}</pre>
      </div>

      <template v-else-if="output">
        <!-- Code Tab -->
        <div v-if="activeTab === 'code'" class="code-output">
          <div class="code-header">
            <h4>{{ CODE_OUTPUT_LABELS[codeOutputTarget] }} Output</h4>
            <div class="code-header-actions">
              <div class="code-mode-toggle">
                <button
                  :class="['toggle-btn', { active: codeOutputTarget === 'dom' }]"
                  @click="codeOutputTarget = 'dom'"
                >
                  VDOM
                </button>
                <button
                  :class="['toggle-btn', { active: codeOutputTarget === 'ssr' }]"
                  @click="codeOutputTarget = 'ssr'"
                >
                  SSR
                </button>
                <button
                  :class="['toggle-btn', { active: codeOutputTarget === 'vapor' }]"
                  @click="codeOutputTarget = 'vapor'"
                >
                  Vapor
                </button>
              </div>
              <div class="code-mode-toggle code-view-toggle">
                <button
                  :class="['toggle-btn', { active: codeViewMode === 'ts' }]"
                  :disabled="!activeCodeOutput.isTypeScript"
                  @click="activeCodeOutput.isTypeScript && (codeViewMode = 'ts')"
                >
                  TS
                </button>
                <button
                  :class="['toggle-btn', { active: codeViewMode === 'js' }]"
                  :disabled="!activeCodeOutput.isTypeScript"
                  @click="activeCodeOutput.isTypeScript && (codeViewMode = 'js')"
                >
                  JS
                </button>
              </div>
              <button class="btn-ghost" @click="copyToClipboard(activeCodeOutputText)">Copy</button>
            </div>
          </div>
          <div v-if="activeCodeOutput.error" class="wasm-error">
            <h3>{{ CODE_OUTPUT_LABELS[codeOutputTarget] }} Compilation Error</h3>
            <pre>{{ activeCodeOutput.error }}</pre>
          </div>
          <template v-else>
            <CodeHighlight
              v-if="activeCodeOutput.isTypeScript && codeViewMode === 'js'"
              :key="`${activeCodeHighlightKey}-js`"
              :code="activeCodeOutput.formattedJsCode"
              language="javascript"
              :theme
              show-line-numbers
            />
            <CodeHighlight
              v-else
              :key="`${activeCodeHighlightKey}-main`"
              :code="activeCodeOutput.formattedCode || activeCodeOutput.code"
              :language="activeCodeOutput.isTypeScript ? 'typescript' : 'javascript'"
              :theme
              show-line-numbers
            />
            <div
              v-if="activeCodeOutput.templates.length > 0 && codeOutputTarget !== 'vapor'"
              class="sfc-block"
            >
              <p class="section-label">
                Template Fragments ({{ activeCodeOutput.templates.length }})
              </p>
              <div
                v-for="(template, index) in activeCodeOutput.templates"
                :key="`${codeOutputTarget}-${index}`"
                class="style-block"
              >
                <span class="style-meta">
                  <span class="badge">template {{ index + 1 }}</span>
                </span>
                <CodeHighlight :code="template" language="javascript" :theme show-line-numbers />
              </div>
            </div>
          </template>
        </div>

        <!-- AST Tab -->
        <div v-else-if="activeTab === 'ast'" class="ast-output">
          <div class="ast-header">
            <h4>Abstract Syntax Tree</h4>
            <div class="ast-options">
              <label class="ast-option">
                <input v-model="astHideLoc" type="checkbox" />
                <span>Hide loc</span>
              </label>
              <label class="ast-option">
                <input v-model="astHideSource" type="checkbox" />
                <span>Hide source</span>
              </label>
              <label class="ast-option">
                <input v-model="astCollapsed" type="checkbox" />
                <span>Compact</span>
              </label>
              <button class="btn-ghost btn-small" @click="copyToClipboard(astJson)">Copy</button>
            </div>
          </div>
          <CodeHighlight :code="astJson" language="json" :theme show-line-numbers />
        </div>

        <!-- Helpers Tab -->
        <div v-else-if="activeTab === 'helpers'" class="helpers-output">
          <h4>Runtime Helpers Used ({{ output.helpers?.length ?? 0 }})</h4>
          <ul v-if="output.helpers?.length > 0" class="helpers-list">
            <li v-for="(helper, i) in output.helpers" :key="i" class="helper-item">
              <span class="helper-name">{{ helper }}</span>
            </li>
          </ul>
          <p v-else class="no-helpers">No runtime helpers needed</p>
        </div>

        <!-- SFC Tab -->
        <div v-else-if="activeTab === 'sfc' && sfcResult" class="sfc-output">
          <h4>SFC Descriptor</h4>

          <div v-if="sfcResult.descriptor.template" class="sfc-block">
            <h5>
              Template
              {{
                sfcResult.descriptor.template.lang ? `(${sfcResult.descriptor.template.lang})` : ""
              }}
            </h5>
            <CodeHighlight :code="sfcResult.descriptor.template.content" language="html" :theme />
          </div>

          <div v-if="sfcResult.descriptor.scriptSetup" class="sfc-block">
            <h5>
              Script Setup
              {{
                sfcResult.descriptor.scriptSetup.lang
                  ? `(${sfcResult.descriptor.scriptSetup.lang})`
                  : ""
              }}
            </h5>
            <CodeHighlight
              :code="sfcResult.descriptor.scriptSetup.content"
              language="typescript"
              :theme
            />
          </div>

          <div v-if="sfcResult.descriptor.script" class="sfc-block">
            <h5>
              Script
              {{ sfcResult.descriptor.script.lang ? `(${sfcResult.descriptor.script.lang})` : "" }}
            </h5>
            <CodeHighlight
              :code="sfcResult.descriptor.script.content"
              language="typescript"
              :theme
            />
          </div>

          <div v-if="sfcResult.descriptor.styles?.length > 0" class="sfc-block">
            <h5>Styles ({{ sfcResult.descriptor.styles?.length }})</h5>
            <div v-for="(style, i) in sfcResult.descriptor.styles" :key="i" class="style-block">
              <span class="style-meta">
                <span v-if="style.scoped" class="badge">scoped</span>
                <span v-if="style.lang" class="badge">{{ style.lang }}</span>
              </span>
              <CodeHighlight :code="style.content" language="css" :theme />
            </div>
          </div>
        </div>

        <!-- CSS Tab -->
        <div v-else-if="activeTab === 'css'" class="css-output">
          <h4>CSS Compilation (LightningCSS)</h4>

          <div class="css-options">
            <label class="option checkbox">
              <input v-model="cssOptions.minify" type="checkbox" />
              <span>Minify</span>
            </label>
            <label class="option checkbox">
              <input v-model="cssOptions.scoped" type="checkbox" />
              <span>Force Scoped</span>
            </label>
          </div>

          <template v-if="cssResult">
            <div class="css-compiled">
              <h5>Compiled CSS</h5>
              <div class="code-actions">
                <button class="btn-ghost" @click="copyToClipboard(formattedCss || cssResult.code)">
                  Copy
                </button>
              </div>
              <CodeHighlight
                :code="formattedCss || cssResult.code"
                language="css"
                :theme
                show-line-numbers
              />
            </div>

            <div v-if="cssResult.cssVars?.length > 0" class="css-vars">
              <h5>CSS Variables (v-bind)</h5>
              <ul class="helpers-list">
                <li v-for="(v, i) in cssResult.cssVars" :key="i" class="helper-item">
                  <span class="helper-name">{{ v }}</span>
                </li>
              </ul>
            </div>

            <div v-if="cssResult.errors?.length > 0" class="css-errors">
              <h5>Errors</h5>
              <pre v-for="(err, i) in cssResult.errors" :key="i" class="error-message">{{
                err
              }}</pre>
            </div>
          </template>
          <p v-else class="no-css">No styles in this SFC</p>
        </div>

        <!-- Bindings Tab -->
        <div
          v-else-if="activeTab === 'bindings' && sfcResult?.script?.bindings"
          class="bindings-output"
        >
          <h4>Script Setup Bindings</h4>

          <div class="bindings-summary">
            <div v-for="(count, type) in bindingsSummary" :key="type" class="summary-card">
              <span class="summary-count">{{ count }}</span>
              <span :class="['summary-type', `type-${type}`]">{{ type }}</span>
            </div>
          </div>

          <div class="bindings-groups">
            <div v-for="(vars, type) in groupedBindings" :key="type" class="binding-group">
              <div :class="['group-header', `type-${type}`]">
                <span class="group-icon">{{ getBindingIcon(type as string) }}</span>
                <span class="group-title">{{ getBindingLabel(type as string) }}</span>
                <span class="group-count">{{ vars.length }}</span>
              </div>
              <div class="group-vars">
                <span v-for="v in vars" :key="v" :class="['var-chip', `type-${type}`]">{{
                  v
                }}</span>
              </div>
            </div>
          </div>
        </div>
        <div v-else-if="activeTab === 'bindings'" class="bindings-output">
          <p class="no-bindings">No bindings information available</p>
        </div>

        <!-- Tokens Tab -->
        <div v-else-if="activeTab === 'tokens'" class="tokens-output">
          <div class="token-stats">
            <div class="stat-card">
              <span class="stat-value">{{ tokenStats.total }}</span>
              <span class="stat-label">Total</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ tokenStats.tags }}</span>
              <span class="stat-label">Tags</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ tokenStats.directives }}</span>
              <span class="stat-label">Directives</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ tokenStats.interpolations }}</span>
              <span class="stat-label">Interpolations</span>
            </div>
          </div>

          <h4>Token Stream</h4>
          <div class="token-stream">
            <div
              v-for="(token, i) in lexicalTokens"
              :key="i"
              class="token-item"
              :style="{ '--token-color': getTokenTypeColor(token.type) }"
            >
              <span class="token-badge" :style="{ background: getTokenTypeColor(token.type) }">
                {{ getTokenTypeIcon(token.type) }}
              </span>
              <div class="token-content">
                <div class="token-main">
                  <span v-if="token.name" class="token-name">{{ token.name }}</span>
                  <span v-if="token.value" class="token-value-text">{{ token.value }}</span>
                </div>
                <span class="token-location">{{ token.line }}:{{ token.column }}</span>
              </div>
            </div>
          </div>

          <h4>By Type</h4>
          <div class="token-groups">
            <template v-for="(tokens, type) in tokensByType">
              <div v-if="tokens.length > 0" :key="type" class="token-group">
                <div
                  class="group-header"
                  :style="{
                    borderLeftColor: getTokenTypeColor(String(type)),
                  }"
                >
                  <span
                    class="group-icon"
                    :style="{
                      background: getTokenTypeColor(String(type)),
                    }"
                  >
                    {{ getTokenTypeIcon(String(type)) }}
                  </span>
                  <span class="group-title">{{ getTokenTypeLabel(String(type)) }}</span>
                  <span class="group-count">{{ tokens.length }}</span>
                </div>
                <div class="group-tokens">
                  <span
                    v-for="(token, i) in tokens.slice(0, 12)"
                    :key="i"
                    class="group-token-chip"
                    :style="{
                      '--chip-color': getTokenTypeColor(String(type)),
                    }"
                  >
                    {{ token.name || token.value?.slice(0, 25) || token.raw.slice(0, 25) }}
                  </span>
                  <span v-if="tokens.length > 12" class="more-indicator">
                    +{{ tokens.length - 12 }} more
                  </span>
                </div>
              </div>
            </template>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>
