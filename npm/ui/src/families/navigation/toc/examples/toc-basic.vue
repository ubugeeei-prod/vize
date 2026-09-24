<!-- Table of contents that links to article headings and tracks the active section. -->
<script setup lang="ts">
import { useId } from "vue";

import { TocItem, TocLink, TocList, TocRoot } from "../toc.ts";

const sections = [
  { id: useId(), title: "Installation" },
  { id: useId(), title: "Configuration" },
  { id: useId(), title: "Deployment" },
];
</script>

<template>
  <div>
    <TocRoot aria-label="On this page">
      <TocList>
        <TocItem v-for="section in sections" :key="section.id" :target-id="section.id">
          <TocLink :target-id="section.id">{{ section.title }}</TocLink>
        </TocItem>
      </TocList>
    </TocRoot>
    <article>
      <section v-for="section in sections" :key="section.id">
        <h2 :id="section.id">{{ section.title }}</h2>
        <p>Details about {{ section.title.toLowerCase() }}.</p>
      </section>
    </article>
  </div>
</template>
