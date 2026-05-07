<script setup lang="ts">
import { ref, onMounted, useTemplateRef } from "vue";

const inputRef = useTemplateRef<HTMLInputElement>("input");
const count = ref(0);

onMounted(() => {
  if (inputRef.value) {
    const num: number = inputRef.value.value;
    inputRef.value.nonExistentMethod();
  }
});
</script>

<template>
  <input ref="input" />
  <span>{{ count }}</span>
</template>
