<script setup lang="ts">
import { ref } from "vue";

const emit = defineEmits<{
  (e: "update", value: string): void;
  (e: "count", value: number): void;
}>();

const count = ref(0);

function handleClick() {
  emit("update", 123);
  emit("count", "not a number");
}
</script>

<template>
  <button @click="handleClick">Emit</button>
  <span>{{ count }}</span>
</template>
