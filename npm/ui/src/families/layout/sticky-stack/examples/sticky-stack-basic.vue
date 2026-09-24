<!-- App bar and section tabs that stay pinned one below the other while the page scrolls. -->
<script setup lang="ts">
import { StickyStack, StickyStackItem } from "../sticky-stack.ts";

const sections = ["Overview", "Pricing", "Reviews", "Support"];
</script>

<template>
  <StickyStack :offset="0">
    <StickyStackItem as="header" :estimated-height="56">
      <strong>Acme Store</strong>
    </StickyStackItem>
    <StickyStackItem as="nav" :estimated-height="40" aria-label="Sections">
      <a v-for="section in sections" :key="section" :href="`#${section.toLowerCase()}`">
        {{ section }}
      </a>
    </StickyStackItem>
    <section v-for="section in sections" :key="section" :aria-label="section">
      <h2>{{ section }}</h2>
      <p>Details about {{ section.toLowerCase() }} for this product.</p>
    </section>
  </StickyStack>
</template>
