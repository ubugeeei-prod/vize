<!-- Product photo with labelled markers that open detail popovers and a clickable window region. -->
<script setup lang="ts">
import { ref } from "vue";

import { Hotspot, HotspotArea, HotspotContent, HotspotImage, HotspotMarker } from "../hotspot.ts";

const active = ref<string | null>(null);
</script>

<template>
  <Hotspot v-model:active="active">
    <HotspotImage src="/media/living-room.jpg" alt="Furnished living room" />
    <HotspotMarker id="lamp" :x="25" :y="40" label="Floor lamp">
      <HotspotContent>Brass floor lamp with a linen shade.</HotspotContent>
    </HotspotMarker>
    <HotspotMarker id="sofa" :x="60" :y="70" label="Sofa">
      <HotspotContent>Three-seat sofa in oat linen.</HotspotContent>
    </HotspotMarker>
    <HotspotArea
      id="window"
      label="Window"
      :shape="{ type: 'rect', x: 70, y: 5, width: 25, height: 30 }"
    />
  </Hotspot>
  <output>{{ active ?? "No hotspot selected" }}</output>
</template>
