<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import {
  mdiMagnify,
  mdiWeatherSunny,
  mdiWeatherNight,
  mdiThemeLightDark,
  mdiChevronLeft,
  mdiChevronRight,
} from "@mdi/js";
import { useArts } from "../composables/useArts";
import { useSearch } from "../composables/useSearch";
import { useTheme } from "../composables/useTheme";
import SearchBar from "./SearchBar.vue";
import Sidebar from "./Sidebar.vue";
import SearchModal from "./SearchModal.vue";
import MdiIcon from "./MdiIcon.vue";
import { useResizable } from "../composables/useResizable";

const router = useRouter();
const { arts, load } = useArts();
const { query, results } = useSearch(arts);
const { currentTheme, cycleTheme } = useTheme();

const searchModalOpen = ref(false);
const sidebarCollapsed = ref(false);
const sidebarWidth = useResizable({
  direction: "horizontal",
  minSize: 200,
  maxSize: () => Math.max(240, window.innerWidth - 320),
  storageKey: "musea-sidebar-width",
  defaultSize: 240,
  documentClass: "musea-sidebar-resizing",
});

const mainStyle = computed(() => ({
  "--musea-sidebar-width": `${sidebarWidth.size.value}px`,
}));

const sidebarStyle = computed(() => ({
  width: sidebarCollapsed.value ? "40px" : `${sidebarWidth.size.value}px`,
}));

function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value;
}

const themeIcon = computed(() => {
  switch (currentTheme.value) {
    case "dark":
      return mdiWeatherNight;
    case "system":
      return mdiThemeLightDark;
    default:
      return mdiWeatherSunny;
  }
});

const themeLabel = computed(() => {
  switch (currentTheme.value) {
    case "dark":
      return "Dark";
    case "system":
      return "System";
    default:
      return "Light";
  }
});

// Global keyboard shortcuts
const handleKeydown = (e: KeyboardEvent) => {
  if ((e.metaKey || e.ctrlKey) && e.key === "k") {
    e.preventDefault();
    searchModalOpen.value = !searchModalOpen.value;
  }
  if ((e.metaKey || e.ctrlKey) && e.key === "b") {
    e.preventDefault();
    toggleSidebar();
  }
};

onMounted(() => {
  load();
  document.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
});

const handleSearchSelect = (art: { path: string }, variantName?: string) => {
  router.push({ name: "component", params: { path: art.path } });
};
</script>

<template>
  <div class="gallery-layout">
    <header class="header">
      <div class="header-left">
        <router-link to="/" class="logo">
          <svg class="logo-svg" viewBox="232 24 300 210" fill="none" aria-hidden="true">
            <g transform="translate(180, 50)">
              <g transform="translate(180, 80) skewX(-20)">
                <rect x="0" y="0" width="150" height="4" rx="2" fill="currentColor" />
                <rect x="20" y="25" width="100" height="3" rx="1.5" fill="currentColor" />
                <rect x="10" y="-25" width="80" height="2" rx="1" fill="currentColor" />
              </g>
              <g transform="skewX(-15)">
                <path d="M 200 0 L 120 180 L 210 60 L 200 0 Z" fill="currentColor" />
                <path d="M 60 0 L 120 180 L 160 40 L 60 0 Z" fill="currentColor" />
              </g>
            </g>
          </svg>
          Musea
        </router-link>
        <span class="header-subtitle">Component Gallery</span>
      </div>

      <div class="header-center">
        <button type="button" class="search-trigger" @click="searchModalOpen = true">
          <MdiIcon class="search-icon" :path="mdiMagnify" :size="16" />
          <span>Search components...</span>
          <kbd>⌘K</kbd>
        </button>
      </div>

      <div class="header-right">
        <button
          type="button"
          class="theme-toggle"
          :title="`Theme: ${themeLabel}`"
          @click="cycleTheme"
        >
          <MdiIcon :path="themeIcon" :size="18" />
        </button>
      </div>
    </header>

    <main class="main" :class="{ 'sidebar-collapsed': sidebarCollapsed }" :style="mainStyle">
      <!-- Sidebar -->
      <aside class="sidebar-wrapper" :class="{ collapsed: sidebarCollapsed }" :style="sidebarStyle">
        <Sidebar v-show="!sidebarCollapsed" :arts="results" />
        <div
          v-if="!sidebarCollapsed"
          class="sidebar-resize-handle"
          title="Resize sidebar"
          @pointerdown.stop.prevent="sidebarWidth.onPointerDown"
        />
        <button
          type="button"
          class="sidebar-toggle"
          :title="sidebarCollapsed ? 'Expand sidebar (⌘B)' : 'Collapse sidebar (⌘B)'"
          @click="toggleSidebar"
        >
          <MdiIcon :path="sidebarCollapsed ? mdiChevronRight : mdiChevronLeft" :size="16" />
        </button>
      </aside>

      <!-- Main Content -->
      <section class="content">
        <router-view />
      </section>
    </main>

    <!-- Search Modal -->
    <SearchModal
      :arts="arts"
      :is-open="searchModalOpen"
      @close="searchModalOpen = false"
      @select="handleSearchSelect"
    />
  </div>
