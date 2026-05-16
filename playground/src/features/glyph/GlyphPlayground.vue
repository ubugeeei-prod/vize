<script setup lang="ts">
import "./GlyphPlayground.css";
import { ref, watch, computed, inject, toRaw, onMounted, onUnmounted, type ComputedRef } from "vue";
import MonacoEditor from "../../shared/MonacoEditor.vue";
import CodeHighlight from "../../shared/CodeHighlight.vue";
import type { WasmModule, FormatOptions, FormatResult } from "../../wasm/index";
import { getWasm } from "../../wasm/index";
import { GLYPH_PRESET } from "../../shared/presets/glyph";
import { mdiFileEdit, mdiAutoFix, mdiCheck } from "@mdi/js";

const props = defineProps<{
  compiler: WasmModule | null;
}>();
const _injectedTheme = inject<ComputedRef<"dark" | "light">>("theme");
const theme = computed<"dark" | "light">(() => _injectedTheme?.value ?? "light");

const source = ref(GLYPH_PRESET);
const formatResult = ref<FormatResult | null>(null);
const formatKey = ref(0);
const error = ref<string | null>(null);
const formatTime = ref<number | null>(null);
const activeTab = ref<"formatted" | "diff" | "options">("formatted");

// Format options
const options = ref<FormatOptions>({
  printWidth: 100,
  tabWidth: 2,
  useTabs: false,
  semi: true,
  singleQuote: false,
  jsxSingleQuote: false,
  trailingComma: "all",
  bracketSpacing: true,
  bracketSameLine: false,
  arrowParens: "always",
  endOfLine: "lf",
  quoteProps: "as-needed",
  singleAttributePerLine: false,
  vueIndentScriptAndStyle: false,
  sortAttributes: true,
  attributeSortOrder: "alphabetical",
  mergeBindAndNonBindAttrs: false,
  maxAttributesPerLine: null,
  normalizeDirectiveShorthands: true,
  sortBlocks: true,
});

const diffLines = computed(() => {
  if (!formatResult.value) return [];

  const original = source.value.split("\n");
  const formatted = formatResult.value.code.split("\n");
  const diff: Array<{
    type: "same" | "removed" | "added";
    content: string;
    lineNum: number;
  }> = [];

  // Simple diff - just show removed and added lines
  const maxLen = Math.max(original.length, formatted.length);
  let origIdx = 0;
  let fmtIdx = 0;

  while (origIdx < original.length || fmtIdx < formatted.length) {
    const origLine = original[origIdx];
    const fmtLine = formatted[fmtIdx];

    if (origLine === fmtLine) {
      diff.push({
        type: "same",
        content: origLine || "",
        lineNum: origIdx + 1,
      });
      origIdx++;
      fmtIdx++;
    } else if (origLine !== undefined && fmtLine !== undefined) {
      diff.push({ type: "removed", content: origLine, lineNum: origIdx + 1 });
      diff.push({ type: "added", content: fmtLine, lineNum: fmtIdx + 1 });
      origIdx++;
      fmtIdx++;
    } else if (origLine !== undefined) {
      diff.push({ type: "removed", content: origLine, lineNum: origIdx + 1 });
      origIdx++;
    } else if (fmtLine !== undefined) {
      diff.push({ type: "added", content: fmtLine, lineNum: fmtIdx + 1 });
      fmtIdx++;
    }
  }

  return diff;
});

async function format() {
  const compiler = getWasm();
  if (!compiler) return;

  const startTime = performance.now();
  error.value = null;

  try {
    const raw = toRaw(options.value);
    const opts: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(raw)) {
      if (v != null) opts[k] = v;
    }
    const result = compiler.formatSfc(source.value, opts);
    formatResult.value = result;
    formatKey.value++;
    formatTime.value = performance.now() - startTime;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
    formatResult.value = null;
  }
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
}

let formatTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  [source, options],
  () => {
    if (!getWasm()) return;
    if (formatTimer) clearTimeout(formatTimer);
    formatTimer = setTimeout(format, 300);
  },
  { deep: true },
);

let hasInitialized = false;
let pollInterval: ReturnType<typeof setInterval> | null = null;

function tryInitialize() {
  const compiler = getWasm();
  if (compiler && !hasInitialized) {
    hasInitialized = true;
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
    format();
  }
}

