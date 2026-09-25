<!-- Polite status spinner with an accessible name for pending work. -->
<script setup lang="ts">
import { Spinner } from "../spinner.ts";
</script>

<template>
  <Spinner aria-label="Saving changes">Saving…</Spinner>
</template>
