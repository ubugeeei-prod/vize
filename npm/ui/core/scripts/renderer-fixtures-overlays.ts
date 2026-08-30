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
    filename: "MotionConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { motionTokenVar, startViewTransition, useReducedMotion } from "./motion.ts";

const reduced = useReducedMotion();
const count = ref(0);
function next() {
  void startViewTransition(() => {
    count.value += 1;
  }).finished;
}
</script>

<template>
  <button
    type="button"
    data-vize-motion="enter"
    :data-reduced-motion="reduced.value || undefined"
    :data-motion-ease="motionTokenVar('ease-standard')"
    @click="next"
  >
    Advance
  </button>
  <output>{{ count }}</output>
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
  {
    filename: "TooltipConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { TooltipContent, TooltipRoot, TooltipTrigger } from "./tooltip.ts";
</script>

<template>
  <TooltipRoot id="renderer-tooltip" :delay-duration="0">
    <TooltipTrigger>More details</TooltipTrigger>
    <TooltipContent portal-disabled>Compiled across every renderer lane.</TooltipContent>
  </TooltipRoot>
</template>
`,
  },
];
