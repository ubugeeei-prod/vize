export const landmarkRendererFixtures = [
  {
    filename: "LandmarkConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { Landmark, LandmarkProvider } from "./families/accessibility/landmark/landmark.ts";
</script>

<template>
  <LandmarkProvider discover @navigate="(landmark) => void landmark.role">
    <Landmark role="banner">Site header</Landmark>
    <Landmark v-slot="{ focused }" role="navigation" aria-label="Primary">
      Primary navigation {{ focused ? "(focused)" : "" }}
    </Landmark>
    <Landmark role="main">Main content</Landmark>
    <Landmark role="contentinfo">Footer</Landmark>
  </LandmarkProvider>
</template>
`,
  },
] as const;
