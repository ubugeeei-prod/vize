<script setup lang="ts">
import {
  ref,
  watch,
  computed,
  onMounted,
  onUnmounted,
  defineAsyncComponent,
  useTemplateRef,
} from "vue";
import { shallowRef } from "vue";
import { useRoute, useRouter } from "vue-router";
import { usePalette } from "../composables/usePalette";
import { useArts } from "../composables/useArts";
import { getPreviewUrl, vMuseaGlobals } from "../api";
import { ownRecordValue, sendMessage, snapshotMessageRecord } from "../composables/usePostMessage";
import { indentUsage, usagePropsAttributes, usageScript } from "../utils/usageCode";
import { safeUrl } from "../utils/safeUrl";
import { getControlComponent } from "./controlComponent";
import { supportedProps, unsupportedPropEdit } from "../utils/supportedProps";
import {
  MUSEA_PROPS_QUERY,
  createPropsPermalink,
  readPropsPermalink,
} from "../utils/propsPermalink";
import SlotEditor from "./SlotEditor.vue";
import HighlightedCode from "./HighlightedCode.vue";
import type { ArtVariant } from "../../src/types/index.js";

const props = defineProps<{
  artPath: string;
  variants: ArtVariant[];
  selectedVariantName?: string;
}>();
const emit = defineEmits<{ (event: "selectVariant", name: string): void }>();

const MonacoEditor = defineAsyncComponent(() => import("./MonacoEditor.vue"));
const route = useRoute();
const router = useRouter();
const shareStatus = ref("");
const shareError = ref<string | null>(null);

const {
  palette,
  loading,
  error,
  values,
  customProps,
  deletedPaletteProps,
  allControls,
  mergedValues,
  customPropNames,
  load,
  setValue,
  setAllValues,
  addProp,
  removeProp,
  resetValues,
  saveValues,
  clearSavedValues,
} = usePalette();
const { getArt } = useArts();

const iframeRef = useTemplateRef<HTMLIFrameElement>("iframeRef");
const iframeReady = ref(false);
const slotContent = shallowRef<Record<string, string>>({});
const copiedUsage = ref(false);
const art = computed(() => getArt(props.artPath));
const unsupported = computed(() => new Set(palette.value?.unsupportedProps ?? []));
const functionProps = computed(
  () =>
    new Set(
      (palette.value?.controls ?? [])
        .filter((control) => control.control === "function")
        .map((control) => control.name),
    ),
);
const jsonUnsupported = computed(() => new Set([...unsupported.value, ...functionProps.value]));
const appliedValues = computed(() => supportedProps(mergedValues.value, jsonUnsupported.value));
const retainedUnsupported = computed(() =>
  Object.keys(mergedValues.value).filter((name) => unsupported.value.has(name)),
);
const controlsMode = ref<"controls" | "code">("controls");
const saveStatus = ref<"idle" | "saved">("idle");
const selectedVariant = computed(
  () =>
    props.variants.find((variant) => variant.name === props.selectedVariantName) ??
    props.variants.find((variant) => variant.isDefault) ??
    props.variants[0],
);
const variantValues = computed<Record<string, unknown>>(() => {
  const authored = selectedVariant.value?.props ?? selectedVariant.value?.args ?? {};
  return Object.fromEntries(
    Object.keys(authored)
      .filter((name) => name !== "viewport")
      .map((name) => [name, ownRecordValue(authored, name)]),
  );
});
const variantAttrs = computed<Record<string, unknown>>(() => selectedVariant.value?.attrs ?? {});

// Code mode state
const codeEditorContent = ref("");
const codeError = ref<string | null>(null);

// Add prop form state
const showAddForm = ref(false);
const newPropName = ref("");
const newPropControl = ref("text");
const newPropDefault = ref("");
const newPropError = ref<string | null>(null);

const previewUrl = computed(() => {
  if (!selectedVariant.value) return "";
  return getPreviewUrl(props.artPath, selectedVariant.value.name);
});

function sendPreviewProps(iframe: HTMLIFrameElement, values: Record<string, unknown>): void {
  sendMessage(iframe, "musea:set-props", {
    props: snapshotMessageRecord(values),
    attrs: snapshotMessageRecord(variantAttrs.value),
  });
}

