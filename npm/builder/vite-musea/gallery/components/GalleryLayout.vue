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
const isCompact = ref(false);
let compactMedia: MediaQueryList | null = null;
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

function syncCompactLayout(event: MediaQueryListEvent | MediaQueryList) {
  isCompact.value = event.matches;
}

function resizeSidebarWithKeyboard(event: KeyboardEvent) {
  const step = event.shiftKey ? 40 : 10;
  if (event.key === "ArrowLeft") {
    sidebarWidth.size.value = Math.max(200, sidebarWidth.size.value - step);
  } else if (event.key === "ArrowRight") {
    sidebarWidth.size.value = Math.min(
      Math.max(240, window.innerWidth - 320),
      sidebarWidth.size.value + step,
    );
  } else {
    return;
  }
  event.preventDefault();
  localStorage.setItem("musea-sidebar-width", String(sidebarWidth.size.value));
}

onMounted(() => {
  load();
  document.addEventListener("keydown", handleKeydown);
  compactMedia = window.matchMedia("(max-width: 768px)");
  syncCompactLayout(compactMedia);
  compactMedia.addEventListener("change", syncCompactLayout);
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
  compactMedia?.removeEventListener("change", syncCompactLayout);
});

const handleSearchSelect = (art: { path: string }) => {
  router.push({ name: "component", params: { path: art.path } });
};

function closeSearchModal() {
  searchModalOpen.value = false;
}

function openSearchModal() {
  searchModalOpen.value = true;
}
</script>

<template>
  <div class="gallery-layout">
    <header class="header">
      <div class="header-left">
        <RouterLink to="/" class="logo">
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
        </RouterLink>
        <span v-if="!isCompact" class="header-subtitle">Component Gallery</span>
      </div>

      <div v-if="!isCompact" class="header-center">
        <button type="button" class="search-trigger" @click="openSearchModal">
          <MdiIcon class="search-icon" :path="mdiMagnify" :size="16" />
          <span>Search components...</span>
          <kbd>⌘K</kbd>
        </button>
      </div>

      <div class="header-right">
        <button
          v-if="isCompact"
          type="button"
          class="search-compact"
          aria-label="Search components"
          @click="openSearchModal"
        >
          <MdiIcon :path="mdiMagnify" :size="18" />
        </button>
        <button
          type="button"
          class="theme-toggle"
          :title="`Theme: ${themeLabel}`"
          :aria-label="`Theme: ${themeLabel}. Change theme`"
          @click="cycleTheme"
        >
          <MdiIcon :path="themeIcon" :size="18" />
        </button>
      </div>
    </header>

    <main class="main" :class="{ 'sidebar-collapsed': sidebarCollapsed }" :style="mainStyle">
      <!-- Sidebar -->
      <aside
        v-if="!isCompact"
        class="sidebar-wrapper"
        :class="{ collapsed: sidebarCollapsed }"
        :style="sidebarStyle"
      >
        <Sidebar v-show="!sidebarCollapsed" :arts="results" />
        <button
          v-if="!sidebarCollapsed"
          type="button"
          class="sidebar-resize-handle"
          aria-label="Resize sidebar with left and right arrow keys"
          @pointerdown.stop.prevent="sidebarWidth.onPointerDown"
          @keydown="resizeSidebarWithKeyboard"
        ></button>
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
        <RouterView />
      </section>
    </main>

    <!-- Search Modal -->
    <SearchModal
      :arts
      :is-open="searchModalOpen"
      @close="closeSearchModal"
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
  z-index: var(--musea-z-header);
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
  padding-inline-start: 1.5rem;
  border-inline-start: 1px solid var(--musea-border);
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

  & span {
    flex: 1;
    text-align: start;
  }

  & kbd {
    padding: 0.125rem 0.375rem;
    background: var(--musea-bg-primary);
    border: 1px solid var(--musea-border);
    border-radius: var(--musea-radius-sm);
    font-size: 0.75rem;
    font-family: var(--musea-font-mono);
  }
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

.header-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.theme-toggle,
.search-compact {
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

.theme-toggle:hover,
.search-compact:hover {
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
  border-inline-end: 1px solid var(--musea-border);
  min-width: 0;

  &.collapsed {
    overflow: hidden;

    & .sidebar-toggle {
      inset-inline-end: auto;
      inset-inline-start: 50%;
      transform: translateX(-50%);
    }
  }

  & :deep(.sidebar) {
    border-inline-end: none;
  }

  &:hover .sidebar-resize-handle::before {
    background: color-mix(in srgb, var(--musea-border) 78%, transparent);
  }
}

.sidebar-resize-handle {
  position: absolute;
  top: 0;
  inset-inline-end: -4px;
  width: 8px;
  height: 100%;
  border: 0;
  padding: 0;
  background: transparent;
  cursor: col-resize;
  z-index: var(--musea-z-sticky);
  touch-action: none;

  &::before {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    inset-inline-start: 50%;
    width: 1px;
    transform: translateX(-50%);
    background: transparent;
    transition: background-color var(--musea-transition);
  }

  &:hover::before {
    background: color-mix(in srgb, var(--musea-border) 78%, transparent);
  }
}

.sidebar-toggle {
  position: absolute;
  bottom: 0.75rem;
  inset-inline-end: 0.75rem;
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
  z-index: var(--musea-z-control);
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
</style>
