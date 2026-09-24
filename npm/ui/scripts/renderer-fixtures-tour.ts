export const tourRendererFixtures = [
  {
    filename: "TourConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  TourArrow,
  TourClose,
  TourContent,
  TourDescription,
  TourNext,
  TourPrev,
  TourProgress,
  TourRoot,
  TourSpotlight,
  TourStep,
  TourTitle,
} from "./families/overlays/tour/tour.ts";

const searchTarget = ref<HTMLElement | null>(null);
const open = ref(false);
const step = ref<"welcome" | "search">("welcome");
const steps = [
  { value: "welcome", title: "Welcome" },
  { value: "search", title: "Search", target: searchTarget, placement: "bottom-start" },
] as const;
</script>

<template>
  <input ref="searchTarget" aria-label="Search" />
  <button type="button" @click="open = true">Start tour</button>
  <TourRoot v-model:open="open" v-model:step="step" :steps missing-target="skip">
    <template #default="{ step: current }">
      <TourSpotlight :padding="6" :radius="8" />
      <TourContent :close-on-pointer-down-outside="false">
        <TourTitle>{{ current?.title }}</TourTitle>
        <TourDescription>Take a quick look around.</TourDescription>
        <TourStep value="search">Type to search anything.</TourStep>
        <TourProgress v-slot="{ current: position, total }">{{ position }} of {{ total }}</TourProgress>
        <TourPrev>Back</TourPrev>
        <TourNext v-slot="{ last }">{{ last ? "Finish" : "Next" }}</TourNext>
        <TourClose reason="skip">Skip tour</TourClose>
        <TourArrow />
      </TourContent>
    </template>
  </TourRoot>
</template>
`,
  },
] as const;