watch(
  [() => props.artPath, () => selectedVariant.value?.name, () => route.query[MUSEA_PROPS_QUERY]],
  ([path, _variant, shared]) => {
    const parsed = shared === undefined ? null : readPropsPermalink(shared, path);
    shareError.value = parsed?.error ?? null;
    shareStatus.value = "";
    if (path) void load(path, parsed?.state?.palette, variantValues.value);
    iframeReady.value = false;
    slotContent.value = parsed?.state?.slots ?? {};
    controlsMode.value = "controls";
  },
  { immediate: true },
);

// Send only supported props; retain unsupported raw values in the editor.
watch(
  appliedValues,
  (newValues) => {
    const iframe = iframeRef.value;
    if (!iframe || !iframeReady.value) return;
    sendPreviewProps(iframe, newValues);
  },
  { deep: true },
);

watch(
  [values, customProps, deletedPaletteProps],
  () => {
    saveStatus.value = "idle";
  },
  { deep: true },
);

// Send slots to iframe when slot content changes
watch(
  slotContent,
  (content) => {
    const iframe = iframeRef.value;
    if (!iframe || !iframeReady.value) return;
    sendMessage(iframe, "musea:set-slots", { slots: content });
  },
  { deep: true },
);

// Sync code editor content when switching to code mode or when values change in controls mode
watch(controlsMode, (mode) => {
  if (mode === "code") {
    codeEditorContent.value = JSON.stringify(mergedValues.value, null, 2);
    codeError.value = null;
  }
});

watch(
  mergedValues,
  (newValues) => {
    if (controlsMode.value === "code") {
      const formatted = JSON.stringify(newValues, null, 2);
      if (codeEditorContent.value !== formatted) {
        codeEditorContent.value = formatted;
      }
    }
  },
  { deep: true },
);

function onCodeEditorUpdate(newContent: string) {
  codeEditorContent.value = newContent;
  try {
    const parsed = JSON.parse(newContent);
    if (typeof parsed === "object" && parsed !== null && !Array.isArray(parsed)) {
      const functionEdit = unsupportedPropEdit(mergedValues.value, parsed, functionProps.value);
      codeError.value =
        unsupportedPropEdit(mergedValues.value, parsed, unsupported.value) ??
        (functionEdit
          ? "Function props cannot be edited as JSON. Keep their retained value or remove them."
          : null);
      if (!codeError.value) setAllValues(parsed as Record<string, unknown>);
    } else {
      codeError.value = "JSON must be an object";
    }
  } catch (e) {
    codeError.value = e instanceof Error ? e.message : "Invalid JSON";
  }
}

// Listen for musea:ready from iframe instead of @load
function onReadyMessage(event: MessageEvent) {
  if (event.origin !== window.location.origin) return;
  const data = event.data;
  if (data?.type !== "musea:ready") return;
  if (event.source !== iframeRef.value?.contentWindow) return;
  iframeReady.value = true;
  // Send initial props if any
  const iframe = iframeRef.value;
  if (!iframe) return;
  if (Object.keys(appliedValues.value).length > 0 || Object.keys(variantAttrs.value).length > 0) {
    sendPreviewProps(iframe, appliedValues.value);
  }
  if (Object.keys(slotContent.value).length > 0) {
    sendMessage(iframe, "musea:set-slots", { slots: slotContent.value });
  }
}

function requestIframeReady() {
  const iframe = iframeRef.value;
  if (!iframe) return;
  iframeReady.value = false;
  sendMessage(iframe, "musea:request-ready");
}

onMounted(() => {
  window.addEventListener("message", onReadyMessage);
});

onUnmounted(() => {
  window.removeEventListener("message", onReadyMessage);
});

function onResetValues() {
  // Reset must also remove the permalink, otherwise reload would reapply it.
  if (route.query[MUSEA_PROPS_QUERY] !== undefined) {
    const { [MUSEA_PROPS_QUERY]: _shared, ...query } = route.query;
    void router.replace({ query, hash: route.hash });
  }
  resetValues(variantValues.value);
  clearSavedValues(props.artPath);
  slotContent.value = {};
  controlsMode.value = "controls";
  saveStatus.value = "idle";
  shareError.value = null;
  shareStatus.value = "";
  const iframe = iframeRef.value;
  if (!iframe || !iframeReady.value) return;
  sendPreviewProps(iframe, appliedValues.value);
  sendMessage(iframe, "musea:set-slots", { slots: {} });
}

