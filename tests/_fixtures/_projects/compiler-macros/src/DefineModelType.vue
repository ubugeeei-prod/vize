<script setup lang="ts">
const modelValue = defineModel<string>();
const count = defineModel<number>("count");
console.log(modelValue.value, count.value);
</script>

<template>
  <input :value="modelValue" />
  <span>{{ count }}</span>
</template>
