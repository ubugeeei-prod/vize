<script setup lang="ts">
import FrescoStack from "./FrescoStack.vue";
import type { ConvenienceStackProps } from "./layout-primitive-types.js";

const { gap = 1, align, justify } = defineProps<ConvenienceStackProps>();

defineOptions({ name: "FrescoHStack" });
</script>

<template>
  <FrescoStack direction="horizontal" :gap :align :justify><slot /></FrescoStack>
</template>
