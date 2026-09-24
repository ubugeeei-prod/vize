<!-- Product tour opened from a button that walks through two template-ref targets. -->
<script setup lang="ts">
import { computed, ref, useTemplateRef } from "vue";

import {
  TourArrow,
  TourClose,
  TourContent,
  TourDescription,
  TourNext,
  TourPrev,
  TourProgress,
  TourRoot,
  TourTitle,
} from "../tour.ts";

const searchField = useTemplateRef<HTMLInputElement>("search");
const newButton = useTemplateRef<HTMLButtonElement>("create");
const steps = [
  { value: "search", target: searchField, title: "Search", text: "Find any project by name." },
  { value: "create", target: newButton, title: "Create", text: "Start a new project here." },
];
const open = ref(false);
const step = ref("search");
const current = computed(() => steps.find((item) => item.value === step.value) ?? steps[0]);
</script>

<template>
  <TourRoot v-model:open="open" v-model:step="step" :steps>
    <input ref="search" type="search" aria-label="Search projects" />
    <button ref="create" type="button">New project</button>
    <button type="button" @click="() => (open = true)">Start tour</button>
    <TourContent>
      <TourTitle>{{ current?.title }}</TourTitle>
      <TourDescription>{{ current?.text }}</TourDescription>
      <TourProgress />
      <TourPrev>Back</TourPrev>
      <TourNext>Next</TourNext>
      <TourClose>Skip tour</TourClose>
      <TourArrow />
    </TourContent>
  </TourRoot>
</template>
