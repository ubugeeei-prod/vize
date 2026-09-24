export const drawerRendererFixtures = [
  {
    filename: "DrawerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  DrawerClose,
  DrawerContent,
  DrawerDescription,
  DrawerHandle,
  DrawerRoot,
  DrawerTitle,
  DrawerTrigger,
} from "./families/overlays/drawer/drawer.ts";
import type { DrawerSnapPoint } from "./families/overlays/drawer/drawer.ts";

const snapPoints: readonly DrawerSnapPoint[] = [0.4, "320px", 1];
const activeSnapPoint = ref<DrawerSnapPoint | null>(0.4);
</script>

<template>
  <DrawerRoot id="renderer-drawer" v-model:active-snap-point="activeSnapPoint" :snap-points="snapPoints">
    <DrawerTrigger>Open drawer</DrawerTrigger>
    <DrawerContent v-slot="{ dragging }">
      <DrawerHandle />
      <DrawerTitle>Renderer drawer</DrawerTitle>
      <DrawerDescription>Compiled across every renderer lane.</DrawerDescription>
      <p :data-dragging="dragging || undefined">Snap {{ activeSnapPoint }}</p>
      <DrawerClose>Close</DrawerClose>
    </DrawerContent>
  </DrawerRoot>
</template>
`,
  },
  {
    filename: "ConfirmConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ConfirmProvider } from "./families/overlays/confirm/confirm.ts";
</script>

<template>
  <ConfirmProvider id="renderer-confirm" confirm-label="OK" cancel-label="Back" portal-disabled>
    <p>Application content</p>
    <template #content="{ request, resolve, cancel }">
      <h2>{{ request.title }}</h2>
      <button type="button" @click="() => resolve('confirm')">Yes</button>
      <button type="button" @click="cancel">No</button>
    </template>
  </ConfirmProvider>
</template>
`,
  },
] as const;