function onSaveValues() {
  saveValues(props.artPath);
  saveStatus.value = "saved";
}

async function copyPermalink() {
  shareError.value = null;
  shareStatus.value = "";
  try {
    const url = createPropsPermalink(window.location.href, props.artPath, {
      palette: {
        version: 1,
        values: values.value,
        customProps: customProps.value,
        deletedPaletteProps: [...deletedPaletteProps.value],
      },
      slots: slotContent.value,
    });
    await navigator.clipboard.writeText(url);
    shareStatus.value = "Link copied";
  } catch (error) {
    shareError.value = error instanceof Error ? error.message : "Unable to copy the props link.";
  }
}

function setControlsMode(mode: "controls" | "code") {
  controlsMode.value = mode;
}

function openAddForm() {
  newPropError.value = null;
  showAddForm.value = true;
}

function closeAddForm() {
  showAddForm.value = false;
}

function onSlotsUpdate(slots: Record<string, string>) {
  slotContent.value = slots;
}

function onAddProp() {
  const name = newPropName.value.trim();
  if (!name) return;
  if (unsupported.value.has(name)) {
    newPropError.value = `${name} cannot be applied by Vue.`;
    return;
  }
  newPropError.value = null;
  let defaultValue: unknown = newPropDefault.value;
  if (newPropControl.value === "number") {
    defaultValue = Number(newPropDefault.value) || 0;
  } else if (newPropControl.value === "boolean") {
    defaultValue = newPropDefault.value === "true";
  } else if (newPropControl.value === "object") {
    try {
      defaultValue = JSON.parse(newPropDefault.value || "{}");
    } catch {
      defaultValue = {};
    }
  }
  addProp(name, newPropControl.value, defaultValue);
  newPropName.value = "";
  newPropControl.value = "text";
  newPropDefault.value = "";
  showAddForm.value = false;
}

// Generate usage code
const hasSlotContent = computed(() => {
  return Object.values(slotContent.value).some((v) => v.trim());
});

const usageCode = computed(() => {
  if (!palette.value) return "";
  const componentName = palette.value.componentTagName || palette.value.title || "Component";
  const propsStr = usagePropsAttributes(appliedValues.value);
  const templateOnly = (() => {
    if (!propsStr && !hasSlotContent.value) {
      return `<${componentName} />`;
    }
    if (hasSlotContent.value) {
      const slotParts: string[] = [];
      for (const [name, content] of Object.entries(slotContent.value)) {
        if (!content.trim()) continue;
        if (name === "default") {
          slotParts.push(`  ${content}`);
        } else {
          slotParts.push(`  <template #${name}>\n    ${content}\n  </template>`);
        }
      }
      return `<${componentName}${propsStr}>\n${slotParts.join("\n")}\n</${componentName}>`;
    }
    return `<${componentName}${propsStr} />`;
  })();

  const script = usageScript(art.value?.scriptSetupContent, art.value?.scriptSetupIsolated);
  if (!script) return templateOnly;
  return (
    `<script setup lang="ts">\n${script}\n</` +
    `script>\n\n<template>\n${indentUsage(templateOnly)}\n</template>`
  );
});

const valuesJson = computed(() => JSON.stringify(mergedValues.value, null, 2));

async function copyUsage() {
  try {
    await navigator.clipboard.writeText(usageCode.value);
    copiedUsage.value = true;
    setTimeout(() => {
      copiedUsage.value = false;
    }, 2000);
  } catch {
    // fallback
  }
}

const controlKindOptions = [
  { label: "Text", value: "text" },
  { label: "Number", value: "number" },
  { label: "Boolean", value: "boolean" },
  { label: "Select", value: "select" },
  { label: "Color", value: "color" },
  { label: "Object", value: "object" },
];
</script>

