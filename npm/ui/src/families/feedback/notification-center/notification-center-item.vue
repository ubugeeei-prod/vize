<script setup lang="ts" generic="Data = unknown">
import { computed, useTemplateRef } from "vue";

import { deriveDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { notificationCenterContext } from "./notification-center-context.ts";
import type {
  NotificationCenterItemExpose,
  NotificationCenterItemSlotState,
  NotificationReadState,
  NotificationRecord,
} from "./notification-center-types.ts";

const {
  notification,
  position,
  setSize,
  markReadOnActivate = true,
} = defineProps<{
  /**
   * Notification snapshot to render.
   *
   * @default required
   */
  readonly notification: NotificationRecord<Data>;

  /**
   * One-based position in the feed (`aria-posinset`).
   *
   * @default required
   */
  readonly position: number;

  /**
   * Feed size (`aria-setsize`); `-1` when unknown.
   *
   * @default required
   */
  readonly setSize: number;

  /**
   * Mark the notification read when the article itself is clicked or receives Enter.
   *
   * @default true
   */
  readonly markReadOnActivate?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the article is activated by click or Enter. */
  activate: [notification: NotificationRecord<Data>, nativeEvent: Event];
}>();

defineSlots<{
  /** Article contents. Put `titleId`/`descriptionId` on the visible headline and text. */
  default?(props: NotificationCenterItemSlotState<Data>): unknown;
}>();

const context = notificationCenterContext.use();
const element = useTemplateRef<HTMLElement>("element");
const articleId = computed(() => context.itemId(notification.id));
const titleId = computed(() => deriveDeterministicId(articleId.value, "title"));
const descriptionId = computed(() => deriveDeterministicId(articleId.value, "description"));
const readState = computed<NotificationReadState>(() => (notification.read ? "read" : "unread"));
const slotState = computed<NotificationCenterItemSlotState<Data>>(() => ({
  archive: () => context.store.archive(notification.id),
  descriptionId: descriptionId.value,
  markRead: () => context.store.markRead(notification.id),
  markUnread: () => context.store.markUnread(notification.id),
  notification,
  position,
  remove: () => context.store.remove(notification.id),
  setSize,
  titleId: titleId.value,
}));

function activate(event: Event): void {
  emit("activate", notification, event);
  if (markReadOnActivate) context.store.markRead(notification.id);
}

function onClick(event: MouseEvent): void {
  if (
    event.target instanceof Element &&
    event.target.closest("a, button, input, select, textarea")
  ) {
    return;
  }
  activate(event);
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== "Enter" || event.target !== element.value || event.defaultPrevented) return;
  event.preventDefault();
  activate(event);
}

const articleHandlers = { onClick, onKeydown } as const;

type NotificationCenterItemSetupExpose = Omit<NotificationCenterItemExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies NotificationCenterItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <article
    :id="articleId"
    ref="element"
    tabindex="0"
    :aria-posinset="position"
    :aria-setsize="setSize"
    :aria-labelledby="titleId"
    :aria-describedby="notification.description === null ? undefined : descriptionId"
    data-vize-ui="notification-center-item"
    part="item"
    :data-read="readState"
    :data-type="notification.type"
    :data-group="notification.group ?? undefined"
    v-bind="articleHandlers"
  >
    <slot v-bind="slotState">
      <strong :id="titleId" part="item-title">{{ notification.title }}</strong>
      <p v-if="notification.description !== null" :id="descriptionId" part="item-description">
        {{ notification.description }}
      </p>
    </slot>
  </article>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
