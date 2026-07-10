<script setup lang="ts">
import { BadgeType } from "./types";

interface Props {
  badges: BadgeType[];
}

const props = defineProps<Props>();
</script>

<template>
  <div>{{ props.badges.length }}</div>
</template>
