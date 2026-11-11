<script setup lang="ts">
import type { PickEmits, PickProps } from "lib/types";

const { row_id } = defineProps<PickProps>();
const emit = defineEmits<PickEmits>();
</script>

<template>
  <button type="button" @click="emit('pik', row_id)" />
</template>
