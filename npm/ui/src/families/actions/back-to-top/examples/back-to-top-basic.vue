<!-- Back-to-top button that appears once a scrollable article passes the threshold and returns focus to it. -->
<script setup lang="ts">
import { useId, useTemplateRef } from "vue";

import { BackToTop } from "../back-to-top.ts";

const titleId = useId();
const article = useTemplateRef<HTMLElement>("article");
// The article scrolls on its own, so it needs a fixed block size.
const scrollerSize = { blockSize: "12rem", overflow: "auto" };
const sections = ["Installation", "Configuration", "Components", "Theming", "Testing", "Deploying"];
</script>

<template>
  <div>
    <section ref="article" :aria-labelledby="titleId" :style="scrollerSize">
      <h2 :id="titleId">Getting started</h2>
      <p v-for="section in sections" :key="section">
        {{ section }}: read this chapter before moving on to the next one.
      </p>
    </section>
    <BackToTop :target="article" :threshold="120" />
  </div>
</template>
