<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted, defineAsyncComponent } from "vue";
import hljs from "highlight.js/lib/core";
import xml from "highlight.js/lib/languages/xml";
import json from "highlight.js/lib/languages/json";
import { usePalette } from "../composables/usePalette";
import { getPreviewUrl } from "../api";
import { sendMessage } from "../composables/usePostMessage";
import TextControl from "./controls/TextControl.vue";
import NumberControl from "./controls/NumberControl.vue";
import BooleanControl from "./controls/BooleanControl.vue";
import RangeControl from "./controls/RangeControl.vue";
import SelectControl from "./controls/SelectControl.vue";
import ColorControl from "./controls/ColorControl.vue";
import ObjectControl from "./controls/ObjectControl.vue";
import SlotEditor from "./SlotEditor.vue";

const MonacoEditor = defineAsyncComponent(() => import("./MonacoEditor.vue"));

hljs.registerLanguage("xml", xml);
hljs.registerLanguage("json", json);

const props = defineProps<{
  artPath: string;
  defaultVariantName?: string;
}>();

const {
  palette,
  loading,
  error,
  values,
  allControls,
  mergedValues,
  customPropNames,
  load,
  setValue,
  setAllValues,
  addProp,
  removeProp,
  resetValues,
} = usePalette();

const iframeRef = ref<HTMLIFrameElement | null>(null);
const iframeReady = ref(false);
const slotContent = ref<Record<string, string>>({});
const copiedUsage = ref(false);

// Mode toggle
const controlsMode = ref<"controls" | "code">("controls");

// Code mode state
const codeEditorContent = ref("");
const codeError = ref<string | null>(null);

// Add prop form state
const showAddForm = ref(false);
const newPropName = ref("");
const newPropControl = ref("text");
const newPropDefault = ref("");

const previewUrl = computed(() => {
  if (!props.defaultVariantName) return "";
  return getPreviewUrl(props.artPath, props.defaultVariantName);
});

watch(
  () => props.artPath,
  (path) => {
    if (path) load(path);
    iframeReady.value = false;
    slotContent.value = {};
    controlsMode.value = "controls";
  },
  { immediate: true },
);

// Send props to iframe when mergedValues change
watch(
  mergedValues,
  (newValues) => {
    const iframe = iframeRef.value;
    if (!iframe || !iframeReady.value) return;
    sendMessage(iframe, "musea:set-props", { props: newValues });
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
      codeError.value = null;
      setAllValues(parsed as Record<string, unknown>);
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
  if (Object.keys(mergedValues.value).length > 0) {
    sendMessage(iframe, "musea:set-props", { props: mergedValues.value });
  }
  if (Object.keys(slotContent.value).length > 0) {
    sendMessage(iframe, "musea:set-slots", { slots: slotContent.value });
  }
}

onMounted(() => {
  window.addEventListener("message", onReadyMessage);
});

onUnmounted(() => {
  window.removeEventListener("message", onReadyMessage);
});

function onResetValues() {
  resetValues();
  slotContent.value = {};
  controlsMode.value = "controls";
  const iframe = iframeRef.value;
  if (!iframe || !iframeReady.value) return;
  sendMessage(iframe, "musea:set-props", { props: mergedValues.value });
  sendMessage(iframe, "musea:set-slots", { slots: {} });
}

function onSlotsUpdate(slots: Record<string, string>) {
  slotContent.value = slots;
}

function onAddProp() {
  const name = newPropName.value.trim();
  if (!name) return;
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
  const componentName = palette.value.title || "Component";
  const propsEntries = Object.entries(mergedValues.value).filter(
    ([, v]) => v !== undefined && v !== "",
  );
  if (propsEntries.length === 0 && !hasSlotContent.value) {
    return `<${componentName} />`;
  }
  const propsStr = propsEntries
    .map(([k, v]) => {
      if (typeof v === "boolean") return v ? ` ${k}` : ` :${k}="false"`;
      if (typeof v === "number") return ` :${k}="${v}"`;
      return ` ${k}="${String(v)}"`;
    })
    .join("");
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
});

