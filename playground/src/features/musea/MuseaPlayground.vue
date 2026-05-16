<script setup lang="ts">
import "./MuseaPlayground.css";
import { ref, watch, computed, inject, onMounted, onUnmounted, type ComputedRef } from "vue";
import MonacoEditor from "../../shared/MonacoEditor.vue";
import CodeHighlight from "../../shared/CodeHighlight.vue";
import { type WasmModule, getWasm } from "../../wasm/index";
import { ART_PRESET } from "../../shared/presets/musea";
import { mdiPalette, mdiDiamond } from "@mdi/js";
import { useArtParsing } from "./useArtParsing";

const props = defineProps<{
  compiler: WasmModule | null;
}>();
const _injectedTheme = inject<ComputedRef<"dark" | "light">>("theme");
const theme = computed<"dark" | "light">(() => _injectedTheme?.value ?? "light");

const source = ref(ART_PRESET);

const {
  parsedArt,
  csfOutput,
  error,
  diagnostics,
  compileTime,
  designTokens,
  colorTokens,
  sizeTokens,
  otherTokens,
  variantCount,
  compile,
} = useArtParsing(source, () => props.compiler ?? getWasm());

type TabType = "parsed" | "csf" | "variants";
const validTabs: TabType[] = ["parsed", "csf", "variants"];

function getTabFromUrl(): TabType {
  const params = new URLSearchParams(window.location.search);
  const tab = params.get("tab");
  if (tab && validTabs.includes(tab as TabType)) {
    return tab as TabType;
  }
  return "parsed";
}

function setTabToUrl(tab: TabType) {
  const url = new URL(window.location.href);
  url.searchParams.set("tab", tab);
  window.history.replaceState({}, "", url.toString());
}

const activeTab = ref<TabType>(getTabFromUrl());