<template>
  <div class="props-panel">
    <div v-if="loading" class="props-loading">
      <div class="loading-spinner" />
      Loading props...
    </div>

    <div v-else-if="error" class="props-error">
      {{ error }}
    </div>

    <template v-else-if="palette">
      <div class="props-split">
        <!-- Left: Live Preview -->
        <div v-if="safeUrl(previewUrl)" class="props-split-left">
          <div class="props-preview">
            <div class="props-preview-header">
              <span class="props-preview-label">Live Preview</span>
            </div>
            <div class="props-preview-frame">
              <iframe
                ref="iframeRef"
                v-musea-globals
                :src="safeUrl(previewUrl)"
                title="Live component preview"
                sandbox="allow-scripts allow-same-origin"
                @load="requestIframeReady"
              />
            </div>
          </div>

          <!-- Usage Code -->
          <div class="props-usage">
            <div class="props-usage-header">
              <span>Usage</span>
              <button type="button" class="props-copy-btn" @click="copyUsage">
                <svg
                  v-if="copiedUsage"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  width="12"
                  height="12"
                >
                  <polyline points="20 6 9 17 4 12" />
                </svg>
                <svg
                  v-else
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  width="12"
                  height="12"
                >
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                </svg>
                {{ copiedUsage ? "Copied!" : "Copy" }}
              </button>
            </div>
            <pre
              class="props-usage-code musea-code-block"
              tabindex="0"
              role="region"
              aria-label="Usage code"
            ><HighlightedCode :code="usageCode" language="xml" /></pre>
          </div>

          <div class="props-json">
            <div class="props-json-header">Current Values</div>
            <pre
              class="props-json-code musea-code-block"
              tabindex="0"
              role="region"
              aria-label="Props JSON"
            ><HighlightedCode :code="valuesJson" language="json" /></pre>
          </div>
        </div>

        <!-- Right: Controls -->
        <div class="props-split-right">
          <div class="props-header">
            <div class="props-heading">
              <h3 class="props-title">Props Controls</h3>
              <label class="variant-select-label">
                <span>Variant</span>
                <select
                  class="variant-select"
                  :value="selectedVariant?.name"
                  @change="emit('selectVariant', ($event.target as HTMLSelectElement).value)"
                >
                  <option v-for="variant in variants" :key="variant.name" :value="variant.name">
                    {{ variant.name }}{{ variant.isDefault ? " · Default" : "" }}
                  </option>
                </select>
              </label>
            </div>
            <div class="props-header-actions">
              <div class="props-mode-toggle">
                <button
                  type="button"
                  class="props-mode-btn"
                  :class="{ active: controlsMode === 'controls' }"
                  @click="() => setControlsMode('controls')"
                >
                  Controls
                </button>
                <button
                  type="button"
                  class="props-mode-btn"
                  :class="{ active: controlsMode === 'code' }"
                  @click="() => setControlsMode('code')"
                >
                  Code
                </button>
              </div>
              <button type="button" class="props-reset" @click="onResetValues">Reset</button>
              <button type="button" class="props-share" @click="copyPermalink">Copy link</button>
              <button
                type="button"
                class="props-save"
                :class="{ saved: saveStatus === 'saved' }"
                @click="onSaveValues"
              >
                {{ saveStatus === "saved" ? "Saved" : "Save" }}
              </button>
            </div>
          </div>

          <p v-if="shareError" class="props-code-error" role="alert">{{ shareError }}</p>
          <p v-else-if="shareStatus" class="props-share-status" role="status">{{ shareStatus }}</p>

          <p v-if="retainedUnsupported.length" class="props-code-error" role="note">
            {{ retainedUnsupported.join(", ") }} is retained only. Preview and copied usage contain
            supported props.
          </p>

          <!-- Controls Mode -->
          <template v-if="controlsMode === 'controls'">
            <div class="props-grid">
              <template v-for="group in palette.groups || []" :key="group">
                <div v-if="group" class="props-group-header">{{ group }}</div>
                <template
                  v-for="control in allControls.filter((c) => c.group === group)"
                  :key="control.name"
                >
                  <div
                    class="props-control-row"
                    :class="{ 'props-control-custom': customPropNames.has(control.name) }"
                  >
                    <div class="props-control-content">
                      <component
                        :is="getControlComponent(control.control, unsupported.has(control.name))"
                        :label="control.name"
                        :description="control.description"
                        :required="control.required"
                        :options="control.options"
                        :min="control.range?.min"
                        :max="control.range?.max"
                        :step="control.range?.step"
                        :model-value="ownRecordValue(values, control.name)"
                        @update:model-value="(v: unknown) => setValue(control.name, v)"
                      />
                    </div>
                    <span v-if="customPropNames.has(control.name)" class="props-custom-badge"
                      >custom</span
                    >
                    <button
                      type="button"
                      class="props-remove-btn"
                      title="Remove prop"
                      @click="() => removeProp(control.name)"
                    >
                      <svg
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        width="14"
                        height="14"
                      >
                        <line x1="18" y1="6" x2="6" y2="18" />
                        <line x1="6" y1="6" x2="18" y2="18" />
                      </svg>
                    </button>
                  </div>
                </template>
              </template>

              <template v-for="control in allControls.filter((c) => !c.group)" :key="control.name">
                <div
                  class="props-control-row"
                  :class="{ 'props-control-custom': customPropNames.has(control.name) }"
                >
                  <div class="props-control-content">
                    <component
                      :is="getControlComponent(control.control, unsupported.has(control.name))"
                      :label="control.name"
                      :description="control.description"
                      :required="control.required"
                      :options="control.options"
                      :min="control.range?.min"
                      :max="control.range?.max"
                      :step="control.range?.step"
                      :model-value="ownRecordValue(values, control.name)"
                      @update:model-value="(v: unknown) => setValue(control.name, v)"
                    />
                  </div>
                  <span v-if="customPropNames.has(control.name)" class="props-custom-badge"
                    >custom</span
                  >
                  <button
                    type="button"
                    class="props-remove-btn"
                    title="Remove prop"
                    @click="() => removeProp(control.name)"
                  >
                    <svg
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      width="14"
                      height="14"
                    >
                      <line x1="18" y1="6" x2="6" y2="18" />
                      <line x1="6" y1="6" x2="18" y2="18" />
                    </svg>
                  </button>
                </div>
              </template>
            </div>

            <!-- Add Prop -->
            <div class="props-add-section">
              <div v-if="showAddForm" class="props-add-form">
                <p v-if="newPropError" class="props-code-error props-add-error" role="alert">
                  {{ newPropError }}
                </p>
                <input
                  v-model="newPropName"
                  type="text"
                  class="props-add-input"
                  aria-label="Prop name"
                  placeholder="Prop name"
                  @keyup.enter="onAddProp"
                />
                <select
                  v-model="newPropControl"
                  class="props-add-select"
                  aria-label="Prop control type"
                >
                  <option v-for="opt in controlKindOptions" :key="opt.value" :value="opt.value">
                    {{ opt.label }}
                  </option>
                </select>
                <input
                  v-model="newPropDefault"
                  type="text"
                  class="props-add-input"
                  aria-label="Default value"
                  placeholder="Default value"
                  @keyup.enter="onAddProp"
                />
                <div class="props-add-actions">
                  <button type="button" class="props-add-confirm" @click="onAddProp">Add</button>
                  <button type="button" class="props-add-cancel" @click="closeAddForm">
                    Cancel
                  </button>
                </div>
              </div>
              <button v-else type="button" class="props-add-btn" @click="openAddForm">
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  width="14"
                  height="14"
                >
                  <line x1="12" y1="5" x2="12" y2="19" />
                  <line x1="5" y1="12" x2="19" y2="12" />
                </svg>
                Add Prop
              </button>
            </div>
          </template>

          <!-- Code Mode -->
          <template v-else>
            <div class="props-code-editor">
              <component
                :is="MonacoEditor"
                :model-value="codeEditorContent"
                language="json"
                height="360px"
                @update:model-value="onCodeEditorUpdate"
              />
              <div v-if="codeError" class="props-code-error">
                {{ codeError }}
              </div>
            </div>
          </template>

          <!-- Slot Editor -->
          <div class="props-slot-editor">
            <SlotEditor :slots="slotContent" @update="onSlotsUpdate" />
          </div>
        </div>
      </div>
    </template>

    <div v-else class="props-empty">
      <p>No props controls available for this component.</p>
      <p class="props-empty-hint">
        Add a <code>component</code> attribute to the <code>&lt;art&gt;</code> block to enable props
        analysis.
      </p>
    </div>
  </div>
</template>

<style scoped>
@import "./PropsPanel.css";
</style>
