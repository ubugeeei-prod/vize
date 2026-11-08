<script setup lang="ts">
import { computed } from "vue";
import type { CrossFileComplexityDimensions, CrossFileComplexityReport } from "../../wasm/index";

const { report } = defineProps<{
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
  if (!report) return [];
  const maxScore = Math.max(1, ...DIMENSION_LABELS.map(([key]) => report!.dimensions[key]));
  return DIMENSION_LABELS.map(([key, label]) => {
    const value = report!.dimensions[key];
    return { key, label, value, percent: Math.round((value / maxScore) * 100) };
  });
});

const signals = computed(() => {
  if (!report) return [];
  const input = report.input;
  return [
    ["Components", input.componentCount],
    ["Cyclomatic", report.cyclomaticScore],
    ["Cognitive", report.cognitiveScore],
    ["Deepest nesting", input.templateMaxNesting],
    ["Scoped slots", input.templateScopedSlotCount],
    ["Unanalyzed expressions", input.templateUnknown],
    ["Prop edges", input.propDrillingEdgeCount],
    ["Provide/inject refs", input.provideInjectReferenceCount],
    ["Reactive edges", input.reactiveEdgeCount],
  ];
});

const bandLabel = computed(() => {
  return report ? BAND_LABELS[report.band] : "Waiting";
});

const bandClass = computed(() => report?.band ?? "waiting");
</script>

<template>
  <section class="complexity-panel" aria-label="Cross-file complexity">
    <div class="complexity-summary">
      <div class="complexity-score">
        <span class="complexity-label">Complexity</span>
        <strong>{{ report?.totalScore ?? 0 }}</strong>
      </div>
      <span :class="['complexity-band', bandClass]">{{ bandLabel }}</span>
    </div>

    <div v-if="report" class="complexity-bars">
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

<style>
.complexity-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-primary);
  background: color-mix(in srgb, var(--bg-secondary) 82%, var(--bg-primary));
}

.complexity-summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.complexity-score {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.complexity-label {
  color: var(--text-muted);
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
}

.complexity-score strong {
  color: var(--text-primary);
  font-size: 26px;
  line-height: 1;
}

.complexity-band {
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  padding: 4px 7px;
  color: var(--text-secondary);
  font-size: 10px;
  font-weight: 700;
}

.complexity-band.low {
  border-color: var(--color-success);
  color: var(--color-success);
}

.complexity-band.moderate {
  border-color: var(--color-warning);
  color: var(--color-warning);
}

.complexity-band.high,
.complexity-band.extreme {
  border-color: var(--color-error);
  color: var(--color-error);
}

.complexity-bars {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.complexity-dimension {
  display: grid;
  gap: 3px;
}

.dimension-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  color: var(--text-secondary);
  font-size: 10px;
}

.dimension-row span:first-child {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dimension-track {
  height: 5px;
  overflow: hidden;
  border-radius: 3px;
  background: var(--bg-tertiary);
}

.dimension-fill {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--accent-rust), var(--color-success));
}

.complexity-empty {
  color: var(--text-muted);
  font-size: 11px;
}

.complexity-signals {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 5px;
}

.signal-chip {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  min-width: 0;
  padding: 5px 6px;
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  background: var(--bg-primary);
  color: var(--text-muted);
  font-size: 10px;
}

.signal-chip span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.signal-chip strong {
  color: var(--text-primary);
  font-size: 11px;
}
</style>
