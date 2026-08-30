export const avatarRendererFixtures = [
  {
    filename: "AvatarConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Avatar } from "./avatar.ts";

const status = ref<"away" | "busy" | "none" | "offline" | "online">("online");
</script>

<template>
  <Avatar
    alt="Aki Kimura"
    fallback="AK"
    loading="lazy"
    name="Aki Kimura"
    src="/avatars/aki.png"
    :status
  >
    <template #fallback="{ fallback, name, status }">
      <span>{{ fallback }} {{ name }} {{ status }}</span>
    </template>
  </Avatar>
</template>
`,
  },
] as const;
