<script setup lang="ts">
import { ref } from "vue";
import ChildButton from "./ChildButton.vue";

const emit = defineEmits<{ retry: [] }>();
const selected = ref<string[]>([]);
function reset(): void {
  selected.value = [];
}
</script>

<template>
  <button type="button" @click="emit('retry')">a</button>
  <button type="button" @click="() => emit('retry')">b</button>
  <button type="button" @click="selected = []">c</button>
  <ChildButton label="d" @click="reset" />
  <ChildButton label="e" @click="reset()" />
  <button type="button" @click="() => (selected = [])">f</button>
  <ChildButton label="g" @click="() => (selected = [])" />
  <ChildButton label="h" @click="() => reset()" />
  <ChildButton label="i" @click="() => emit('retry')" />
</template>
