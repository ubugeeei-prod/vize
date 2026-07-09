<script setup lang="ts">
import "./ComplexityPanel.css";
import { computed } from "vue";
import type { CrossFileComplexityDimensions, CrossFileComplexityReport } from "../../wasm/index";

const props = defineProps<{
  report: CrossFileComplexityReport | null;
}>();

const DIMENSION_LABELS: Array<[keyof CrossFileComplexityDimensions, string]> = [
  ["templateControlFlow", "Template flow"],
  ["slotUsage", "Slots"],
  ["propDrilling", "Prop drilling"],
  ["globalState", "Global state"],
  ["provideInject", "Provide/inject"],
  ["fallthroughAttrs", "Fallthrough attrs"],
  ["reactiveGraph", "Reactive graph"],
];

const BAND_LABELS: Record<CrossFileComplexityReport["band"], string> = {
  low: "Low",
  moderate: "Moderate",
  high: "High",
  extreme: "Extreme",
};

const dimensions = computed(() => {
  if (!props.report) return [];
  const maxScore = Math.max(1, ...DIMENSION_LABELS.map(([key]) => props.report!.dimensions[key]));
  return DIMENSION_LABELS.map(([key, label]) => {
    const value = props.report!.dimensions[key];
    return { key, label, value, percent: Math.round((value / maxScore) * 100) };
  });
});

const signals = computed(() => {
  if (!props.report) return [];
  const input = props.report.input;
  return [
    ["Components", input.componentCount],
    ["Cyclomatic", props.report.cyclomaticScore],
    ["Cognitive", props.report.cognitiveScore],
    ["v-if tree depth", input.componentTreeVIfMaxDepth],
    ["v-for tree depth", input.componentTreeVForMaxDepth],
    ["Scoped slot depth", input.componentTreeScopedSlotMaxDepth],
    ["Nesting score", input.componentTreeTemplateNestingScore],
    ["Template v-if", input.templateIfCount],
    ["Template v-for", input.templateForCount],
    ["&& / ||", input.templateLogicalOperatorCount],
    ["Prop edges", input.propDrillingEdgeCount],
    ["Provide/inject refs", input.provideInjectReferenceCount],
    ["Reactive edges", input.reactiveEdgeCount],
  ];
});

const bandLabel = computed(() => {
  return props.report ? BAND_LABELS[props.report.band] : "Waiting";
});

const bandClass = computed(() => props.report?.band ?? "waiting");
</script>

<template>
  <section class="complexity-panel" aria-label="Cross-file complexity">
    <div class="complexity-summary">
      <div class="complexity-score">
        <span class="complexity-label">Complexity</span>
        <strong>{{ props.report?.totalScore ?? 0 }}</strong>
      </div>
      <span :class="['complexity-band', bandClass]">{{ bandLabel }}</span>
    </div>

    <div v-if="props.report" class="complexity-bars">
      <div v-for="item in dimensions" :key="item.key" class="complexity-dimension">
        <div class="dimension-row">
          <span>{{ item.label }}</span>
          <span>{{ item.value }}</span>
        </div>
        <div class="dimension-track">
          <div class="dimension-fill" :style="{ width: `${item.percent}%` }"></div>
        </div>
      </div>
    </div>
    <div v-else class="complexity-empty">Analysis pending</div>

    <div v-if="signals.length" class="complexity-signals">
      <div v-for="[label, value] in signals" :key="label" class="signal-chip">
        <span>{{ label }}</span>
        <strong>{{ value }}</strong>
      </div>
    </div>
  </section>
</template>
