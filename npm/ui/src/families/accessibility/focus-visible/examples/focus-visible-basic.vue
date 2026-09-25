<!-- Buttons that show a focus ring only for keyboard focus through data-focus-visible. -->
<script setup lang="ts">
import { FocusVisibleProvider } from "../focus-visible.ts";
</script>

<template>
  <FocusVisibleProvider>
    <button type="button">Save</button>
    <button type="button">Publish</button>
  </FocusVisibleProvider>
</template>
