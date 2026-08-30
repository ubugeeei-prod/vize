import { statusLightRendererFixtures } from "./renderer-fixtures-status-light.ts";

const bannerRendererFixtures = [
  {
    filename: "BannerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Banner } from "./families/feedback/banner/banner.ts";

const open = ref(true);
</script>

<template>
  <Banner v-model:open="open" dismissible role="status" tone="info">
    <template #title="{ live, state }">
      Deploy {{ state }} {{ live }}
    </template>
    <template #actions="{ dismiss }">
      <button type="button" @click="dismiss()">Dismiss</button>
    </template>
  </Banner>
</template>
`,
  },
] as const;

const calloutRendererFixtures = [
  {
    filename: "CalloutConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Callout } from "./families/feedback/callout/callout.ts";

const role = ref<"note" | "status">("status");
</script>

<template>
  <Callout role="status" tone="info" density="compact" aria-label="Sync notice">
    <template #icon="{ tone }">
      <span>{{ tone }}</span>
    </template>
    <template #title="{ role: currentRole }">
      {{ role }} {{ currentRole }}
    </template>
    <template #description="{ live }">
      {{ live }}
    </template>
    <template #default="{ state }">
      <span>{{ state }}</span>
    </template>
    <template #actions>
      <button type="button">Review</button>
    </template>
  </Callout>
</template>
`,
  },
] as const;

export const feedbackRendererFixtures = [
  ...bannerRendererFixtures,
  ...calloutRendererFixtures,
  ...statusLightRendererFixtures,
] as const;