watch(activeTab, (tab) => {
  setTabToUrl(tab);
});

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
}

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
  <div class="musea-playground">
    <div class="panel input-panel">
      <div class="panel-header">
        <div class="header-title">
          <svg class="icon" viewBox="0 0 24 24"><path :d="mdiPalette" fill="currentColor" /></svg>
          <h2>Source</h2>
        </div>
        <div class="panel-actions">
          <a href="/musea-examples/__musea__" target="_blank" rel="noopener" class="btn-examples">
            <svg
              viewBox="0 0 24 24"
              width="12"
              height="12"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
              <polyline points="15 3 21 3 21 9" />
              <line x1="10" y1="14" x2="21" y2="3" />
            </svg>
            Examples
          </a>
          <button class="btn-ghost" @click="source = ART_PRESET">Reset</button>
        </div>
      </div>
      <div class="editor-container">
        <MonacoEditor v-model="source" language="vue" :diagnostics :theme />
      </div>
    </div>

    <div class="panel output-panel">
      <div class="panel-header">
        <div class="header-title">
          <svg class="icon" viewBox="0 0 24 24"><path :d="mdiDiamond" fill="currentColor" /></svg>
          <h2>Art Analysis</h2>
          <span v-if="compileTime !== null" class="perf-badge">
            {{ compileTime.toFixed(2) }}ms
          </span>
        </div>
        <div class="tabs">
          <button
            :class="['tab', { active: activeTab === 'parsed' }]"
            @click="activeTab = 'parsed'"
          >
            Metadata
          </button>
          <button
            :class="['tab', { active: activeTab === 'variants' }]"
            @click="activeTab = 'variants'"
          >
            Variants
            <span v-if="variantCount > 0" class="tab-count">{{ variantCount }}</span>
          </button>
          <button :class="['tab', { active: activeTab === 'csf' }]" @click="activeTab = 'csf'">
            CSF
          </button>
        </div>
      </div>

      <div class="output-content">
        <div v-if="error" class="error-panel">
          <div class="error-header">Parse Error</div>
          <pre class="error-content">{{ error }}</pre>
        </div>

        <template v-else-if="parsedArt">
          <!-- Parsed Tab -->
          <div v-if="activeTab === 'parsed'" class="parsed-output">
            <div class="output-header-bar">
              <span class="output-title">Component Metadata</span>
              <div class="file-badges">
                <span v-if="parsedArt.hasScriptSetup" class="file-badge">setup</span>
                <span v-if="parsedArt.hasScript" class="file-badge">script</span>
                <span v-if="parsedArt.styleCount > 0" class="file-badge"
                  >{{ parsedArt.styleCount }} style</span
                >
              </div>
            </div>

            <div class="metadata-section">
              <div class="metadata-grid">
                <div class="metadata-item">
                  <span class="meta-label">Title</span>
                  <span class="meta-value">{{ parsedArt.metadata.title }}</span>
                </div>
                <div v-if="parsedArt.metadata.description" class="metadata-item span-full">
                  <span class="meta-label">Description</span>
                  <span class="meta-value">{{ parsedArt.metadata.description }}</span>
                </div>
                <div v-if="parsedArt.metadata.component" class="metadata-item">
                  <span class="meta-label">Component</span>
                  <code class="meta-code">{{ parsedArt.metadata.component }}</code>
                </div>
                <div v-if="parsedArt.metadata.category" class="metadata-item">
                  <span class="meta-label">Category</span>
                  <span class="meta-value category-value">{{ parsedArt.metadata.category }}</span>
                </div>
                <div v-if="parsedArt.metadata.tags?.length" class="metadata-item">
                  <span class="meta-label">Tags</span>
                  <span class="tags-list">
                    <span v-for="tag in parsedArt.metadata.tags" :key="tag" class="tag-item">{{
                      tag
                    }}</span>
                  </span>
                </div>
                <div class="metadata-item">
                  <span class="meta-label">Status</span>
                  <span :class="['status-badge', parsedArt.metadata.status]">{{
                    parsedArt.metadata.status
                  }}</span>
                </div>
              </div>
            </div>

            <!-- Design Tokens -->
            <template v-if="designTokens.length > 0">
              <div class="section-header">
                <span class="section-title">Design Tokens</span>
                <span class="section-count">{{ designTokens.length }}</span>
              </div>

              <!-- Color Tokens -->
              <div v-if="colorTokens.length > 0" class="token-section">
                <div class="token-category">Colors</div>
                <div class="color-grid">
                  <div
                    v-for="token in colorTokens"
                    :key="token.name"
                    class="color-token"
                    role="button"
                    tabindex="0"
                    :title="`Click to copy: ${token.name}`"
                    @click="copyToClipboard(token.name)"
                    @keydown.enter="copyToClipboard(token.name)"
                  >
                    <div class="color-swatch" :style="{ background: token.value }"></div>
                    <div class="token-info">
                      <code class="token-name">{{ token.name }}</code>
                      <span class="token-value">{{ token.value }}</span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Size Tokens -->
              <div v-if="sizeTokens.length > 0" class="token-section">
                <div class="token-category">Sizes</div>
                <div class="token-list">
                  <div
                    v-for="token in sizeTokens"
                    :key="token.name"
                    class="size-token"
                    role="button"
                    tabindex="0"
                    :title="`Click to copy: ${token.name}`"
                    @click="copyToClipboard(token.name)"
                    @keydown.enter="copyToClipboard(token.name)"
                  >
                    <code class="token-name">{{ token.name }}</code>
                    <span class="token-value">{{ token.value }}</span>
                    <div class="size-preview" :style="{ width: token.value }"></div>
                  </div>
                </div>
              </div>

              <!-- Other Tokens -->
              <div v-if="otherTokens.length > 0" class="token-section">
                <div class="token-category">Other</div>
                <div class="token-list">
                  <div
                    v-for="token in otherTokens"
                    :key="token.name"
                    class="other-token"
                    role="button"
                    tabindex="0"
                    :title="`Click to copy: ${token.name}`"
                    @click="copyToClipboard(token.name)"
                    @keydown.enter="copyToClipboard(token.name)"
                  >
                    <code class="token-name">{{ token.name }}</code>
                    <span class="token-value">{{ token.value }}</span>
                  </div>
                </div>
              </div>
            </template>
          </div>

          <!-- Variants Tab -->
          <div v-else-if="activeTab === 'variants'" class="variants-output">
            <div class="output-header-bar">
              <span class="output-title">Variants</span>
              <span class="variant-count"
                >{{ parsedArt.variants.length }} variant{{
                  parsedArt.variants.length !== 1 ? "s" : ""
                }}</span
              >
            </div>

            <div class="variants-list">
              <div v-for="variant in parsedArt.variants" :key="variant.name" class="variant-item">
                <div class="variant-header">
                  <div class="variant-name">
                    {{ variant.name }}
                    <span v-if="variant.isDefault" class="default-badge">default</span>
                    <span v-if="variant.skipVrt" class="skip-badge">skip vrt</span>
                  </div>
                  <button class="btn-copy" @click="copyToClipboard(variant.template)">Copy</button>
                </div>
                <div class="variant-template">
                  <CodeHighlight :code="variant.template" language="html" :theme />
                </div>
              </div>
            </div>
          </div>

          <!-- CSF Tab -->
          <div v-else-if="activeTab === 'csf' && csfOutput" class="csf-output">
            <div class="output-header-bar">
              <span class="output-title">Storybook CSF</span>
              <div class="csf-actions">
                <code class="filename-badge">{{ csfOutput.filename }}</code>
                <button class="btn-copy" @click="copyToClipboard(csfOutput.code)">Copy</button>
              </div>
            </div>
            <div class="code-container">
              <CodeHighlight
                :code="csfOutput.code"
                language="typescript"
                show-line-numbers
                :theme
              />
            </div>
          </div>
        </template>

        <div v-else class="loading-state">
          <span>Enter an Art file to see the output</span>
        </div>
      </div>
    </div>
  </div>
</template>
