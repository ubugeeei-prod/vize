<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from "vue";
import { useRoute } from "vue-router";
import { mdiViewGrid, mdiFolder, mdiChevronUp, mdiChevronDown } from "@mdi/js";
import { useArts } from "../composables/useArts";
import { useActions } from "../composables/useActions";
import { useEventCapture } from "../composables/useEventCapture";
import MdiIcon from "../components/MdiIcon.vue";
import VariantCard from "../components/VariantCard.vue";
import VariantTabs from "../components/VariantTabs.vue";
import StatusBadge from "../components/StatusBadge.vue";
import PropsPanel from "../components/PropsPanel.vue";
import DocumentationPanel from "../components/DocumentationPanel.vue";
import A11yBadge from "../components/A11yBadge.vue";
import A11yPanel from "../components/A11yPanel.vue";
import VrtPanel from "../components/VrtPanel.vue";
import AddonToolbar from "../components/AddonToolbar.vue";
import ActionsPanel from "../components/ActionsPanel.vue";
import FullscreenPreview from "../components/FullscreenPreview.vue";
import { getVariantSectionId } from "../utils/variantSections";
import { useResizable } from "../composables/useResizable";

const route = useRoute();
const { getArt, load } = useArts();
const {
  events,
  init: initActions,
  clear: clearActions,
  setCurrentVariant: setActionsVariant,
} = useActions();
const { setCurrentVariant } = useEventCapture();

const activeTab = ref<"variants" | "props" | "docs" | "a11y" | "vrt">("variants");
const actionCount = computed(() => events.value.length);
const actionsExpanded = ref(false);
const actionsContentRef = ref<HTMLElement | null>(null);
const actionsPanel = useResizable({
  direction: "vertical",
  minSize: 88,
  maxSize: () => Math.max(160, window.innerHeight - 96),
  storageKey: "musea-actions-height",
  defaultSize: 240,
  invert: true,
  documentClass: "musea-actions-resizing",
});
const selectedVariantName = ref<string>("");
const variantSectionElements = new Map<string, HTMLElement>();
let variantObserver: IntersectionObserver | null = null;

const artPath = computed(() => route.params.path as string);
const art = computed(() => getArt(artPath.value));

// Get the currently selected variant
const selectedVariant = computed(() => {
  if (!art.value) return null;
  return (
    art.value.variants.find((v) => v.name === selectedVariantName.value) || art.value.variants[0]
  );
});

watch(
  art,
  (newArt) => {
    if (newArt) {
      const defaultVariant = newArt.variants.find((v) => v.isDefault) || newArt.variants[0];
      selectedVariantName.value = defaultVariant?.name || "";
      setCurrentVariant(selectedVariantName.value);
      setActionsVariant(selectedVariantName.value);
    }
  },
  { immediate: true },
);

watch(selectedVariantName, (name) => {
  setCurrentVariant(name);
  setActionsVariant(name);
});

const variantSectionIds = computed<Record<string, string>>(() => {
  const ids: Record<string, string> = {};
  const usedIds = new Map<string, number>();

  for (const [index, variant] of (art.value?.variants ?? []).entries()) {
    const baseId = getVariantSectionId(variant.name, index);
    const duplicateCount = usedIds.get(baseId) ?? 0;
    usedIds.set(baseId, duplicateCount + 1);
    ids[variant.name] = duplicateCount === 0 ? baseId : `${baseId}-${duplicateCount + 1}`;
  }

  return ids;
});

const actionCaptureEvents = computed(() => art.value?.metadata.actionEvents ?? []);

const actionsTrackMousemove = computed(() =>
  actionCaptureEvents.value.some((eventName) => eventName === "mousemove"),
);

const actionTrackingLabel = computed(() => {
  if (actionsTrackMousemove.value) return "Tracks mousemove";
  if (actionCaptureEvents.value.length > 0) return "Custom capture";
  return "Standard capture";
});

const actionsFooterSummary = computed(() => {
  if (actionCount.value > 0) {
    return `${actionCount.value} captured · ${actionTrackingLabel.value}`;
  }
  return actionTrackingLabel.value;
});

function syncActionsPanelHeight() {
  if (actionsContentRef.value) {
    actionsContentRef.value.style.height = `${actionsPanel.size.value}px`;
  }
}

watch(
  () => actionsPanel.size.value,
  () => {
    syncActionsPanelHeight();
  },
  { flush: "sync" },
);

watch(actionsExpanded, async (expanded) => {
  if (!expanded) return;
  await nextTick();
  syncActionsPanelHeight();
});

