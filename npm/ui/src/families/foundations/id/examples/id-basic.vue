<!-- Hydration-stable IDs wiring a label and hint to an email field. -->
<script setup lang="ts">
import { computed } from "vue";

import { deriveDeterministicId, useDeterministicId } from "../id.ts";

const inputId = useDeterministicId({ hint: "email" });
const hintId = computed(() => deriveDeterministicId(inputId.value, "hint"));
</script>

<template>
  <div>
    <label :for="inputId">Email address</label>
    <input :id="inputId" type="email" autocomplete="email" :aria-describedby="hintId" />
    <p :id="hintId">We only use this address for order receipts.</p>
  </div>
</template>