</template>

<style scoped>
.gallery-layout {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.header {
  background: var(--musea-bg-secondary);
  border-bottom: 1px solid var(--musea-border);
  padding: 0 1.5rem;
  height: var(--musea-header-height);
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: sticky;
  top: 0;
  z-index: 100;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 1.5rem;
}

.header-center {
  flex: 1;
  display: flex;
  justify-content: center;
  max-width: 400px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1.125rem;
  font-weight: 700;
  color: var(--musea-accent);
  text-decoration: none;
}

.logo-svg {
  width: 42px;
  height: 29px;
  flex-shrink: 0;
}

.header-subtitle {
  color: var(--musea-text-muted);
  font-size: 0.8125rem;
  font-weight: 500;
  padding-left: 1.5rem;
  border-left: 1px solid var(--musea-border);
}

.search-trigger {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  background: var(--musea-bg-tertiary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  color: var(--musea-text-muted);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all var(--musea-transition);
}

.search-trigger:hover {
  border-color: var(--musea-accent);
  color: var(--musea-text-secondary);
}

.search-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.search-trigger span {
  flex: 1;
  text-align: left;
}

.search-trigger kbd {
  padding: 0.125rem 0.375rem;
  background: var(--musea-bg-primary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  font-size: 0.75rem;
  font-family: var(--musea-font-mono);
}

.header-right {
  display: flex;
  align-items: center;
}

.theme-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: var(--musea-bg-tertiary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  color: var(--musea-text-muted);
  cursor: pointer;
  transition: all var(--musea-transition);
}

.theme-toggle:hover {
  border-color: var(--musea-accent);
  color: var(--musea-text);
}

.main {
  display: flex;
  flex: 1;
  overflow: hidden;
  height: calc(100vh - var(--musea-header-height));
  min-height: 0;
}

.sidebar-wrapper {
  flex: 0 0 auto;
  height: 100%;
  max-height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  display: flex;
  flex-direction: column;
  position: relative;
  background: var(--musea-bg-secondary);
  border-right: 1px solid var(--musea-border);
  min-width: 0;
}

.sidebar-wrapper.collapsed {
  overflow: hidden;
}

.sidebar-wrapper :deep(.sidebar) {
  border-right: none;
}

.sidebar-resize-handle {
  position: absolute;
  top: 0;
  right: -4px;
  width: 8px;
  height: 100%;
  cursor: col-resize;
  z-index: 20;
  touch-action: none;
}

.sidebar-resize-handle::before {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  width: 1px;
  transform: translateX(-50%);
  background: transparent;
  transition: background-color var(--musea-transition);
}

.sidebar-wrapper:hover .sidebar-resize-handle::before,
.sidebar-resize-handle:hover::before {
  background: color-mix(in srgb, var(--musea-border) 78%, transparent);
}

.sidebar-toggle {
  position: absolute;
  bottom: 0.75rem;
  right: 0.75rem;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--musea-bg-tertiary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  color: var(--musea-text-muted);
  cursor: pointer;
  transition: all var(--musea-transition);
  z-index: 10;
}

.sidebar-wrapper.collapsed .sidebar-toggle {
  right: auto;
  left: 50%;
  transform: translateX(-50%);
}

.sidebar-toggle:hover {
  background: var(--musea-bg-elevated);
  color: var(--musea-text);
  border-color: var(--musea-text-muted);
}

.content {
  background: var(--musea-bg-primary);
  overflow-y: auto;
  height: calc(100vh - var(--musea-header-height));
  min-width: 0;
  flex: 1 1 auto;
}

@media (max-width: 768px) {
  .main {
    grid-template-columns: 1fr !important;
  }
  .sidebar-wrapper {
    display: none;
  }
  .header-subtitle {
    display: none;
  }
  .header-center {
    display: none;
  }
}
</style>