function disconnectVariantObserver() {
  variantObserver?.disconnect();
  variantObserver = null;
}

function setVariantSectionRef(variantName: string, el: HTMLElement | null) {
  if (el) {
    variantSectionElements.set(variantName, el);
  } else {
    variantSectionElements.delete(variantName);
  }
}

async function setupVariantObserver() {
  disconnectVariantObserver();

  if (activeTab.value !== "variants" || variantSectionElements.size === 0) {
    return;
  }

  await nextTick();

  if (typeof window === "undefined" || !("IntersectionObserver" in window)) {
    return;
  }

  variantObserver = new IntersectionObserver(
    (entries) => {
      const visibleEntries = entries
        .filter((entry) => entry.isIntersecting)
        .sort((a, b) => {
          if (Math.abs(a.intersectionRatio - b.intersectionRatio) > 0.05) {
            return b.intersectionRatio - a.intersectionRatio;
          }
          return Math.abs(a.boundingClientRect.top) - Math.abs(b.boundingClientRect.top);
        });

      const activeEntry = visibleEntries[0];
      const variantName =
        activeEntry?.target instanceof HTMLElement
          ? activeEntry.target.dataset.variantName
          : undefined;

      if (variantName && variantName !== selectedVariantName.value) {
        selectedVariantName.value = variantName;
      }
    },
    {
      rootMargin: "-18% 0px -52% 0px",
      threshold: [0.2, 0.4, 0.7],
    },
  );

  for (const element of variantSectionElements.values()) {
    variantObserver.observe(element);
  }
}

async function syncSelectionFromHash() {
  if (activeTab.value !== "variants" || !route.hash) {
    return;
  }

  await nextTick();

  const targetId = decodeURIComponent(route.hash.slice(1));
  const targetEntry = Object.entries(variantSectionIds.value).find(([, id]) => id === targetId);
  const targetEl = document.getElementById(targetId);

  if (!targetEl) {
    return;
  }

  if (targetEntry) {
    selectedVariantName.value = targetEntry[0];
  }

  targetEl.scrollIntoView({ behavior: "smooth", block: "start" });
}

onMounted(() => {
  load();
  initActions();
});

watch(artPath, () => {
  activeTab.value = "variants";
  clearActions();
  disconnectVariantObserver();
  variantSectionElements.clear();
});

const handleVariantSelect = (variantName: string) => {
  selectedVariantName.value = variantName;

  const targetId = variantSectionIds.value[variantName];
  const targetEl = targetId ? document.getElementById(targetId) : null;
  targetEl?.scrollIntoView({ behavior: "smooth", block: "start" });
};

watch(
  () => [art.value?.path, activeTab.value] as const,
  () => {
    void setupVariantObserver();
    void syncSelectionFromHash();
  },
  { immediate: true },
);

watch(
  () => route.hash,
  () => {
    void syncSelectionFromHash();
  },
);

onUnmounted(() => {
  disconnectVariantObserver();
  variantSectionElements.clear();
});
</script>

