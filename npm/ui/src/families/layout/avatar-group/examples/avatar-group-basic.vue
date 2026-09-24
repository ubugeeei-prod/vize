<!-- Reviewer stack capped at four tiles, collapsing the rest into an accessible "+N more" tile. -->
<script setup lang="ts">
import { AvatarGroup } from "../avatar-group.ts";

interface Reviewer {
  readonly name: string;
  readonly initials: string;
}

const reviewers: readonly Reviewer[] = [
  { name: "Ada Lovelace", initials: "AL" },
  { name: "Grace Hopper", initials: "GH" },
  { name: "Linus Torvalds", initials: "LT" },
  { name: "Barbara Liskov", initials: "BL" },
  { name: "Alan Kay", initials: "AK" },
  { name: "Margaret Hamilton", initials: "MH" },
];
</script>

<template>
  <AvatarGroup :items="reviewers" :max="4" :spacing="-8" aria-label="Reviewers">
    <template #item="{ item }">
      <span role="img" :aria-label="item.name">{{ item.initials }}</span>
    </template>
  </AvatarGroup>
</template>
