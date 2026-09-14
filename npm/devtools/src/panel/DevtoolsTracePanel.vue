<script setup lang="ts">
import { computed } from "vue";

import type { DevtoolsSnapshot } from "../types.ts";

const props = withDefaults(
  defineProps<{
    readonly emptyLabel?: string;
    readonly snapshot: DevtoolsSnapshot;
  }>(),
  {
    emptyLabel: "No trace events",
  },
);

const eventCount = computed(() => props.snapshot.events.length);
const renderNodes = computed(() => props.snapshot.renderTree.slice(0, 8));
const provideNodes = computed(() => props.snapshot.provideTree.slice(0, 8));
const suspenseNodes = computed(() => props.snapshot.suspenseTree.slice(0, 8));
const graphNodes = computed(() => props.snapshot.reactiveGraph.nodes.slice(0, 10));
const graphEdges = computed(() => props.snapshot.reactiveGraph.edges.slice(0, 16));
const graphPoints = computed(() => {
  const total = Math.max(1, graphNodes.value.length);
  return Object.fromEntries(
    graphNodes.value.map((node, index) => {
      const angle = (index / total) * Math.PI * 2 - Math.PI / 2;
      return [
        node.id,
        {
          x: 120 + Math.cos(angle) * 86,
          y: 108 + Math.sin(angle) * 72,
        },
      ];
    }),
  );
});

function pointFor(id: string) {
  return graphPoints.value[id] ?? { x: 120, y: 108 };
}
</script>

<template>
  <section data-vize-devtools-panel="trace">
    <header>
      <h2>Vize Devtools</h2>
      <p>{{ snapshot.sessionId }} / {{ eventCount }} events</p>
    </header>

    <p v-if="!snapshot.enabled || eventCount === 0" data-vize-devtools-empty>
      {{ emptyLabel }}
    </p>

    <div v-else>
      <section aria-labelledby="vize-devtools-render-heading">
        <h3 id="vize-devtools-render-heading">Render updates</h3>
        <ol>
          <li v-for="node in renderNodes" :key="node.componentId">
            <strong>{{ node.componentName }}</strong>
            <span>{{ node.renderCount }} renders</span>
            <span>{{ node.updateCount }} updates</span>
            <span>{{ node.lastEventKind }}</span>
          </li>
        </ol>
      </section>

      <section aria-labelledby="vize-devtools-reactivity-heading">
        <h3 id="vize-devtools-reactivity-heading">Reactive graph</h3>
        <svg viewBox="0 0 240 216" role="img" aria-labelledby="vize-devtools-reactivity-heading">
          <line
            v-for="edge in graphEdges"
            :key="`${edge.sourceId}:${edge.targetId}:${edge.operation}`"
            :x1="pointFor(edge.sourceId).x"
            :x2="pointFor(edge.targetId).x"
            :y1="pointFor(edge.sourceId).y"
            :y2="pointFor(edge.targetId).y"
            :data-operation="edge.operation"
            stroke="currentColor"
            stroke-width="1"
            opacity="0.34"
          />
          <g v-for="node in graphNodes" :key="node.id" :data-node-kind="node.kind">
            <circle
              :cx="pointFor(node.id).x"
              :cy="pointFor(node.id).y"
              :r="node.kind === 'effect' ? 12 : 9"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            />
            <text
              :x="pointFor(node.id).x"
              :y="pointFor(node.id).y + 26"
              text-anchor="middle"
              font-size="10"
            >
              {{ node.label }}
            </text>
          </g>
        </svg>
      </section>

      <section aria-labelledby="vize-devtools-provide-heading">
        <h3 id="vize-devtools-provide-heading">Provide tree</h3>
        <ol>
          <li v-for="provider in provideNodes" :key="provider.providerId">
            <strong>{{ provider.ownerComponentName }}</strong>
            <span>{{ provider.providedKeys.join(", ") }}</span>
            <small>{{ provider.consumers.length }} consumers</small>
          </li>
        </ol>
      </section>

      <section aria-labelledby="vize-devtools-suspense-heading">
        <h3 id="vize-devtools-suspense-heading">Suspense tree</h3>
        <ol>
          <li v-for="boundary in suspenseNodes" :key="boundary.boundaryId">
            <strong>{{ boundary.componentName }}</strong>
            <span>{{ boundary.state }}</span>
            <small v-if="boundary.asyncDependency">{{ boundary.asyncDependency }}</small>
            <small v-if="boundary.error">{{ boundary.error }}</small>
          </li>
        </ol>
      </section>
    </div>
  </section>
</template>
