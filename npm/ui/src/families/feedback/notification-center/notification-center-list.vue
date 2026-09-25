<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import NotificationCenterItem from "./notification-center-item.vue";
import { notificationCenterContext } from "./notification-center-context.ts";
import type {
  NotificationCenterListExpose,
  NotificationRecord,
} from "./notification-center-types.ts";

/** Props handed to the `item` slot for each rendered notification. */
interface NotificationCenterListItemSlotProps {
  readonly notification: NotificationRecord<unknown>;
  readonly position: number;
  readonly setSize: number;
}

const {
  busy = false,
  markReadOnActivate = true,
  closeOnEscape = true,
} = defineProps<{
  /**
   * Whether the feed is loading or inserting articles (`aria-busy`).
   *
   * @default false
   */
  readonly busy?: boolean;

  /**
   * Default for rendered items: mark a notification read when its article is activated.
   *
   * @default true
   */
  readonly markReadOnActivate?: boolean;

  /**
   * Let Escape inside a disclosure-style feed close it and refocus the trigger.
   *
   * @default true
   */
  readonly closeOnEscape?: boolean;
}>();

defineSlots<{
  /** Render one notification, normally with NotificationCenterItem. */
  item?(props: NotificationCenterListItemSlotProps): unknown;

  /** Content after the articles, for example an empty state or a load-more button. */
  default?(): unknown;
}>();

const context = notificationCenterContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const notifications = computed<readonly NotificationRecord<unknown>[]>(
  () => context.store.notifications.value,
);
const count = computed<number>(() => notifications.value.length);
const feedHandlers = { onKeydown } as const;
const hidden = computed(() => !context.inline.value && !context.open.value);

function articles(): HTMLElement[] {
  return Array.from(element.value?.children ?? []).filter(
    (child): child is HTMLElement =>
      child instanceof HTMLElement &&
      child.getAttribute("data-vize-ui") === "notification-center-item",
  );
}

function focusItem(index: number): boolean {
  const target = articles()[index];
  if (!target) return false;
  target.focus();
  return true;
}

function onKeydown(event: KeyboardEvent): void {
  if (event.defaultPrevented || event.isComposing) return;
  if (event.key === "Escape" && closeOnEscape && !context.inline.value) {
    if (context.setOpen(false, event)) {
      event.preventDefault();
      context.triggerElement.value?.focus();
    }
    return;
  }
  if (event.key !== "PageDown" && event.key !== "PageUp") return;
  const list = articles();
  const current = list.findIndex(
    (article) => event.target instanceof Node && article.contains(event.target),
  );
  if (current === -1) return;
  const next = event.key === "PageDown" ? current + 1 : current - 1;
  if (focusItem(next)) event.preventDefault();
}

type NotificationCenterListSetupExpose = Omit<NotificationCenterListExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element, focusItem } satisfies NotificationCenterListSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="context.listId.value"
    ref="element"
    role="feed"
    :aria-label="context.label.value"
    :aria-busy="busy ? 'true' : 'false'"
    :hidden="hidden ? true : undefined"
    data-vize-ui="notification-center-list"
    part="list"
    :data-state="context.state.value"
    :data-count="count"
    v-bind="feedHandlers"
  >
    <template
      v-for="(notification, index) in notifications as readonly NotificationRecord<unknown>[]"
      :key="notification.id"
    >
      <slot name="item" :notification :position="index + 1" :set-size="count">
        <NotificationCenterItem
          :notification
          :position="index + 1"
          :set-size="count"
          :mark-read-on-activate
        />
      </slot>
    </template>
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
