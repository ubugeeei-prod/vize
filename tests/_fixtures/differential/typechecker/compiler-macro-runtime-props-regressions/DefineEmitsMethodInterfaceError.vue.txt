<script setup lang="ts">
interface Emits {
  submit(payload: string, force?: boolean): void;
  close(): void;
}

const emit = defineEmits<Emits>();

function handleSubmit() {
  emit("submit", 123);
  emit("close", true);
}
</script>

<template>
  <button @click="handleSubmit">Submit</button>
</template>
