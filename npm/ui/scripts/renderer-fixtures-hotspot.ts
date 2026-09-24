export const hotspotRendererFixtures = [
  {
    filename: "HotspotConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  HotspotArea,
  HotspotContent,
  HotspotImage,
  HotspotMarker,
  HotspotRoot,
} from "./families/media/hotspot/hotspot.ts";
import type { HotspotDefinition } from "./families/media/hotspot/hotspot.ts";

const active = ref<string | null>(null);
const items: readonly HotspotDefinition<{ readonly price: number }>[] = [
  { id: "lamp", x: 25, y: 40, label: "Floor lamp", data: { price: 120 } },
  { id: "sofa", x: 60, y: 70, label: "Sofa", data: { price: 900 } },
];
</script>

<template>
  <HotspotRoot v-model:active="active" :hotspots="items">
    <template #default="{ hotspots }">
      <HotspotImage src="/room.jpg" alt="Living room" />
      <HotspotMarker
        v-for="item in hotspots"
        :id="item.id"
        :key="item.id"
        :x="item.x"
        :y="item.y"
        :label="item.label"
      >
        <HotspotContent>{{ item.label }} {{ item.data?.price }}</HotspotContent>
      </HotspotMarker>
      <HotspotArea id="window" label="Window" :shape="{ type: 'rect', x: 70, y: 5, width: 25, height: 30 }" />
    </template>
  </HotspotRoot>
</template>
`,
  },
] as const;
