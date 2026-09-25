export const avatarGroupRendererFixtures = [
  {
    filename: "AvatarGroupConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { Avatar } from "./families/layout/avatar/avatar.ts";
import { AvatarGroup, AvatarGroupOverflow } from "./families/layout/avatar-group/avatar-group.ts";

interface Member {
  readonly id: string;
  readonly name: string;
}

const members: readonly Member[] = [
  { id: "ada", name: "Ada" },
  { id: "grace", name: "Grace" },
  { id: "linus", name: "Linus" },
];
</script>

<template>
  <AvatarGroup :items="members" :max="2" :spacing="-8" aria-label="Members">
    <template #item="{ item }">
      <Avatar :name="item.name" :fallback="item.name.charAt(0)" />
    </template>
    <template #overflow="{ count, hiddenItems }">
      <AvatarGroupOverflow :count="count" :items="hiddenItems">
        <template #default="{ text }">{{ text }}</template>
      </AvatarGroupOverflow>
    </template>
  </AvatarGroup>
</template>
`,
  },
] as const;
