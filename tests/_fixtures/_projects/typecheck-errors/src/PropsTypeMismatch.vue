<script setup lang="ts">
import { ref } from "vue";

const props = defineProps<{
  title: string;
  count: number;
  disabled: boolean;
}>();

const doubled: string = props.count * 2;
const upper: number = props.title.toUpperCase();
const flag = ref(props.disabled);
</script>

<template>
  <div>{{ doubled }} {{ upper }} {{ flag }}</div>
</template>
