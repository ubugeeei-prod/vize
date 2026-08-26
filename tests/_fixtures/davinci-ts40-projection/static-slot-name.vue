<script setup lang="ts">
const label = "ready"
</script>
<template>
  <Widget>
    <template #default="{ item }">
      {{ item.label }}
    </template>
    <template #static-header="{ active }">
      {{ active ? label : "idle" }}
    </template>
  </Widget>
</template>
