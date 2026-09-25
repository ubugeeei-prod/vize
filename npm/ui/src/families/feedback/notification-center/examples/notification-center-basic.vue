<!-- Bell trigger that discloses a notification feed where each entry can be marked read. -->
<script setup lang="ts">
import {
  NotificationCenter,
  NotificationCenterEmpty,
  NotificationCenterItem,
  NotificationCenterList,
  NotificationCenterTrigger,
} from "../notification-center.ts";
import type { NotificationInput } from "../notification-center.ts";

const initial: readonly NotificationInput[] = [
  { id: "build", title: "Build passed", description: "main is green again.", createdAt: 2 },
  {
    id: "review",
    title: "Review requested",
    description: "Ada asked you to review #42.",
    createdAt: 1,
  },
];

// A fixed clock keeps server and client markup identical.
const now = (): number => 3;
</script>

<template>
  <NotificationCenter :initial :now label="Notifications">
    <NotificationCenterTrigger>Notifications</NotificationCenterTrigger>
    <NotificationCenterList>
      <template #item="{ notification, position, setSize }">
        <NotificationCenterItem
          v-slot="{ titleId, descriptionId, markRead }"
          :notification
          :position
          :set-size
        >
          <strong :id="titleId">{{ notification.title }}</strong>
          <p :id="descriptionId">{{ notification.description }}</p>
          <button v-if="!notification.read" type="button" @click="markRead">Mark as read</button>
        </NotificationCenterItem>
      </template>
      <NotificationCenterEmpty>You're all caught up.</NotificationCenterEmpty>
    </NotificationCenterList>
  </NotificationCenter>
</template>
