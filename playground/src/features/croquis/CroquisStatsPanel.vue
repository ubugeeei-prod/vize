<script setup lang="ts">
import { computed } from "vue";
import type {
  CroquisStats,
  CssDisplay,
  InvalidExportDisplay,
  MacroDisplay,
  TypeExportDisplay,
} from "../../wasm/index";

const props = defineProps<{
  stats?: CroquisStats;
  macros?: MacroDisplay[];
  css?: CssDisplay;
  typeExports?: TypeExportDisplay[];
  invalidExports?: InvalidExportDisplay[];
}>();

const macros = computed(() => props.macros ?? []);
const typeExports = computed(() => props.typeExports ?? []);
const invalidExports = computed(() => props.invalidExports ?? []);
</script>

<template>
  <div class="stats-output">
    <div class="stats-grid">
      <div class="stat-box">
        <div class="stat-number">{{ stats?.binding_count || 0 }}</div>
        <div class="stat-label">Bindings</div>
      </div>
      <div class="stat-box">
        <div class="stat-number">{{ stats?.macro_count || 0 }}</div>
        <div class="stat-label">Macros</div>
      </div>
      <div class="stat-box">
        <div class="stat-number">{{ stats?.scope_count || 0 }}</div>
        <div class="stat-label">Scopes</div>
      </div>
      <div class="stat-box">
        <div class="stat-number">{{ css?.v_bind_count || 0 }}</div>
        <div class="stat-label">v-bind()</div>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">Compiler Macros</h3>
      <div v-if="macros.length === 0" class="empty-state">No macros detected</div>
      <div v-else class="macro-list">
        <div v-for="macro in macros" :key="`${macro.name}-${macro.start}`" class="macro-item">
          <span class="macro-name">{{ macro.name }}</span>
          <code v-if="macro.type_args" class="macro-type">{{ macro.type_args }}</code>
          <span v-if="macro.binding" class="macro-binding">-> {{ macro.binding }}</span>
        </div>
      </div>
    </div>

    <div v-if="css" class="section">
      <h3 class="section-title">CSS Analysis</h3>
      <div class="css-info">
        <span class="css-stat">{{ css.selector_count }} selectors</span>
        <span v-if="css.is_scoped" class="css-badge scoped">scoped</span>
        <span v-if="css.v_bind_count > 0" class="css-badge vbind"
          >{{ css.v_bind_count }} v-bind</span
        >
      </div>
    </div>

    <div v-if="typeExports.length > 0" class="section">
      <h3 class="section-title">Type Exports <span class="badge hoisted">hoisted</span></h3>
      <div class="export-list">
        <div v-for="te in typeExports" :key="`${te.name}-${te.start}`" class="export-item valid">
          <span class="export-kind">{{ te.kind }}</span>
          <code class="export-name">{{ te.name }}</code>
          <span class="export-badge hoisted">hoisted to module</span>
        </div>
      </div>
    </div>

    <div v-if="invalidExports.length > 0" class="section">
      <h3 class="section-title">Invalid Exports <span class="badge error">error</span></h3>
      <div class="export-list">
        <div
          v-for="ie in invalidExports"
          :key="`${ie.name}-${ie.start}`"
          class="export-item invalid"
        >
          <span class="export-kind">{{ ie.kind }}</span>
          <code class="export-name">{{ ie.name }}</code>
          <span class="export-badge error">not allowed in script setup</span>
        </div>
      </div>
    </div>
  </div>
</template>
