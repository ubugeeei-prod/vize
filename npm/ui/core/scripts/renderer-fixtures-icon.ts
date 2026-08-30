export const iconRendererFixtures = [
  {
    filename: "IconConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { Icon } from "./families/layout/icon/icon.ts";
</script>

<template>
  <Icon title="Search" description="Search current workspace" size="sm">
    <path d="M10 18a8 8 0 1 1 5.66-2.34L21 21" />
  </Icon>
</template>
`,
  },
  {
    filename: "IconButtonConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { Icon } from "./families/layout/icon/icon.ts";
import { IconButton } from "./families/layout/icon/icon-button.ts";
</script>

<template>
  <IconButton aria-label="Refresh feed" size="sm" tone="accent" variant="soft">
    <Icon aria-hidden size="sm">
      <path d="M4 12h16" />
    </Icon>
  </IconButton>
</template>
`,
  },
] as const;
