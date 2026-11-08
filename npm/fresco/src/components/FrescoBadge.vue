<script setup lang="ts">
import { computed } from "vue";

import { badgeVariantColors, type BadgeProps } from "./display-primitive-types.js";

const { label, variant = "default", border = false, fg, bg } = defineProps<BadgeProps>();

defineOptions({ name: "Badge" });

const colors = computed(() => badgeVariantColors[variant]);
const foreground = computed(() => fg ?? colors.value.fg);
const background = computed(() => bg ?? colors.value.bg);
const borderStyle = { paddingLeft: 1, paddingRight: 1 } as const;
</script>

<template>
  <div v-if="border" border="single" :fg="foreground" :bg="background" :style="borderStyle">
    <text :fg="foreground" :bg="background">{{ label }}</text>
  </div>
  <text v-else :fg="foreground" :bg="background" :bold="true">[{{ label }}]</text>
</template>
