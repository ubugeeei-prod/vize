<!-- Controlled swipeable pager whose segmented tabs and scroll-snap pages stay in sync. -->
<script setup lang="ts">
import { ref } from "vue";

import { Pager, PagerPage, PagerTab, PagerTabList, PagerViewport } from "../pager.ts";

type Library = "photos" | "albums" | "shared";

const pages: readonly Library[] = ["photos", "albums", "shared"];
const labels: Record<Library, string> = { photos: "Photos", albums: "Albums", shared: "Shared" };
const active = ref<Library>("photos");
// Swiping needs a horizontal scroll-snap viewport with one full-width page per snap point.
const viewportLayout = { display: "flex", overflowX: "auto", scrollSnapType: "x mandatory" };
const pageLayout = { flex: "0 0 100%", scrollSnapAlign: "start" };
</script>

<template>
  <Pager v-model="active" :pages>
    <PagerTabList aria-label="Library">
      <PagerTab v-for="page in pages" :key="page" :page>{{ labels[page] }}</PagerTab>
    </PagerTabList>
    <PagerViewport :style="viewportLayout">
      <PagerPage v-for="page in pages" :key="page" :page :style="pageLayout">
        <p>{{ labels[page] }} in your library.</p>
      </PagerPage>
    </PagerViewport>
  </Pager>
</template>
