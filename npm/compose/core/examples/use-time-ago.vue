<script setup lang="ts">
import { ref } from "vue";
import { useTimeAgo } from "@vizejs/composable/use-time-ago";

const savedAt = ref<number>();
const timeAgo = useTimeAgo(() => savedAt.value ?? Date.now(), { updateIntervalMs: 1000 });

function save() {
  savedAt.value = Date.now();
}
</script>

<template>
  <div class="composable-example">
    <p>Tell the reader how long ago a draft was saved, updating as time passes.</p>
    <button type="button" @click="save">Save draft</button>
    <output aria-live="polite">{{
      savedAt === undefined ? "Not saved yet." : `Saved ${timeAgo}`
    }}</output>
  </div>
</template>
