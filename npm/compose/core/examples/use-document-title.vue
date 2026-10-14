<script setup lang="ts">
import { computed, ref } from "vue";
import { useDocumentTitle } from "@vizejs/composable/use-document-title";

const unread = ref(0);
useDocumentTitle(computed(() => (unread.value > 0 ? `(${unread.value}) Inbox` : "Inbox")));
</script>

<template>
  <div class="composable-example">
    <p>Show the unread count in the browser tab title.</p>
    <div class="example-actions">
      <button type="button" @click="() => (unread += 1)">Receive message</button>
      <button type="button" @click="() => (unread = 0)">Mark all read</button>
    </div>
    <output aria-live="polite">{{ unread }} unread</output>
  </div>
</template>