const usageHighlighted = computed(() => hljs.highlight(usageCode.value, { language: "xml" }).value);

const valuesHighlighted = computed(
  () => hljs.highlight(JSON.stringify(mergedValues.value, null, 2), { language: "json" }).value,
);

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

function getControlComponent(kind: string) {
  switch (kind) {
    case "text":
      return TextControl;
    case "number":
      return NumberControl;
    case "boolean":
      return BooleanControl;
    case "range":
      return RangeControl;
    case "select":
    case "radio":
      return SelectControl;
    case "color":
      return ColorControl;
    case "object":
    case "array":
      return ObjectControl;
    default:
      return TextControl;
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
        <div v-if="previewUrl" class="props-split-left">
          <div class="props-preview">
            <div class="props-preview-header">
              <span class="props-preview-label">Live Preview</span>
            </div>
            <div class="props-preview-frame">
              <iframe ref="iframeRef" :src="previewUrl" />
            </div>
          </div>

          <!-- Usage Code -->
          <div class="props-usage">
            <div class="props-usage-header">
              <span>Usage</span>
              <button type="button" class="props-copy-btn" @click="copyUsage">
                <svg
                  v-if="!copiedUsage"
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
                <svg
                  v-else
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  width="12"
                  height="12"
                >
                  <polyline points="20 6 9 17 4 12" />
                </svg>
                {{ copiedUsage ? "Copied!" : "Copy" }}
              </button>
            </div>
            <pre class="props-usage-code hljs"><code v-html="usageHighlighted"></code></pre>
          </div>

          <div class="props-json">
            <div class="props-json-header">Current Values</div>
            <pre class="props-json-code hljs"><code v-html="valuesHighlighted"></code></pre>
          </div>
        </div>

        <!-- Right: Controls -->
        <div class="props-split-right">
          <div class="props-header">
            <h3 class="props-title">Props Controls</h3>
            <div class="props-header-actions">
              <div class="props-mode-toggle">
                <button
                  type="button"
                  class="props-mode-btn"
                  :class="{ active: controlsMode === 'controls' }"
                  @click="controlsMode = 'controls'"
                >
                  Controls
                </button>
                <button
                  type="button"
                  class="props-mode-btn"
                  :class="{ active: controlsMode === 'code' }"
                  @click="controlsMode = 'code'"
                >
                  Code
                </button>
              </div>
              <button type="button" class="props-reset" @click="onResetValues">Reset</button>
            </div>
          </div>

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
                        :is="getControlComponent(control.control)"
                        :label="control.name"
                        :description="control.description"
                        :required="control.required"
                        :options="control.options"
                        :min="control.range?.min"
                        :max="control.range?.max"
                        :step="control.range?.step"
                        :model-value="values[control.name]"
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
                      @click="removeProp(control.name)"
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
                      :is="getControlComponent(control.control)"
                      :label="control.name"
                      :description="control.description"
                      :required="control.required"
                      :options="control.options"
                      :min="control.range?.min"
                      :max="control.range?.max"
                      :step="control.range?.step"
                      :model-value="values[control.name]"
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
                    @click="removeProp(control.name)"
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
              <button
                v-if="!showAddForm"
                type="button"
                class="props-add-btn"
                @click="showAddForm = true"
              >
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
              <div v-else class="props-add-form">
                <input
                  v-model="newPropName"
                  type="text"
                  class="props-add-input"
                  placeholder="Prop name"
                  @keyup.enter="onAddProp"
                />
                <select v-model="newPropControl" class="props-add-select">
                  <option v-for="opt in controlKindOptions" :key="opt.value" :value="opt.value">
                    {{ opt.label }}
                  </option>
                </select>
                <input
                  v-model="newPropDefault"
                  type="text"
                  class="props-add-input"
                  placeholder="Default value"
                  @keyup.enter="onAddProp"
                />
                <div class="props-add-actions">
                  <button type="button" class="props-add-confirm" @click="onAddProp">Add</button>
                  <button type="button" class="props-add-cancel" @click="showAddForm = false">
                    Cancel
                  </button>
                </div>
              </div>
            </div>
          </template>

          <!-- Code Mode -->
          <template v-else>
            <div class="props-code-editor">
              <MonacoEditor
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
.props-panel {
  padding: 0.5rem;
}

.props-split {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.props-split-left {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  position: sticky;
  top: 0;
  align-self: start;
}

.props-split-right {
  display: flex;
  flex-direction: column;
}

@media (max-width: 900px) {
  .props-split {
    grid-template-columns: 1fr;
  }
}

.props-loading {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  justify-content: center;
  min-height: 200px;
  color: var(--musea-text-muted);
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--musea-border);
  border-top-color: var(--musea-accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.props-error {
  padding: 1rem;
  color: var(--musea-error);
  background: rgba(248, 113, 113, 0.1);
  border: 1px solid rgba(248, 113, 113, 0.2);
  border-radius: var(--musea-radius-md);
  font-size: 0.8125rem;
}

.props-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1.25rem;
}

.props-title {
  font-size: 0.875rem;
  font-weight: 600;
}

.props-header-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.props-mode-toggle {
  display: flex;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  overflow: hidden;
}

.props-mode-btn {
  background: var(--musea-bg-tertiary);
  border: none;
  color: var(--musea-text-muted);
  font-size: 0.6875rem;
  font-weight: 500;
  padding: 0.25rem 0.625rem;
  cursor: pointer;
  transition: all var(--musea-transition);
}

.props-mode-btn:not(:last-child) {
  border-right: 1px solid var(--musea-border);
}

.props-mode-btn.active {
  background: var(--musea-accent);
  color: var(--musea-accent-contrast);
}

.props-mode-btn:not(.active):hover {
  color: var(--musea-text);
}

.props-reset {
  background: var(--musea-bg-tertiary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  color: var(--musea-text-muted);
  font-size: 0.75rem;
  padding: 0.25rem 0.625rem;
  cursor: pointer;
  transition: all var(--musea-transition);
}

.props-reset:hover {
  border-color: var(--musea-text-muted);
  color: var(--musea-text);
}

.props-grid {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.props-group-header {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--musea-text-muted);
  margin-top: 0.5rem;
  padding-bottom: 0.375rem;
  border-bottom: 1px solid var(--musea-border-subtle);
}

/* Control row with remove button */
.props-control-row {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 0.25rem;
}

.props-control-content {
  flex: 1;
  min-width: 0;
}

.props-control-custom {
  border-left: 2px dashed var(--musea-accent);
  padding-left: 0.5rem;
}

.props-custom-badge {
  font-size: 0.5625rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--musea-accent);
  background: rgba(224, 112, 72, 0.1);
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  white-space: nowrap;
  margin-top: 0.25rem;
}

.props-remove-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: none;
  border-radius: 3px;
  background: transparent;
  color: var(--musea-text-muted);
  cursor: pointer;
  opacity: 0;
  transition: all var(--musea-transition);
  margin-top: 0.25rem;
}

.props-control-row:hover .props-remove-btn {
  opacity: 1;
}

.props-remove-btn:hover {
  background: rgba(248, 113, 113, 0.15);
  color: var(--musea-error, #f87171);
}

/* Add prop section */
.props-add-section {
  margin-top: 1rem;
}

.props-add-btn {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  width: 100%;
  padding: 0.5rem;
  border: 1px dashed var(--musea-border);
  border-radius: var(--musea-radius-sm);
  background: transparent;
  color: var(--musea-text-muted);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all var(--musea-transition);
  justify-content: center;
}

.props-add-btn:hover {
  border-color: var(--musea-accent);
  color: var(--musea-accent);
}

.props-add-form {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.75rem;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  background: var(--musea-bg-secondary);
}

.props-add-input {
  padding: 0.375rem 0.5rem;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  background: var(--musea-bg);
  color: var(--musea-text);
  font-size: 0.75rem;
  font-family: var(--musea-font-mono);
  outline: none;
  transition: border-color var(--musea-transition);
}

.props-add-input:focus {
  border-color: var(--musea-accent);
}

.props-add-select {
  padding: 0.375rem 0.5rem;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  background: var(--musea-bg);
  color: var(--musea-text);
  font-size: 0.75rem;
  outline: none;
}

.props-add-actions {
  display: flex;
  gap: 0.375rem;
}

.props-add-confirm {
  padding: 0.25rem 0.75rem;
  border: none;
  border-radius: var(--musea-radius-sm);
  background: var(--musea-accent);
  color: var(--musea-accent-contrast);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity var(--musea-transition);
}

.props-add-confirm:hover {
  opacity: 0.85;
}

.props-add-cancel {
  padding: 0.25rem 0.75rem;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  background: transparent;
  color: var(--musea-text-muted);
  font-size: 0.75rem;
  cursor: pointer;
  transition: all var(--musea-transition);
}

.props-add-cancel:hover {
  color: var(--musea-text);
  border-color: var(--musea-text-muted);
}

/* Code editor */
.props-code-editor {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.props-code-error {
  padding: 0.375rem 0.625rem;
  font-size: 0.6875rem;
  font-family: var(--musea-font-mono);
  color: var(--musea-error, #f87171);
  background: rgba(248, 113, 113, 0.1);
  border: 1px solid rgba(248, 113, 113, 0.2);
  border-radius: var(--musea-radius-sm);
}

.props-json {
  margin-top: 1.5rem;
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  overflow: hidden;
}

.props-json-header {
  padding: 0.5rem 0.75rem;
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--musea-text-muted);
  background: var(--musea-bg-tertiary);
  border-bottom: 1px solid var(--musea-border);
}

.props-json-code {
  padding: 0.75rem;
  font-family: var(--musea-font-mono);
  font-size: 0.75rem;
  color: var(--musea-text-secondary);
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.props-preview {
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  overflow: hidden;
}

.props-preview-header {
  padding: 0.5rem 0.75rem;
  background: var(--musea-bg-tertiary);
  border-bottom: 1px solid var(--musea-border);
}

.props-preview-label {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--musea-text-muted);
}

.props-preview-frame {
  aspect-ratio: 4 / 3;
  background: #fff;
  max-height: 300px;
}

.props-preview-frame iframe {
  width: 100%;
  height: 100%;
  border: none;
}

.props-slot-editor {
  margin-top: 1.25rem;
}

.props-usage {
  margin-top: 1.25rem;
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-md);
  overflow: hidden;
}

.props-usage-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: var(--musea-bg-tertiary);
  border-bottom: 1px solid var(--musea-border);
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--musea-text-muted);
}

.props-copy-btn {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.125rem 0.375rem;
  border: 1px solid var(--musea-border);
  border-radius: 3px;
  background: var(--musea-bg-tertiary);
  color: var(--musea-text-muted);
  font-size: 0.625rem;
  cursor: pointer;
  transition: all var(--musea-transition);
}

.props-copy-btn:hover {
  color: var(--musea-text);
  border-color: var(--musea-text-muted);
}

.props-usage-code {
  padding: 0.75rem;
  font-family: "SF Mono", "Fira Code", "Consolas", monospace;
  font-size: 0.75rem;
  color: var(--musea-text-secondary);
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.props-empty {
  padding: 2rem;
  text-align: center;
  color: var(--musea-text-muted);
  font-size: 0.875rem;
}

.props-empty-hint {
  margin-top: 0.5rem;
  font-size: 0.8125rem;
}

.props-empty code {
  background: var(--musea-bg-tertiary);
  padding: 0.125rem 0.375rem;
  border-radius: 4px;
  font-family: var(--musea-font-mono);
}
</style>