<template>
  <div v-if="art" class="component-view">
    <div class="component-header">
      <div class="component-title-row">
        <h1 class="component-title">{{ art.metadata.title }}</h1>
        <StatusBadge :status="art.metadata.status" />
      </div>
      <p v-if="art.metadata.description" class="component-description">
        {{ art.metadata.description }}
      </p>
      <div class="component-meta">
        <span class="meta-tag">
          <MdiIcon :path="mdiViewGrid" :size="12" />
          {{ art.variants.length }} variant{{ art.variants.length !== 1 ? "s" : "" }}
        </span>
        <span v-if="art.metadata.category" class="meta-tag">
          <MdiIcon :path="mdiFolder" :size="12" />
          {{ art.metadata.category }}
        </span>
        <span v-for="tag in art.metadata.tags" :key="tag" class="meta-tag"> #{{ tag }} </span>
      </div>
    </div>

    <div class="component-sticky-menu">
      <AddonToolbar />

      <div class="component-tabs">
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'variants' }"
          @click="activeTab = 'variants'"
        >
          Variants
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'props' }"
          @click="activeTab = 'props'"
        >
          Props
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'docs' }"
          @click="activeTab = 'docs'"
        >
          Docs
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'a11y' }"
          @click="activeTab = 'a11y'"
        >
          A11y
          <A11yBadge :art-path="art.path" :variant-name="selectedVariant?.name" />
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'vrt' }"
          @click="activeTab = 'vrt'"
        >
          VRT
        </button>
      </div>
    </div>

    <div class="component-content">
      <div v-if="activeTab === 'variants'" class="variants-view">
        <div class="variant-preview-area">
          <section
            v-for="(variant, index) in art.variants"
            :id="variantSectionIds[variant.name]"
            :key="variant.name"
            :ref="(el) => setVariantSectionRef(variant.name, el as HTMLElement | null)"
            class="variant-section"
            :data-variant-name="variant.name"
          >
            <div class="variant-section-header">
              <span class="variant-section-index">{{ String(index + 1).padStart(2, "0") }}</span>
              <h2 class="variant-section-title">{{ variant.name }}</h2>
              <span v-if="variant.isDefault" class="variant-section-badge">Default</span>
            </div>

            <VariantCard
              :art-path="art.path"
              :variant="variant"
              :component-name="art.metadata.title"
            />
          </section>
        </div>

        <aside class="variant-toc-column" aria-label="Variants navigation">
          <VariantTabs
            :variants="art.variants"
            :selected-variant="selectedVariantName"
            :section-ids="variantSectionIds"
            @select="handleVariantSelect"
          />
        </aside>
      </div>

      <PropsPanel
        v-if="activeTab === 'props'"
        :art-path="art.path"
        :default-variant-name="art.variants.find((v) => v.isDefault)?.name || art.variants[0]?.name"
      />

      <DocumentationPanel v-if="activeTab === 'docs'" :art-path="art.path" />

      <A11yPanel
        v-if="activeTab === 'a11y'"
        :art-path="art.path"
        :default-variant-name="selectedVariant?.name"
      />

      <VrtPanel
        v-if="activeTab === 'vrt'"
        :art-path="art.path"
        :default-variant-name="selectedVariant?.name"
      />
    </div>

    <!-- Actions Footer Panel (sticky bottom) -->
    <div class="actions-footer" :class="{ expanded: actionsExpanded }">
      <div class="actions-footer-shell">
        <div
          v-if="actionsExpanded"
          class="actions-footer-resizer"
          title="Resize Actions panel"
          @pointerdown.stop.prevent="actionsPanel.onPointerDown"
        />
        <button
          type="button"
          class="actions-footer-toggle"
          @click="actionsExpanded = !actionsExpanded"
        >
          <span class="actions-footer-toggle-copy">
            <span class="actions-footer-toggle-line">
              <span class="actions-footer-title">Actions</span>
              <span v-if="actionCount > 0" class="action-count-badge">{{
                actionCount > 99 ? "99+" : actionCount
              }}</span>
            </span>
            <span class="actions-footer-caption">{{ actionsFooterSummary }}</span>
          </span>

          <MdiIcon
            class="actions-footer-chevron"
            :path="actionsExpanded ? mdiChevronDown : mdiChevronUp"
            :size="14"
          />
        </button>

        <div v-if="actionsExpanded" ref="actionsContentRef" class="actions-footer-content">
          <ActionsPanel :capture-events="actionCaptureEvents" />
        </div>
      </div>
    </div>

    <FullscreenPreview />
  </div>

  <div v-else class="component-not-found">
    <h2>Component not found</h2>
    <p>The requested component could not be found.</p>
    <router-link to="/" class="back-link">Back to home</router-link>
  </div>
</template>

<style scoped>
.component-view {
  max-width: 1400px;
  margin: 0 auto;
  padding: 2rem;
}

.component-header {
  margin-bottom: 1.5rem;
}

.component-title-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.5rem;
}

.component-title {
  font-size: 1.5rem;
  font-weight: 700;
}

.component-description {
  color: var(--musea-text-muted);
  font-size: 0.9375rem;
  max-width: 600px;
  margin-bottom: 0.75rem;
}

.component-meta {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.meta-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.25rem 0.625rem;
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  font-size: 0.75rem;
  color: var(--musea-text-muted);
}

.meta-tag svg {
  width: 12px;
  height: 12px;
}

.component-sticky-menu {
  position: sticky;
  top: 0;
  z-index: 20;
  margin-bottom: 1.5rem;
  padding-bottom: 0.875rem;
  background: linear-gradient(
    to bottom,
    var(--musea-bg-primary) 0%,
    var(--musea-bg-primary) calc(100% - 0.5rem),
    transparent 100%
  );
}

.component-view :deep(.addon-toolbar) {
  margin-bottom: 0.75rem;
}

