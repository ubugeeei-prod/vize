<script setup lang="ts">
import { Counter } from "../../ui/src/index";

defineProps<{ initial: number }>();
</script>

<template>
  <Counter :count="initial" />
</template>
