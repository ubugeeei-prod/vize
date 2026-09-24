<!-- Determinate progress bar with a visible label and value text. -->
<script setup lang="ts">
import { ProgressBar } from "../progress-bar.ts";
</script>

<template>
  <ProgressBar :value="64" label="Syncing library" value-label="64 of 100 tracks" />
</template>
