<!-- Indeterminate progress bar for work whose duration is unknown. -->
<script setup lang="ts">
import { ProgressBar } from "../progress-bar.ts";
</script>

<template>
  <ProgressBar :value="null" label="Preparing export" value-label="Waiting for the server" />
</template>
