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

const takeCodePoints = (value: string, count: number) => Array.from(value).slice(0, count).join("");
const displayInitials = computed(() => {
  if (initials) return initials;
  const parts = name?.split(" ").filter(Boolean) ?? [];
  if (parts.length >= 2) {
    return `${takeCodePoints(parts[0], 1)}${takeCodePoints(parts[1], 1)}`.toUpperCase();
  }
  if (parts.length === 1) return takeCodePoints(parts[0], 2).toUpperCase();
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
