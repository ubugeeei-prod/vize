<script setup lang="ts">
import { resolveComponent } from "vue";

const completion = 0.72;
const RegisteredExternalMeter = resolveComponent("ExternalMeter");
</script>

<template>
  <div :style="{ flexDirection: 'column' }">
    <text text="External provider" bold />
    <RegisteredExternalMeter :value="completion" :width="24" />
  </div>
</template>
