<script setup lang="ts">
import type { StatusBoxProps } from "./status-box.types";
defineProps<StatusBoxProps>();
</script>

<template>
  <div :class="`status-box--${status}`"></div>
</template>
