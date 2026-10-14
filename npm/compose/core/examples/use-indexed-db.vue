<script setup lang="ts">
import { useIndexedDB } from "@vizejs/composable/use-indexed-db";

const { state: notes, ready } = useIndexedDB("notes", "", { database: "docs-example" });
</script>

<template>
  <div class="composable-example">
    <p>Keep a longer draft in IndexedDB so it survives a reload.</p>
    <label>
      Notes
      <textarea v-model="notes" rows="3" :disabled="!ready"></textarea>
    </label>
    <output aria-live="polite">{{
      ready ? `${notes.length} characters saved in this browser.` : "Opening the database…"
    }}</output>
  </div>
</template>
