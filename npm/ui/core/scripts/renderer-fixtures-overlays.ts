/** Overlay component fixtures compiled by every supported renderer lane. */
export const overlayRendererFixtures = [
  {
    filename: "NestedPortalConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Portal, usePortalStack } from "./portal.ts";

const target = ref("body");
const stack = usePortalStack();
</script>

<template>
  <Portal :to="target">
    <div data-portalled="outer">
      Outer layer
      <Portal :to="target">
        <div data-portalled="inner" :data-layers="stack.value.length">Inner layer</div>
      </Portal>
    </div>
  </Portal>
</template>
`,
  },
  {
    filename: "NestedPresenceConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import Presence from "./presence.vue";

const outerOpen = ref(true);
const innerOpen = ref(true);
</script>

<template>
  <Presence :present="outerOpen">
    <div>
      Outer layer
      <Presence :present="innerOpen">
        <div>Inner layer</div>
      </Presence>
    </div>
  </Presence>
</template>
`,
  },
];
