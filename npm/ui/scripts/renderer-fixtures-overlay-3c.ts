export const overlay3cRendererFixtures = [
  {
    filename: "StickyConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { Sticky } from "./families/layout/sticky/sticky.ts";
</script>

<template>
  <Sticky v-slot="{ stuck }" as="header" :offset="8" @stuck-change="(value) => void value">
    Toolbar {{ stuck ? "pinned" : "in flow" }}
  </Sticky>
</template>
`,
  },
  {
    filename: "BackToTopConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { BackToTop } from "./families/actions/back-to-top/back-to-top.ts";
</script>

<template>
  <BackToTop v-slot="{ visible }" :threshold="600" focus-target="#main" aria-label="Back to top">
    {{ visible ? "Top" : "" }}
  </BackToTop>
</template>
`,
  },
  {
    filename: "FloatingActionButtonConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  FloatingActionButton,
  SpeedDialAction,
  SpeedDialContent,
  SpeedDialRoot,
  SpeedDialTrigger,
} from "./families/actions/floating-action-button/floating-action-button.ts";
</script>

<template>
  <FloatingActionButton aria-label="Compose" placement="bottom-start">+</FloatingActionButton>
  <SpeedDialRoot direction="up" @select="(event) => void event.value">
    <SpeedDialTrigger aria-label="Create">+</SpeedDialTrigger>
    <SpeedDialContent>
      <SpeedDialAction value="note" label="New note">N</SpeedDialAction>
      <SpeedDialAction value="task" label="New task">T</SpeedDialAction>
    </SpeedDialContent>
  </SpeedDialRoot>
</template>
`,
  },
] as const;
