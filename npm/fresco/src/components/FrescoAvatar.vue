<script setup lang="ts">
import { computed } from "vue";

import {
  avatarStatusColors,
  avatarStatusIcons,
  type AvatarProps,
} from "./presentational-primitive-types.js";

const {
  name,
  initials,
  size = "md",
  bg = "blue",
  fg = "white",
  border = true,
  status,
} = defineProps<AvatarProps>();

defineOptions({ name: "FrescoAvatar" });

const displayInitials = computed(() => {
  if (initials) return initials;
  const parts = name?.split(" ").filter(Boolean) ?? [];
  if (parts.length >= 2) return `${parts[0][0]}${parts[1][0]}`.toUpperCase();
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return "??";
});
const padding = computed(() => (size === "lg" ? 1 : 0));
const containerStyle = computed(() => ({
  flexDirection: "row" as const,
  paddingLeft: padding.value,
  paddingRight: padding.value,
}));
</script>

<template>
  <div :border="border ? 'rounded' : undefined" :bg :style="containerStyle">
    <text :fg :bg :bold="true">{{ displayInitials }}</text>
    <text v-if="status" :fg="avatarStatusColors[status]">{{ avatarStatusIcons[status] }}</text>
  </div>
</template>
