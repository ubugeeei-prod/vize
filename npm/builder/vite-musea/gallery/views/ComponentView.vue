<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick, useTemplateRef } from "vue";
import { RouterLink, useRoute } from "vue-router";
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
import TestPanel from "../components/TestPanel.vue";
import AddonToolbar from "../components/AddonToolbar.vue";
import ActionsPanel from "../components/ActionsPanel.vue";
import FullscreenPreview from "../components/FullscreenPreview.vue";
import { getVariantSectionId } from "../utils/variantSections";
import { useResizable } from "../composables/useResizable";
import { createVariantNavigation } from "../utils/variantNavigation";
import { MUSEA_PROPS_QUERY } from "../utils/propsPermalink";

const route = useRoute();
const { getArt, load } = useArts();
const {
  events,
  init: initActions,
  clear: clearActions,
  setCurrentVariant: setActionsVariant,
} = useActions();
const { setCurrentVariant } = useEventCapture();

const activeTab = ref<"variants" | "props" | "test" | "docs" | "a11y" | "vrt">(
  route.query[MUSEA_PROPS_QUERY] === undefined ? "variants" : "props",
);
const actionCount = computed(() => events.value.length);
const actionsExpanded = ref(false);
const actionsContentRef = useTemplateRef<HTMLElement>("actionsContentRef");
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
const variantSectionElements = new Map<string, Element>();

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
  const ids: Record<string, string> = Object.create(null);
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

const {
  disconnectVariantObserver,
  setupVariantObserver,
  syncSelectionFromHash,
  handleVariantSelect,
} = createVariantNavigation({
  activeTab,
  route,
  artPath,
  selectedVariantName,
  variantSectionIds,
  variantSectionElements,
  nextTick,
});

function setVariantSectionRef(variantName: string, el: unknown) {
  if (typeof Element !== "undefined" && el instanceof Element) {
    variantSectionElements.set(variantName, el);
  } else {
    variantSectionElements.delete(variantName);
  }
}

onMounted(() => {
  // The shared event store outlives this view when navigation passes through Home.
  clearActions();
  load();
  initActions();
});

watch(artPath, () => {
  activeTab.value = route.query[MUSEA_PROPS_QUERY] === undefined ? "variants" : "props";
  clearActions();
  disconnectVariantObserver();
  variantSectionElements.clear();
});

watch(
  () => route.query[MUSEA_PROPS_QUERY],
  (shared) => {
    if (shared !== undefined) activeTab.value = "props";
  },
);

function selectTab(tab: typeof activeTab.value) {
  activeTab.value = tab;
}

function toggleActions() {
  actionsExpanded.value = !actionsExpanded.value;
}

watch(
  () => [art.value?.path, activeTab.value] as const,
  () => {
    if (activeTab.value === "variants" && !route.hash) {
      handleVariantSelect(selectedVariantName.value);
    } else {
      void setupVariantObserver();
      void syncSelectionFromHash();
    }
  },
  { immediate: true, flush: "post" },
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
          :class="{ active: activeTab === 'test' }"
          :aria-pressed="activeTab === 'test'"
          @click="() => selectTab('test')"
        >
          Test
          <span v-if="art.variants.some((variant) => variant.play)" class="tab-count">
            {{ art.variants.filter((variant) => variant.play).length }}
          </span>
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'variants' }"
          :aria-pressed="activeTab === 'variants'"
          @click="() => selectTab('variants')"
        >
          Variants
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'props' }"
          :aria-pressed="activeTab === 'props'"
          @click="() => selectTab('props')"
        >
          Props
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'docs' }"
          :aria-pressed="activeTab === 'docs'"
          @click="() => selectTab('docs')"
        >
          Docs
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'a11y' }"
          :aria-pressed="activeTab === 'a11y'"
          @click="() => selectTab('a11y')"
        >
          A11y
          <A11yBadge :art-path="art.path" :variant-name="selectedVariant?.name" />
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'vrt' }"
          :aria-pressed="activeTab === 'vrt'"
          @click="() => selectTab('vrt')"
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
            :ref="(el) => setVariantSectionRef(variant.name, el)"
            class="variant-section"
            :data-variant-name="variant.name"
          >
            <div class="variant-section-header">
              <span class="variant-section-index">{{ String(index + 1).padStart(2, "0") }}</span>
              <h2 class="variant-section-title">{{ variant.name }}</h2>
              <span v-if="variant.isDefault" class="variant-section-badge">Default</span>
            </div>

            <VariantCard :art-path="art.path" :variant />
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
        :variants="art.variants"
        :selected-variant-name="selectedVariant?.name"
        @select-variant="(name) => (selectedVariantName = name)"
      />

      <TestPanel
        v-if="activeTab === 'test' && selectedVariant"
        :art-path="art.path"
        :variant="selectedVariant"
        :variants="art.variants"
        @select-variant="(name) => (selectedVariantName = name)"
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
          :aria-expanded="actionsExpanded"
          @click="toggleActions"
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
    <RouterLink to="/" class="back-link">Back to home</RouterLink>
  </div>
</template>

<style scoped src="./ComponentView.css"></style>
