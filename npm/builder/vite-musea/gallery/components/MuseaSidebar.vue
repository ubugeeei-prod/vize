<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { RouterLink, useRoute } from "vue-router";
import { mdiHome, mdiPalette, mdiCheckCircleOutline, mdiChevronRight } from "@mdi/js";
import type { ArtFileInfo } from "../../src/types/index.js";
import MdiIcon from "./MdiIcon.vue";
import { useGalleryMetadata } from "../composables/useGalleryMetadata";
import type { MuseaExternalArtStatusName } from "../../src/types/index.js";

const props = defineProps<{
  arts: ArtFileInfo[];
}>();

const route = useRoute();
const { config } = useGalleryMetadata();
type TagFilter = "include" | "exclude" | "neutral";
const tagFilters = ref<Record<string, TagFilter>>({});
const statusFilters = ref<Set<MuseaExternalArtStatusName>>(new Set());
const externalStatusNames: MuseaExternalArtStatusName[] = [
  "success",
  "warning",
  "error",
  "pending",
];

watch(
  () => config.value.tags,
  (tags) => {
    for (const [tag, presentation] of Object.entries(tags ?? {})) {
      tagFilters.value[tag] ??= presentation.defaultFilter ?? "neutral";
    }
  },
  { immediate: true },
);

const configuredTags = computed(() => Object.entries(config.value.tags ?? {}));
const filteredArts = computed(() =>
  props.arts.filter((art) => {
    for (const [tag, filter] of Object.entries(tagFilters.value)) {
      if (filter === "include" && !art.metadata.tags.includes(tag)) return false;
      if (filter === "exclude" && art.metadata.tags.includes(tag)) return false;
    }
    return (
      statusFilters.value.size === 0 ||
      (art.metadata.externalStatus && statusFilters.value.has(art.metadata.externalStatus.status))
    );
  }),
);

// Track expanded categories
const expandedCategories = ref<Set<string>>(new Set());

const categoryList = computed(() => {
  const map = new Map<string, ArtFileInfo[]>();
  for (const art of filteredArts.value) {
    const cat = art.metadata.category || "Components";
    if (!map.has(cat)) map.set(cat, []);
    map.get(cat)!.push(art);
  }
  // Auto-expand all categories initially
  for (const cat of map.keys()) {
    expandedCategories.value.add(cat);
  }
  return Array.from(map.entries());
});

const selectedPath = computed(() => route.params.path as string | undefined);

function toggleCategory(category: string) {
  if (expandedCategories.value.has(category)) {
    expandedCategories.value.delete(category);
  } else {
    expandedCategories.value.add(category);
  }
}

function isCategoryExpanded(category: string) {
  return expandedCategories.value.has(category);
}

function cycleTagFilter(tag: string) {
  const current = tagFilters.value[tag] ?? "neutral";
  tagFilters.value[tag] =
    current === "neutral" ? "include" : current === "include" ? "exclude" : "neutral";
}

function toggleStatusFilter(status: MuseaExternalArtStatusName) {
  const next = new Set(statusFilters.value);
  if (next.has(status)) next.delete(status);
  else next.add(status);
  statusFilters.value = next;
}
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-section">
      <RouterLink
        :to="{ name: 'home' }"
        class="sidebar-home-link"
        :class="{ active: route.name === 'home' }"
      >
        <MdiIcon :path="mdiHome" :size="16" />
        Home
      </RouterLink>

      <RouterLink
        :to="{ name: 'tokens' }"
        class="sidebar-home-link"
        :class="{ active: route.name === 'tokens' }"
      >
        <MdiIcon :path="mdiPalette" :size="16" />
        Design Tokens
      </RouterLink>

      <RouterLink
        :to="{ name: 'tests' }"
        class="sidebar-home-link"
        :class="{ active: route.name === 'tests' }"
      >
        <MdiIcon :path="mdiCheckCircleOutline" :size="16" />
        Test Summary
      </RouterLink>
    </div>

    <div v-if="configuredTags.length > 0" class="sidebar-section filter-section">
      <span class="filter-heading">Tags</span>
      <button
        v-for="[tag, presentation] in configuredTags"
        :key="tag"
        type="button"
        class="filter-chip"
        :class="`filter-${tagFilters[tag] ?? 'neutral'}`"
        :aria-label="`${presentation.label ?? tag} tag filter: ${tagFilters[tag] ?? 'neutral'}`"
        :title="`Filter ${tag}: ${tagFilters[tag] ?? 'neutral'}`"
        @click="cycleTagFilter(tag)"
      >
        {{ presentation.label ?? tag }}
      </button>
    </div>

    <div
      v-if="arts.some((art) => art.metadata.externalStatus)"
      class="sidebar-section filter-section"
    >
      <span class="filter-heading">Status</span>
      <button
        v-for="status in externalStatusNames"
        :key="status"
        type="button"
        class="filter-chip status-filter"
        :class="[`external-${status}`, { active: statusFilters.has(status) }]"
        :aria-pressed="statusFilters.has(status)"
        @click="toggleStatusFilter(status)"
      >
        {{ status }}
      </button>
    </div>

    <div v-for="[category, items] in categoryList" :key="category" class="sidebar-section">
      <button
        type="button"
        class="category-header"
        :class="{ 'category-header--expanded': isCategoryExpanded(category) }"
        :aria-expanded="isCategoryExpanded(category)"
        @click="() => toggleCategory(category)"
      >
        <MdiIcon class="category-icon" :path="mdiChevronRight" :size="16" />
        <span class="category-label">{{ category }}</span>
        <span class="category-count">{{ items.length }}</span>
      </button>
      <ul v-show="isCategoryExpanded(category)" class="art-list">
        <li v-for="art in items" :key="art.path">
          <RouterLink
            :to="{ name: 'component', params: { path: art.path } }"
            class="art-item"
            :class="{ active: selectedPath === art.path }"
          >
            <span class="art-name">{{ art.metadata.title }}</span>
            <span
              v-if="art.metadata.externalStatus"
              class="external-status"
              :class="`external-${art.metadata.externalStatus.status}`"
              role="img"
              :aria-label="art.metadata.externalStatus.label ?? art.metadata.externalStatus.status"
              :title="art.metadata.externalStatus.label ?? art.metadata.externalStatus.status"
            ></span>
            <span class="art-variant-count">{{ art.variants.length }}</span>
          </RouterLink>
        </li>
      </ul>
    </div>

    <div v-if="filteredArts.length === 0" class="sidebar-empty">No components found</div>
  </aside>