.component-tabs {
  display: flex;
  gap: 0.25rem;
  border-bottom: 1px solid var(--musea-border);
  background: color-mix(in srgb, var(--musea-bg-primary) 92%, transparent);
  backdrop-filter: blur(10px);
}

.component-content {
  min-height: 0;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: none;
  border: none;
  color: var(--musea-text-muted);
  font-size: 0.875rem;
  font-weight: 500;
  padding: 0.75rem 1rem;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all var(--musea-transition);
}

.tab-btn:hover {
  color: var(--musea-text);
}

.tab-btn.active {
  color: var(--musea-accent);
  border-bottom-color: var(--musea-accent);
}

.action-count-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 0.375rem;
  border-radius: 9px;
  background: var(--musea-accent);
  color: #fff;
  font-size: 0.625rem;
  font-weight: 700;
  line-height: 1;
}

.variants-view {
  display: grid;
  grid-template-columns: minmax(0, 1fr) clamp(220px, 24vw, 280px);
  gap: 2rem;
  align-items: start;
}

.variant-preview-area {
  min-height: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.variant-toc-column {
  min-width: 0;
  position: sticky;
  top: calc(var(--musea-header-height) + 1rem);
  align-self: start;
}

.variant-section {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  scroll-margin-top: calc(var(--musea-header-height) + 1.25rem);
}

.variant-section-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding-inline: 0.25rem;
}

.variant-section-index {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2rem;
  padding: 0.25rem 0.5rem;
  border-radius: 999px;
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  color: var(--musea-text-muted);
  font-size: 0.75rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.variant-section-title {
  font-size: 1rem;
  font-weight: 700;
}

.variant-section-badge {
  font-size: 0.6875rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  padding: 0.25rem 0.5rem;
  border-radius: 999px;
  background: var(--musea-accent-subtle);
  color: var(--musea-accent);
}

.actions-footer {
  position: sticky;
  bottom: 0;
  margin: 0 -2rem -2rem;
  z-index: 30;
}

.actions-footer-shell {
  border-top: 1px solid var(--musea-border-subtle);
  background: color-mix(in srgb, var(--musea-bg-primary) 94%, transparent);
  backdrop-filter: blur(14px);
}

.actions-footer.expanded .actions-footer-shell {
  box-shadow: 0 -12px 28px rgba(18, 18, 18, 0.08);
}

.actions-footer-resizer {
  height: 1.375rem;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: ns-resize;
  touch-action: none;
}

.actions-footer-resizer::before {
  content: "";
  width: 2.75rem;
  height: 2px;
  border-radius: 999px;
  background: var(--musea-border);
}

.actions-footer-toggle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  width: 100%;
  padding: 0.75rem 1rem;
  background: transparent;
  border: none;
  color: var(--musea-text-muted);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition:
    background var(--musea-transition),
    color var(--musea-transition);
}

.actions-footer-toggle:hover {
  background: color-mix(in srgb, var(--musea-bg-secondary) 78%, transparent);
  color: var(--musea-text);
}

.actions-footer.expanded .actions-footer-toggle {
  border-bottom: 1px solid var(--musea-border-subtle);
}

.actions-footer-toggle-copy {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.1875rem;
  min-width: 0;
}

.actions-footer-toggle-line {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  min-width: 0;
}

.actions-footer-title {
  color: var(--musea-text);
  font-size: 0.8125rem;
  font-weight: 600;
}

.actions-footer-caption {
  color: var(--musea-text-muted);
  font-size: 0.6875rem;
}

.actions-footer-chevron {
  color: var(--musea-text-muted);
  transition: transform var(--musea-transition);
  flex-shrink: 0;
}

.actions-footer.expanded .actions-footer-chevron {
  transform: rotate(180deg);
}

.actions-footer-content {
  overflow: hidden;
}

.component-not-found {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  text-align: center;
  color: var(--musea-text-muted);
}

.component-not-found h2 {
  color: var(--musea-text);
  margin-bottom: 0.5rem;
}

.back-link {
  margin-top: 1rem;
  color: var(--musea-accent);
  text-decoration: underline;
}

@media (max-width: 960px) {
  .component-view {
    padding: 1.25rem;
  }

  .component-sticky-menu {
    padding-bottom: 0.75rem;
  }

  .variants-view {
    grid-template-columns: 1fr;
  }

  .variant-toc-column {
    order: -1;
    position: static;
  }

  .actions-footer {
    margin: 0 -1.25rem -1.25rem;
  }

  .actions-footer-toggle {
    padding-inline: 0.875rem;
  }
}
</style>
