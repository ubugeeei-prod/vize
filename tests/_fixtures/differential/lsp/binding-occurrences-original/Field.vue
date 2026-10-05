<script setup lang="ts">
const label = "Name";
const hint = "Shown below the field";
const id = "field";
</script>

<template>
  <label :for="id">{{ label }}</label>
  <input :id="id" />
  <p :id="`${id}-hint`">{{ hint }}</p>
</template>