</template>

<style scoped>
.sidebar {
  background: var(--musea-bg-secondary);
  overflow-y: auto;
  overflow-x: hidden;
}

.sidebar-section {
  padding: 0.5rem 0.75rem;
}

.filter-section {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.filter-heading {
  width: 100%;
  color: var(--musea-text-muted);
  font-size: 0.625rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.filter-chip {
  padding: 0.1875rem 0.375rem;
  border: 1px solid var(--musea-border);
  border-radius: 4px;
  color: var(--musea-text-muted);
  background: transparent;
  font: inherit;
  font-size: 0.6875rem;
  cursor: pointer;
}

.filter-include,
.status-filter.active {
  border-color: var(--musea-accent);
  color: var(--musea-accent-hover);
  background: var(--musea-accent-subtle);
}

.filter-exclude {
  border-color: var(--musea-error);
  color: var(--musea-error);
  text-decoration: line-through;
}

.external-status {
  width: 7px;
  height: 7px;
  flex: 0 0 7px;
  border-radius: 50%;
  background: currentcolor;
}

.external-success {
  color: var(--musea-success);
}

.external-warning,
.external-pending {
  color: var(--musea-warning);
}

.external-error {
  color: var(--musea-error);
}

.sidebar-section {
  & + & {
    padding-top: 0;
  }
}

.sidebar-home-link {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.5rem 0.75rem;
  border-radius: var(--musea-radius-sm);
  font-size: 0.8125rem;
  color: var(--musea-text-secondary);
  cursor: pointer;
  transition: all var(--musea-transition);
  text-decoration: none;
}

.sidebar-home-link:hover {
  background: var(--musea-bg-tertiary);
  color: var(--musea-text);
}

.sidebar-home-link.active {
  background: var(--musea-accent-subtle);
  color: var(--musea-accent-hover);
}

.category-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 0.75rem;
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--musea-text-muted);
  cursor: pointer;
  user-select: none;
  border-radius: var(--musea-radius-sm);
  transition: background var(--musea-transition);
  width: 100%;
  border: 0;
  background: transparent;
  font-family: inherit;
  text-align: start;
}

.category-header:hover {
  background: var(--musea-bg-tertiary);
}

.category-icon {
  width: 16px;
  height: 16px;
  transition: transform var(--musea-transition);
  flex-shrink: 0;
}

.category-header--expanded {
  .category-icon {
    transform: rotate(90deg);
  }
}

.category-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.category-count {
  margin-inline-start: auto;
  background: var(--musea-bg-tertiary);
  padding: 0.125rem 0.375rem;
  border-radius: 4px;
  font-size: 0.625rem;
}

.art-list {
  list-style: none;
  margin: 0;
  margin-top: 0.25rem;
  padding: 0 0 0 1rem;
}

.art-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.375rem 0.75rem 0.375rem 1.75rem;
  border-radius: var(--musea-radius-sm);
  cursor: pointer;
  font-size: 0.8125rem;
  color: var(--musea-text-secondary);
  transition: all var(--musea-transition);
  position: relative;
  text-decoration: none;
}

.art-item::before {
  content: "";
  position: absolute;
  inset-inline-start: 0.75rem;
  top: 50%;
  transform: translateY(-50%);
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--musea-border);
  transition: background var(--musea-transition);
}

.art-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.art-item:hover {
  background: var(--musea-bg-tertiary);
  color: var(--musea-text);
}

.art-item:hover::before {
  background: var(--musea-text-muted);
}

.art-item.active {
  background: var(--musea-accent-subtle);
  color: var(--musea-accent-hover);
}

.art-item.active::before {
  background: var(--musea-accent);
}

.art-variant-count {
  margin-inline-start: auto;
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
  opacity: 0;
  transition: opacity var(--musea-transition);
}

.art-item:hover {
  .art-variant-count {
    opacity: 1;
  }
}

.sidebar-empty {
  padding: 2rem 1rem;
  text-align: center;
  color: var(--musea-text-muted);
  font-size: 0.8125rem;
}
</style>
