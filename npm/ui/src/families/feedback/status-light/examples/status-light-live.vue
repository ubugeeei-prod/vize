<!-- Service health light that announces state changes through a polite status role. -->
<script setup lang="ts">
import { computed, ref, useId } from "vue";

import { StatusLight } from "../status-light.ts";

const healthy = ref(true);
const labelId = useId();
const label = computed(() => (healthy.value ? "API operational" : "API offline"));
</script>

<template>
  <div>
    <StatusLight
      role="status"
      :state="healthy ? 'online' : 'offline'"
      :tone="healthy ? 'success' : 'danger'"
      :aria-labelledby="labelId"
    />
    <span :id="labelId">{{ label }}</span>
    <button type="button" @click="() => (healthy = !healthy)">Simulate outage</button>
  </div>
</template>
