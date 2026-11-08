<script setup lang="ts">
import { computed } from "vue";
import type { ReactivityOverlay, ReactivityOverlayLoss } from "../../wasm/index";

const { overlay } = defineProps<{
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

const summary = computed(() => overlay?.summary ?? emptySummary);
const sources = computed(() => overlay?.sources ?? []);
const losses = computed(() => overlay?.losses ?? []);
const edges = computed(() => overlay?.effectGraph.edges ?? []);
const cycle = computed(() => overlay?.effectGraph.cycle ?? null);

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

    <section class="reactivity-section" aria-label="Sources">
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

    <section class="reactivity-section" aria-label="Losses">
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

    <section class="reactivity-section" aria-label="Effect graph">
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

<style>
.reactivity-output {
  padding: 1rem;
  overflow-y: auto;
}

.reactivity-metrics {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.75rem;
  margin-bottom: 1rem;
}

.reactivity-metric {
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  padding: 0.75rem;
  background: var(--bg-secondary);
}

.reactivity-metric strong {
  display: block;
  font-size: 1.25rem;
  line-height: 1.2;
  color: var(--text-primary);
}

.reactivity-metric span {
  display: block;
  margin-top: 0.25rem;
  font-size: 0.7rem;
  color: var(--text-muted);
}

.reactivity-metric.danger strong {
  color: var(--color-error);
}

.reactivity-section {
  margin-top: 1rem;
}

.reactivity-source-list,
.reactivity-loss-list,
.reactivity-edge-list {
  display: grid;
  gap: 0.5rem;
}

.reactivity-source-row,
.reactivity-loss,
.reactivity-edge {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-height: 2.25rem;
  border: 1px solid var(--border-primary);
  border-radius: 6px;
  padding: 0.5rem 0.625rem;
  background: var(--bg-secondary);
}

.reactivity-source-row code,
.reactivity-loss code,
.reactivity-edge code,
.reactivity-cycle code {
  font-family: "JetBrains Mono", monospace;
  font-size: 0.75rem;
  color: var(--text-primary);
}

.reactivity-kind,
.reactivity-value,
.reactivity-loss-kind {
  border-radius: 4px;
  padding: 0.125rem 0.375rem;
  font-size: 0.65rem;
  font-weight: 600;
  text-transform: uppercase;
}

.reactivity-kind.ref,
.reactivity-value {
  background: rgba(59, 130, 246, 0.14);
  color: #60a5fa;
}

.reactivity-kind.reactive {
  background: rgba(16, 185, 129, 0.14);
  color: #34d399;
}

.reactivity-kind.computed {
  background: rgba(245, 158, 11, 0.16);
  color: #fbbf24;
}

.reactivity-kind.readonly {
  background: rgba(148, 163, 184, 0.16);
  color: #94a3b8;
}

.reactivity-loss-kind {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.reactivity-loss-main {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
  flex: 1;
}

.reactivity-offset {
  margin-left: auto;
  font-family: "JetBrains Mono", monospace;
  font-size: 0.68rem;
  color: var(--text-muted);
}

.reactivity-edge span,
.reactivity-cycle span {
  color: var(--text-muted);
}

.reactivity-cycle {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.75rem;
  border: 1px solid var(--color-error-border);
  border-radius: 6px;
  padding: 0.625rem;
  background: var(--color-error-bg);
}

@media (max-width: 760px) {
  .reactivity-metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