onMounted(() => {
  tryInitialize();
  if (!hasInitialized) {
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
  <div class="glyph-playground">
    <div class="panel input-panel">
      <div class="panel-header">
        <div class="header-title">
          <svg class="icon" viewBox="0 0 24 24">
            <path :d="mdiFileEdit" fill="currentColor" />
          </svg>
          <h2>Source</h2>
        </div>
        <div class="panel-actions">
          <button class="btn-ghost" @click="source = GLYPH_PRESET">Reset</button>
          <button class="btn-ghost" @click="copyToClipboard(source)">Copy</button>
        </div>
      </div>
      <div class="editor-container">
        <MonacoEditor v-model="source" language="html" :theme />
      </div>
    </div>

    <div class="panel output-panel">
      <div class="panel-header">
        <div class="header-title">
          <svg class="icon" viewBox="0 0 24 24">
            <path :d="mdiAutoFix" fill="currentColor" />
          </svg>
          <h2>Code Formatting</h2>
          <span v-if="formatTime !== null" class="perf-badge"> {{ formatTime.toFixed(2) }}ms </span>
          <span
            v-if="formatResult"
            :class="['status-badge', formatResult.changed ? 'changed' : 'unchanged']"
          >
            {{ formatResult.changed ? "Changed" : "Unchanged" }}
          </span>
        </div>
        <div class="tabs">
          <button
            :class="['tab', { active: activeTab === 'formatted' }]"
            @click="activeTab = 'formatted'"
          >
            Formatted
          </button>
          <button :class="['tab', { active: activeTab === 'diff' }]" @click="activeTab = 'diff'">
            Diff
          </button>
          <button
            :class="['tab', { active: activeTab === 'options' }]"
            @click="activeTab = 'options'"
          >
            Options
          </button>
        </div>
      </div>

      <div class="output-content">
        <div v-if="error" class="error-panel">
          <div class="error-header">Format Error</div>
          <pre class="error-content">{{ error }}</pre>
        </div>

        <template v-else-if="formatResult">
          <!-- Formatted Tab -->
          <div v-if="activeTab === 'formatted'" class="formatted-output">
            <div class="output-header-bar">
              <span class="output-title">Formatted Code</span>
              <div class="output-actions">
                <button class="btn-ghost" @click="copyToClipboard(formatResult?.code || '')">
                  Copy
                </button>
              </div>
            </div>
            <div class="code-container">
              <CodeHighlight
                :key="formatKey"
                :code="formatResult.code"
                language="html"
                show-line-numbers
                :theme
              />
            </div>
          </div>

          <!-- Diff Tab -->
          <div v-else-if="activeTab === 'diff'" class="diff-output">
            <div class="output-header-bar">
              <span class="output-title">Changes</span>
              <span class="diff-stats">
                <span class="stat additions"
                  >+{{ diffLines.filter((l) => l.type === "added").length }}</span
                >
                <span class="stat deletions"
                  >-{{ diffLines.filter((l) => l.type === "removed").length }}</span
                >
              </span>
            </div>
            <div v-if="!formatResult.changed" class="success-state">
              <svg class="success-icon" viewBox="0 0 24 24">
                <path :d="mdiCheck" fill="currentColor" />
              </svg>
              <span>No changes needed</span>
            </div>
            <div v-else class="diff-view">
              <div class="diff-line-numbers">
                <span v-for="(_line, i) in diffLines" :key="i" class="diff-ln">{{ i + 1 }}</span>
              </div>
              <div class="diff-code">
                <div
                  v-for="(line, i) in diffLines"
                  :key="i"
                  :class="['diff-line', `diff-${line.type}`]"
                >
                  <span class="line-prefix">{{
                    line.type === "removed" ? "-" : line.type === "added" ? "+" : " "
                  }}</span>
                  <span class="line-content">{{ line.content || " " }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Options Tab -->
          <div v-else-if="activeTab === 'options'" class="options-output">
            <div class="output-header-bar">
              <span class="output-title">Format Configuration</span>
            </div>
            <div class="options-content">
              <div class="options-section">
                <h3 class="section-title">Layout</h3>
                <div class="options-grid">
                  <div class="option-card">
                    <div class="option-header">
                      <span class="option-name">Print Width</span>
                      <input
                        v-model.number="options.printWidth"
                        type="number"
                        min="40"
                        max="200"
                        aria-label="Print Width"
                        class="option-input"
                      />
                    </div>
                    <span class="option-desc">Maximum line length before wrapping</span>
                  </div>
                  <div class="option-card">
                    <div class="option-header">
                      <span class="option-name">Tab Width</span>
                      <input
                        v-model.number="options.tabWidth"
                        type="number"
                        min="1"
                        max="8"
                        aria-label="Tab Width"
                        class="option-input"
                      />
                    </div>
                    <span class="option-desc">Number of spaces per indentation level</span>
                  </div>
                </div>
              </div>

              <div class="options-section">
                <h3 class="section-title">Style</h3>
                <div class="toggle-grid">
                  <div class="toggle-card">
                    <div class="toggle-main">
                      <input
                        v-model="options.useTabs"
                        type="checkbox"
                        aria-label="Use Tabs"
                        class="toggle-checkbox"
                      />
                      <span class="toggle-name">Use Tabs</span>
                    </div>
                    <span class="toggle-desc">Indent with tabs instead of spaces</span>
                  </div>
                  <div class="toggle-card">
                    <div class="toggle-main">
                      <input
                        v-model="options.semi"
                        type="checkbox"
                        aria-label="Semicolons"
                        class="toggle-checkbox"
                      />
                      <span class="toggle-name">Semicolons</span>
                    </div>
                    <span class="toggle-desc">Add semicolons at the end of statements</span>
                  </div>
                  <div class="toggle-card">
                    <div class="toggle-main">
                      <input
                        v-model="options.singleQuote"
                        type="checkbox"
                        aria-label="Single Quotes"
                        class="toggle-checkbox"
                      />
                      <span class="toggle-name">Single Quotes</span>
                    </div>
                    <span class="toggle-desc">Use single quotes instead of double quotes</span>
                  </div>
                  <div class="toggle-card">
                    <div class="toggle-main">
                      <input
                        v-model="options.bracketSpacing"
                        type="checkbox"
                        aria-label="Bracket Spacing"
                        class="toggle-checkbox"
                      />
                      <span class="toggle-name">Bracket Spacing</span>
                    </div>
                    <span class="toggle-desc"
                      >Print spaces between brackets in object literals</span
                    >
                  </div>
                  <div class="toggle-card">
                    <div class="toggle-main">
                      <input
                        v-model="options.bracketSameLine"
                        type="checkbox"
                        aria-label="Bracket Same Line"
                        class="toggle-checkbox"
                      />
                      <span class="toggle-name">Bracket Same Line</span>
                    </div>
                    <span class="toggle-desc">Put closing bracket on the same line</span>
                  </div>
                  <div class="toggle-card">
                    <div class="toggle-main">
                      <input
                        v-model="options.jsxSingleQuote"
                        type="checkbox"
                        aria-label="JSX Single Quotes"
                        class="toggle-checkbox"
                      />
                      <span class="toggle-name">JSX Single Quotes</span>
                    </div>
                    <span class="toggle-desc">Use single quotes in JSX</span>
                  </div>
                </div>
              </div>

              <div class="options-section">
                <h3 class="section-title">Trailing / Parens</h3>
                <div class="options-grid">
                  <div class="option-card">
                    <div class="option-header">
                      <span class="option-name">Trailing Comma</span>
                      <select
                        v-model="options.trailingComma"
                        aria-label="Trailing Comma"
                        class="option-select"
                      >
                        <option value="all">All</option>
                        <option value="es5">ES5</option>
                        <option value="none">None</option>
                      </select>
                    </div>
                    <span class="option-desc">Print trailing commas wherever possible</span>
                  </div>
                  <div class="option-card">
                    <div class="option-header">
                      <span class="option-name">Arrow Parens</span>
                      <select
                        v-model="options.arrowParens"
                        aria-label="Arrow Parens"
                        class="option-select"
                      >
                        <option value="always">Always</option>
                        <option value="avoid">Avoid</option>
                      </select>
                    </div>
                    <span class="option-desc"
                      >Parentheses around a sole arrow function parameter</span
                    >
                  </div>
                  <div class="option-card">
                    <div class="option-header">
                      <span class="option-name">Quote Props</span>
                      <select
                        v-model="options.quoteProps"
                        aria-label="Quote Props"
                        class="option-select"
                      >
                        <option value="as-needed">As Needed</option>
                        <option value="consistent">Consistent</option>
                        <option value="preserve">Preserve</option>
                      </select>
                    </div>
                    <span class="option-desc">When to quote object properties</span>
                  </div>
                  <div class="option-card">
                    <div class="option-header">
                      <span class="option-name">End of Line</span>
                      <select
                        v-model="options.endOfLine"
                        aria-label="End of Line"
                        class="option-select"
                      >
                        <option value="lf">LF</option>
                        <option value="crlf">CRLF</option>
                        <option value="cr">CR</option>
                        <option value="auto">Auto</option>
                      </select>
                    </div>
                    <span class="option-desc">End of line style</span>
                  </div>
                </div>
              </div>

              <div class="options-section">
                <h3 class="section-title">Vue SFC</h3>
                <div class="toggle-grid" style="margin-bottom: 0.75rem">
                  <div class="toggle-card">
                    <div class="toggle-main">
                      <input
                        v-model="options.sortBlocks"
                        type="checkbox"
                        aria-label="Sort Blocks"
                        class="toggle-checkbox"
                      />
                      <span class="toggle-name">Sort Blocks</span>
                    </div>
                    <span class="toggle-desc"
                      >Reorder blocks: script → setup → template → style</span
                    >
                  </div>
                </div>
                <div class="options-grid">
                  <div class="option-card">
                    <div class="option-header">
                      <span class="option-name">Attribute Sort Order</span>
                      <select
                        v-model="options.attributeSortOrder"
                        aria-label="Attribute Sort Order"
                        class="option-select"
                      >
                        <option value="alphabetical">Alphabetical</option>
                        <option value="as-written">As Written</option>
                      </select>
                    </div>
                    <span class="option-desc">How to sort attributes within priority groups</span>
                  </div>
                  <div class="option-card">
                    <div class="option-header">
                      <span class="option-name">Max Attrs Per Line</span>
                      <input
                        :value="options.maxAttributesPerLine ?? ''"
                        type="number"
                        min="1"
                        max="20"
                        placeholder="auto"
                        aria-label="Max Attrs Per Line"
                        class="option-input"
                        @input="
                          options.maxAttributesPerLine =
                            ($event.target as HTMLInputElement).value === ''
                              ? null
                              : Number(($event.target as HTMLInputElement).value)
                        "
                      />
                    </div>
                    <span class="option-desc">Max attributes per line before wrapping</span>
                  </div>
                </div>
                <div class="toggle-grid" style="margin-top: 0.75rem">
                  <div class="toggle-card">
                    <div class="toggle-main">
                      <input
                        v-model="options.sortAttributes"
                        type="checkbox"
                        aria-label="Sort Attributes"
                        class="toggle-checkbox"
                      />
                      <span class="toggle-name">Sort Attributes</span>
                    </div>
                    <span class="toggle-desc">Sort HTML attributes in template</span>
                  </div>
                  <div class="toggle-card">
                    <div class="toggle-main">
                      <input
                        v-model="options.normalizeDirectiveShorthands"
                        type="checkbox"
                        aria-label="Normalize Directives"
                        class="toggle-checkbox"
                      />
                      <span class="toggle-name">Normalize Directives</span>
                    </div>
                    <span class="toggle-desc">Normalize v-bind/v-on/v-slot to shorthand</span>
                  </div>
                  <div class="toggle-card">
                    <div class="toggle-main">
                      <input
                        v-model="options.singleAttributePerLine"
                        type="checkbox"
                        aria-label="Single Attribute Per Line"
                        class="toggle-checkbox"
                      />
                      <span class="toggle-name">Single Attribute Per Line</span>
                    </div>
                    <span class="toggle-desc">Enforce single attribute per line in templates</span>
                  </div>
                  <div class="toggle-card">
                    <div class="toggle-main">
                      <input
                        v-model="options.vueIndentScriptAndStyle"
                        type="checkbox"
                        aria-label="Indent Script/Style"
                        class="toggle-checkbox"
                      />
                      <span class="toggle-name">Indent Script/Style</span>
                    </div>
                    <span class="toggle-desc">Indent code inside script and style tags</span>
                  </div>
                  <div class="toggle-card">
                    <div class="toggle-main">
                      <input
                        v-model="options.mergeBindAndNonBindAttrs"
                        type="checkbox"
                        aria-label="Merge Bind Attrs"
                        class="toggle-checkbox"
                      />
                      <span class="toggle-name">Merge Bind Attrs</span>
                    </div>
                    <span class="toggle-desc">Merge bind and non-bind attrs for sorting</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </template>

        <div v-else class="loading-state">
          <span>Enter Vue code to format</span>
        </div>
      </div>
    </div>
  </div>
</template>
