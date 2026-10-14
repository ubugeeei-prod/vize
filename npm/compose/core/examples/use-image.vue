<script setup lang="ts">
import { ref } from "vue";
import { useImage } from "@vizejs/composable/use-image";

const avatars = {
  Valid: `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="#42b883"/></svg>')}`,
  Broken: "data:image/png;base64,AAAA",
};
const choice = ref<keyof typeof avatars>("Valid");
const { status, image } = useImage(() => ({ src: avatars[choice.value] }));
</script>

<template>
  <div class="composable-example">
    <p>Show a placeholder until an avatar loads, and a fallback if it fails.</p>
    <label>
      Avatar source
      <select v-model="choice">
        <option>Valid</option>
        <option>Broken</option>
      </select>
    </label>
    <img v-if="status === 'loaded'" :src="avatars[choice]" alt="Avatar" width="64" height="64" />
    <output aria-live="polite">{{
      status === "loaded"
        ? `Loaded ${(image as HTMLImageElement | undefined)?.naturalWidth} × ${(image as HTMLImageElement | undefined)?.naturalHeight}`
        : status === "error"
          ? "Could not load the avatar; showing initials instead."
          : "Loading avatar…"
    }}</output>
  </div>
</template>
