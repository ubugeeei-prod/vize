<script setup lang="ts">
import { ref, computed, watch, onMounted, shallowRef, provide } from "vue";
import { loadWasm } from "./wasm/index";
import AtelierPlayground from "./features/atelier/AtelierPlayground.vue";
import MuseaPlayground from "./features/musea/MuseaPlayground.vue";
import PatinaPlayground from "./features/patina/PatinaPlayground.vue";
import GlyphPlayground from "./features/glyph/GlyphPlayground.vue";
import CroquisPlayground from "./features/croquis/CroquisPlayground.vue";
import CrossFilePlayground from "./features/cross-file/CrossFilePlayground.vue";
import TypeCheckPlayground from "./features/canon/TypeCheckPlayground.vue";

// Theme toggle
const isDark = ref(false);
const currentTheme = computed<"dark" | "light">(() => (isDark.value ? "dark" : "light"));
provide("theme", currentTheme);
function toggleTheme() {
  isDark.value = !isDark.value;
  document.body.dataset.theme = isDark.value ? "dark" : "";
}

// Main tab
type MainTab = "atelier" | "patina" | "canon" | "croquis" | "cross-file" | "musea" | "glyph";
const validTabs: MainTab[] = [
  "atelier",
  "patina",
  "canon",
  "croquis",
  "cross-file",
  "musea",
  "glyph",
];

function getInitialTab(): MainTab {
  const params = new URLSearchParams(window.location.search);
  const tab = params.get("tab");
  if (tab && validTabs.includes(tab as MainTab)) {
    return tab as MainTab;
  }
  return "atelier";
}

const mainTab = ref<MainTab>(getInitialTab());

watch(mainTab, (newTab) => {
  const url = new URL(window.location.href);
  url.searchParams.set("tab", newTab);
  window.history.replaceState({}, "", url.toString());
});

// WASM
const wasmStatus = ref<"loading" | "ready" | "mock">("loading");
const compiler = shallowRef<Awaited<ReturnType<typeof loadWasm>> | null>(null);

onMounted(async () => {
  try {
    const loaded = await loadWasm();
    compiler.value = loaded;
    wasmStatus.value = "ready";
  } catch (e) {
    console.error("Failed to load WASM:", e);
  }
});
</script>

<template>
  <div class="app">
    <header class="header">
      <div class="logo">
        <div class="logo-icon">
          <svg viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
            <g transform="translate(15, 10) skewX(-15)">
              <path d="M 65 0 L 40 60 L 70 20 L 65 0 Z" fill="currentColor" />
              <path d="M 20 0 L 40 60 L 53 13 L 20 0 Z" fill="currentColor" />
            </g>
          </svg>
        </div>
        <div class="logo-text">
          <h1>Vize</h1>
          <span class="version">
            Playground
            <span :class="['wasm-status', wasmStatus]">
              {{
                wasmStatus === "loading"
                  ? " (Loading...)"
                  : wasmStatus === "mock"
                    ? " (Mock)"
                    : " (WASM)"
              }}
            </span>
          </span>
        </div>
      </div>

      <div class="main-tabs">
        <button
          :class="['main-tab', { active: mainTab === 'atelier' }]"
          @click="mainTab = 'atelier'"
        >
          <span class="tab-name">Atelier</span>
          <span class="tab-desc">compiler</span>
        </button>
        <button :class="['main-tab', { active: mainTab === 'patina' }]" @click="mainTab = 'patina'">
          <span class="tab-name">Patina</span>
          <span class="tab-desc">linter</span>
        </button>
        <button :class="['main-tab', { active: mainTab === 'glyph' }]" @click="mainTab = 'glyph'">
          <span class="tab-name">Glyph</span>
          <span class="tab-desc">formatter</span>
        </button>
        <button :class="['main-tab', { active: mainTab === 'canon' }]" @click="mainTab = 'canon'">
          <span class="tab-name">Canon</span>
          <span class="tab-desc">typecheck</span>
        </button>
        <button
          :class="['main-tab', { active: mainTab === 'croquis' }]"
          @click="mainTab = 'croquis'"
        >
          <span class="tab-name">Croquis</span>
          <span class="tab-desc">analyzer</span>
        </button>
        <button
          :class="['main-tab', { active: mainTab === 'cross-file' }]"
          @click="mainTab = 'cross-file'"
        >
          <span class="tab-name">Cross</span>
          <span class="tab-desc">xfile</span>
        </button>
        <button :class="['main-tab', { active: mainTab === 'musea' }]" @click="mainTab = 'musea'">
          <span class="tab-name">Musea</span>
          <span class="tab-desc">story</span>
        </button>
      </div>

      <div class="options">
        <button
          class="theme-toggle"
          :title="isDark ? 'Light mode' : 'Dark mode'"
          @click="toggleTheme"
        >
          <svg
            v-if="isDark"
            viewBox="0 0 24 24"
            width="18"
            height="18"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="12" cy="12" r="5" />
            <line x1="12" y1="1" x2="12" y2="3" />
            <line x1="12" y1="21" x2="12" y2="23" />
            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
            <line x1="1" y1="12" x2="3" y2="12" />
            <line x1="21" y1="12" x2="23" y2="12" />
            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
          </svg>
          <svg
            v-else
            viewBox="0 0 24 24"
            width="18"
            height="18"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
          </svg>
        </button>

        <a
          href="https://github.com/ubugeeei/vize"
          target="_blank"
          rel="noopener noreferrer"
          class="github-link"
          aria-label="GitHub repository"
        >
          <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
            <path
              d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"
            />
          </svg>
        </a>
      </div>
    </header>

    <main class="main">
      <template v-if="mainTab === 'patina'">
        <PatinaPlayground :compiler />
      </template>
      <template v-else-if="mainTab === 'canon'">
        <TypeCheckPlayground :compiler />
      </template>
      <template v-else-if="mainTab === 'croquis'">
        <CroquisPlayground :compiler />
      </template>
      <template v-else-if="mainTab === 'cross-file'">
        <CrossFilePlayground :compiler />
      </template>
      <template v-else-if="mainTab === 'musea'">
        <MuseaPlayground :compiler />
      </template>
      <template v-else-if="mainTab === 'glyph'">
        <GlyphPlayground :compiler />
      </template>
      <template v-else>
        <AtelierPlayground :compiler />
      </template>
    </main>

    <footer class="footer">
      <span>Built with Rust + WASM</span>
      <span class="separator">|</span>
      <span
        >by
        <a href="https://github.com/ubugeeei" target="_blank" rel="noopener noreferrer"
          >@ubugeeei</a
        ></span
      >
    </footer>
  </div>
</template>
