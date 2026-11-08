<script setup lang="ts">
import FrescoStack from "./FrescoStack.vue";
import type { ConvenienceStackProps } from "./layout-primitive-types.js";

const { gap = 0, align, justify } = defineProps<ConvenienceStackProps>();

defineOptions({ name: "FrescoVStack" });
</script>

<template>
  <FrescoStack direction="vertical" :gap :align :justify><slot /></FrescoStack>
</template>
