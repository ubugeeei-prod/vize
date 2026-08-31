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

const progressBarRendererFixtures = [
  {
    filename: "ProgressBarConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { ProgressBar } from "./families/feedback/progress-bar/progress-bar.ts";

const value = ref(40);
</script>

<template>
  <ProgressBar
    aria-label="Upload progress"
    dir="rtl"
    :min="20"
    :max="100"
    :value
    value-label="25%"
  >
    <template #indicator="{ percent, state }">
      <span>{{ state }} {{ percent }}</span>
    </template>
  </ProgressBar>
</template>
`,
  },
] as const;

const progressRendererFixtures = [
  {
    filename: "ProgressConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Progress } from "./families/feedback/progress/progress.ts";

const value = ref(40);
</script>

<template>
  <Progress aria-label="Upload progress" :max="100" :value>
    <template #default="{ percent, state }">
      <span>{{ state }} {{ percent }}</span>
    </template>
  </Progress>
</template>
`,
  },
] as const;

export const feedbackRendererFixtures = [
  ...bannerRendererFixtures,
  ...calloutRendererFixtures,
  ...progressBarRendererFixtures,
  ...progressRendererFixtures,
  ...statusLightRendererFixtures,
] as const;
