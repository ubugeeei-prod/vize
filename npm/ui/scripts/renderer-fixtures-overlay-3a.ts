export const overlay3aRendererFixtures = [
  {
    filename: "NotificationCenterConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  NotificationCenterEmpty,
  NotificationCenterItem,
  NotificationCenterList,
  NotificationCenterRoot,
  NotificationCenterTrigger,
  createNotificationStore,
} from "./families/feedback/notification-center/notification-center.ts";

const store = createNotificationStore<{ readonly href: string }>({ now: () => 0 });
store.add({ title: "Build passed", data: { href: "/builds/1" } });
</script>

<template>
  <NotificationCenterRoot id="renderer-inbox" :store>
    <NotificationCenterTrigger v-slot="{ unreadCount }">Bell {{ unreadCount }}</NotificationCenterTrigger>
    <NotificationCenterList busy>
      <template #item="{ notification, position, setSize }">
        <NotificationCenterItem :notification :position :set-size="setSize" />
      </template>
      <NotificationCenterEmpty>All caught up</NotificationCenterEmpty>
    </NotificationCenterList>
  </NotificationCenterRoot>
</template>
`,
  },
  {
    filename: "PopconfirmConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  PopconfirmCancel,
  PopconfirmConfirm,
  PopconfirmContent,
  PopconfirmRoot,
  PopconfirmTrigger,
} from "./families/overlays/popconfirm/popconfirm.ts";

async function remove(): Promise<void> {
  await Promise.resolve();
}
</script>

<template>
  <PopconfirmRoot id="renderer-popconfirm" @confirm="remove">
    <PopconfirmTrigger>Delete</PopconfirmTrigger>
    <PopconfirmContent portal-disabled title="Delete this file?" description="This cannot be undone.">
      <PopconfirmCancel>Keep</PopconfirmCancel>
      <PopconfirmConfirm v-slot="{ pending }">{{ pending ? "Deleting" : "Delete" }}</PopconfirmConfirm>
    </PopconfirmContent>
  </PopconfirmRoot>
</template>
`,
  },
] as const;
