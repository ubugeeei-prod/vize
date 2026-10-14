<script setup lang="ts">
import { useUrlHash } from "@vizejs/composable/use-url-hash";

const { state: section } = useUrlHash();
const sections = ["intro", "install", "usage"];
</script>

<template>
  <div class="composable-example">
    <p>Link to a section of the page through the URL hash.</p>
    <nav class="example-actions" aria-label="Sections">
      <button v-for="name in sections" :key="name" type="button" @click="() => (section = name)">
        Go to {{ name }}
      </button>
    </nav>
    <output aria-live="polite">{{
      section ? `Hash: #${section}` : "No section in the URL."
    }}</output>
  </div>
</template>
