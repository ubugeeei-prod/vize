<script setup lang="ts">
const completion = 0.72;
</script>

<template>
  <div :style="{ flexDirection: 'column' }">
    <text text="External provider" bold />
    <ExternalMeter :value="completion" :width="24" />
  </div>
</template>
