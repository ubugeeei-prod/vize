<script setup lang="ts">
import { ref } from "vue";
import { mdiCodeTags } from "@mdi/js";
import MonacoEditor, { type ScopeDecoration, type Diagnostic } from "../../shared/MonacoEditor.vue";
import ExperimentalFeatures from "../../shared/ExperimentalFeatures.vue";
import type { ExperimentalOptions } from "../../shared/experimentalFeatures";
import { ANALYSIS_PRESET } from "../../shared/presets/croquis";

defineProps<{ theme: "dark" | "light"; scopes: ScopeDecoration[]; diagnostics: Diagnostic[] }>();
const source = defineModel<string>({ required: true });
const visualize = defineModel<boolean>("visualize", { required: true });
const experimentals = defineModel<ExperimentalOptions>("experimentals", { required: true });
const emit = defineEmits<{ example: [key: string] }>();
function loadExample(key: string) {
  emit("example", key);
}
const editor = ref<InstanceType<typeof MonacoEditor> | null>(null);
defineExpose({
  applyScopeDecorations: (scopes: ScopeDecoration[]) => editor.value?.applyScopeDecorations(scopes),
});
</script>

<template>
  <div class="panel-header">
    <div class="header-title">
      <svg class="icon" viewBox="0 0 24 24"><path :d="mdiCodeTags" fill="currentColor" /></svg>
      <h2>Source</h2>
    </div>
    <div class="panel-actions">
      <label class="toggle-label">
        <input v-model="visualize" type="checkbox" />
        <span>Visualize Scopes</span>
      </label>
      <button type="button" class="btn-ghost" @click="() => (source = ANALYSIS_PRESET)">
        Reset
      </button>
    </div>
  </div>
  <div class="experimental-controls">
    <ExperimentalFeatures v-model="experimentals" scope="croquis" @example="loadExample" />
  </div>
  <div class="editor-container">
    <MonacoEditor ref="editor" v-model="source" language="vue" :scopes :diagnostics :theme />
  </div>
</template>
