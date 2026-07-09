<script setup lang="ts">
import "./ReactivityOverlayPanel.css";
import { computed } from "vue";
import type { ReactivityOverlay, ReactivityOverlayLoss } from "../../wasm/index";

const props = defineProps<{
  overlay?: ReactivityOverlay;
}>();

const emptySummary = {
  sourceCount: 0,
  refSourceCount: 0,
  reactiveSourceCount: 0,
  computedSourceCount: 0,
  readonlySourceCount: 0,
  needsValueAccessCount: 0,
  lossCount: 0,
  effectEdgeCount: 0,
  effectCycleCount: 0,
};

const summary = computed(() => props.overlay?.summary ?? emptySummary);
const sources = computed(() => props.overlay?.sources ?? []);
const losses = computed(() => props.overlay?.losses ?? []);
const edges = computed(() => props.overlay?.effectGraph.edges ?? []);
const cycle = computed(() => props.overlay?.effectGraph.cycle ?? null);

function labelLoss(loss: ReactivityOverlayLoss): string {
  switch (loss.kind) {
    case "reactiveDestructure":
    case "refValueDestructure":
    case "propsDestructure":
      return `destructure ${loss.extractedProps.join(", ") || "binding"}`;
    case "refValueExtract":
      return `${loss.sourceName}.value -> ${loss.targetName}`;
    case "reactivePropertyExtract":
      return `${loss.sourceName}.${loss.propertyName} -> ${loss.targetName}`;
    case "functionArgumentExtract":
      return `${loss.argumentName} -> ${loss.calleeName}()`;
    case "getterCallExtract":
      return `${loss.getterName}() -> ${loss.targetName}`;
    case "plainValueAlias":
      return `${loss.sourceName} -> ${loss.targetName}`;
    case "reactiveSpread":
      return `{ ...${loss.sourceName} }`;
    case "reactiveReassign":
      return `${loss.sourceName} = ...`;
  }
}
</script>

<template>
  <div class="reactivity-output">
    <div class="reactivity-metrics">
      <div class="reactivity-metric">
        <strong>{{ summary.sourceCount }}</strong>
        <span>Sources</span>
      </div>
      <div class="reactivity-metric danger">
        <strong>{{ summary.lossCount }}</strong>
        <span>Losses</span>
      </div>
      <div class="reactivity-metric">
        <strong>{{ summary.needsValueAccessCount }}</strong>
        <span>.value</span>
      </div>
      <div class="reactivity-metric">
        <strong>{{ summary.effectEdgeCount }}</strong>
        <span>Effects</span>
      </div>
    </div>

    <section class="reactivity-section">
      <h3 class="section-title">Sources</h3>
      <div v-if="sources.length === 0" class="empty-state">No reactive sources</div>
      <div v-else class="reactivity-source-list">
        <div v-for="source in sources" :key="source.id" class="reactivity-source-row">
          <code>{{ source.name }}</code>
          <span :class="['reactivity-kind', source.category]">{{ source.kind }}</span>
          <span v-if="source.needsValueAccess" class="reactivity-value">.value</span>
          <span class="reactivity-offset">
            {{ source.declarationOffset }}:{{ source.declarationEndOffset }}
          </span>
        </div>
      </div>
    </section>

    <section class="reactivity-section">
      <h3 class="section-title">Losses</h3>
      <div v-if="losses.length === 0" class="success-state">No reactivity loss</div>
      <div v-else class="reactivity-loss-list">
        <div v-for="loss in losses" :key="`${loss.kind}-${loss.start}`" class="reactivity-loss">
          <div class="reactivity-loss-main">
            <span class="reactivity-loss-kind">{{ loss.kind }}</span>
            <code>{{ labelLoss(loss) }}</code>
          </div>
          <span class="reactivity-offset">{{ loss.start }}:{{ loss.end }}</span>
        </div>
      </div>
    </section>

    <section class="reactivity-section">
      <h3 class="section-title">Effect Graph</h3>
      <div v-if="edges.length === 0" class="empty-state">No effect edges</div>
      <div v-else class="reactivity-edge-list">
        <div v-for="edge in edges" :key="`${edge.from}-${edge.to}`" class="reactivity-edge">
          <code>{{ edge.from }}</code>
          <span>-></span>
          <code>{{ edge.to }}</code>
        </div>
      </div>
      <div v-if="cycle" class="reactivity-cycle">
        <span>Cycle</span>
        <code>{{ cycle.join(" -> ") }}</code>
      </div>
    </section>
  </div>
</template>
