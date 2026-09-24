<!-- Section navigation that pins to the top of a scrollable article and reports when it is stuck. -->
<script setup lang="ts">
import { useId, useTemplateRef } from "vue";

import { Sticky } from "../sticky.ts";

const titleId = useId();
const article = useTemplateRef<HTMLElement>("article");
// The article scrolls on its own, so it needs a fixed block size.
const scrollerSize = { blockSize: "14rem", overflow: "auto" };
const chapters = ["Overview", "Setup", "Usage", "Recipes", "FAQ"];
</script>

<template>
  <section ref="article" :aria-labelledby="titleId" :style="scrollerSize">
    <h2 :id="titleId">Handbook</h2>
    <Sticky v-slot="{ stuck }" as="nav" :root="article" :offset="0" aria-label="Chapters">
      <a href="/handbook/overview">Overview</a>
      <a href="/handbook/setup">Setup</a>
      <a href="/handbook/usage">Usage</a>
      <span v-if="stuck">(pinned)</span>
    </Sticky>
    <p v-for="chapter in chapters" :key="chapter">
      {{ chapter }}: this chapter is long enough to scroll the navigation out of its place.
    </p>
  </section>
</template>
