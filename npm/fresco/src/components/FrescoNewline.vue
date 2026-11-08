<script setup lang="ts">
import { computed } from "vue";

import type { NewlineProps } from "./text-utility-types.js";

const { count = 1 } = defineProps<NewlineProps>();

defineOptions({ name: "Newline" });

const content = computed(() => "\n".repeat(Math.max(0, count)));
</script>

<template>
  <text :text="content" />
</template>
