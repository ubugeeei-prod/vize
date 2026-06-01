<script setup lang="ts">
import type {} from "./shims";
import { computed, shallowRef } from "vue";
import { Panel, VueFlow, addEdge, type Connection, type Edge, type Node } from "@vue-flow/core";
import { TresCanvas } from "@tresjs/core";
import type { FlowNodeData } from "./types";

const nodes = shallowRef<Node<FlowNodeData>[]>([
  {
    id: "source",
    type: "input",
    position: { x: 40, y: 60 },
    data: { label: "Vue Flow source", status: "ready" },
  },
  {
    id: "target",
    type: "output",
    position: { x: 280, y: 160 },
    data: { label: "Vue Flow target", status: "blocked" },
  },
]);

const edges = shallowRef<Edge[]>([
  {
    id: "source-target",
    source: "source",
    target: "target",
    animated: true,
    label: "strict edge",
  },
]);

const selectedNodeId = shallowRef("source");
const cubeRotation = shallowRef<[number, number, number]>([0.25, 0.45, 0]);

const selectedNode = computed(() => {
  return nodes.value.find((node) => node.id === selectedNodeId.value) ?? nodes.value[0];
});

function handleConnect(connection: Connection): void {
  const nextElements = addEdge({ ...connection, animated: true }, edges.value);
  edges.value = nextElements.filter((element): element is Edge => {
    return "source" in element && "target" in element;
  });
}
</script>

<template>
  <section class="graph-grid" aria-label="Graph and 3D coverage">
    <VueFlow
      v-model:nodes="nodes"
      v-model:edges="edges"
      class="flow"
      fit-view-on-init
      @connect="handleConnect"
      @node-click="selectedNodeId = $event.node.id"
    >
      <Panel position="top-left">
        {{ selectedNode.data.label }}: {{ selectedNode.data.status }}
      </Panel>
    </VueFlow>

    <div class="tres-scene">
      <TresCanvas clear-color="#111827">
        <TresPerspectiveCamera :position="[3, 3, 5]" :look-at="[0, 0, 0]" />
        <TresAmbientLight :intensity="1.2" />
        <TresMesh :rotation="cubeRotation">
          <TresBoxGeometry :args="[1.2, 1.2, 1.2]" />
          <TresMeshStandardMaterial color="#42b883" />
        </TresMesh>
      </TresCanvas>
    </div>
  </section>
</template>

<style scoped>
.graph-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 12px;
}

.flow,
.tres-scene {
  border: 1px solid #d0d7de;
  border-radius: 8px;
  height: 360px;
  overflow: hidden;
}
</style>
