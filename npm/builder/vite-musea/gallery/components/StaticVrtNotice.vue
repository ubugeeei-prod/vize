<script setup lang="ts">
import { joinBasePath } from "../staticApi";

const galleryUrl = new URL(joinBasePath("/"), window.location.href).href;
const command = `vp exec musea-vrt --gallery-url '${galleryUrl.replaceAll("'", "'\"'\"'")}'`;
</script>

<template>
  <div class="vrt-empty">
    <p>Capture this hosted gallery with Node and Playwright:</p>
    <p class="vrt-command">
      <code>{{ command }}</code>
    </p>
    <p class="vrt-hint">Baselines, current captures, diffs, and reports are saved locally.</p>
  </div>
</template>

<style scoped>
.vrt-empty {
  padding: 2rem;
  text-align: center;
  color: var(--musea-text-muted);
  font-size: 0.875rem;
}
.vrt-command {
  overflow-wrap: anywhere;
}
.vrt-hint {
  font-size: 0.75rem;
  margin-top: 0.5rem;
  opacity: 0.7;
}
</style>
