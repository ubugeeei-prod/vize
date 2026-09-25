<!-- Raised section surface labelled and described by its own heading and summary. -->
<script setup lang="ts">
import { useId } from "vue";

import { Surface } from "../surface.ts";

const titleId = useId();
const summaryId = useId();
</script>

<template>
  <Surface elevation="raised" tone="info" :aria-labelledby="titleId" :aria-describedby="summaryId">
    <h2 :id="titleId">Billing</h2>
    <p :id="summaryId">Your next invoice is issued on the first of the month.</p>
    <button type="button">Manage plan</button>
  </Surface>
</template>
