export const marqueeRendererFixtures = [
  {
    filename: "MarqueeConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  MarqueeContent,
  MarqueePauseButton,
  MarqueeRoot,
} from "./families/media/marquee/marquee.ts";

const playing = ref(true);
</script>

<template>
  <MarqueeRoot v-model:playing="playing" direction="left" :speed="60" aria-label="Partners">
    <template #default="{ state }">
      <MarqueePauseButton>
        <template #default="{ label }">{{ label }}</template>
      </MarqueePauseButton>
      <MarqueeContent>
        <template #default="{ copy }">
          <span :data-copy="copy">Acme · Globex · Initech · {{ state }}</span>
        </template>
      </MarqueeContent>
    </template>
  </MarqueeRoot>
</template>
`,
  },
] as const;
