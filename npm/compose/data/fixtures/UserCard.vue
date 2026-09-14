<script setup lang="ts">
import { defineDataResource } from "../src/index.ts";

export const userResource = defineDataResource({
  key: "user.profile",
  source: "./UserCard.vue",
  async loader({ input }: { input: { id: string } }) {
    return {
      id: input.id,
      name: "Ada",
    };
  },
});
</script>

<template>
  <section>{{ userResource.key }}</section>
</template>
