<!-- Polite success status with application-owned dismissal and a labelled heading. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { Alert } from "../alert.ts";

const open = ref(true);
const titleId = useId();
</script>

<template>
  <div>
    <Alert role="status" variant="success" :open :aria-labelledby="titleId">
      <strong :id="titleId">Changes saved</strong>
      <p>Your profile was updated.</p>
      <button type="button" @click="() => (open = false)">Dismiss</button>
    </Alert>
    <button v-if="!open" type="button" @click="() => (open = true)">Show message again</button>
  </div>
</template>
