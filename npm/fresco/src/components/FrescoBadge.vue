<script setup lang="ts">
import { computed } from "vue";

import { badgeVariantColors, type BadgeProps } from "./display-primitive-types.js";

const props = withDefaults(defineProps<BadgeProps>(), {
  variant: "default",
  border: false,
});

defineOptions({ name: "Badge" });

const colors = computed(() => badgeVariantColors[props.variant]);
const foreground = computed(() => props.fg ?? colors.value.fg);
const background = computed(() => props.bg ?? colors.value.bg);
const borderStyle = { paddingLeft: 1, paddingRight: 1 } as const;
</script>

<template>
  <div v-if="border" border="single" :fg="foreground" :bg="background" :style="borderStyle">
    <text :fg="foreground" :bg="background">{{ label }}</text>
  </div>
  <text v-else :fg="foreground" :bg="background" :bold="true">[{{ label }}]</text>
</template>
