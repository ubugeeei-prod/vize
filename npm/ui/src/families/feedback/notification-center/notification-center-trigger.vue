<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { notificationCenterContext } from "./notification-center-context.ts";
import type {
  NotificationCenterSlotState,
  NotificationCenterTriggerExpose,
} from "./notification-center-types.ts";

const { disabled = false, formatLabel = undefined } = defineProps<{
  /**
   * Remove the trigger from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Build the accessible name from the root label and unread count.
   *
   * @default `${label}, ${count} unread` when unread notifications exist, else `label`
   */
  readonly formatLabel?: (label: string, unreadCount: number) => string;
}>();

const emit = defineEmits<{
  /** Fired before the trigger toggles the center. Call `preventDefault()` to keep state unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Trigger contents, typically an icon and an unread badge. */
  default?(props: NotificationCenterSlotState): unknown;
}>();

const context = notificationCenterContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const unreadCount = computed(() => context.store.unreadCount.value);
const slotState = computed<NotificationCenterSlotState>(() => ({
  count: context.store.notifications.value.length,
  open: context.open.value,
  state: context.state.value,
  unreadCount: unreadCount.value,
}));
const accessibleName = computed(() => {
  if (formatLabel) return formatLabel(context.label.value, unreadCount.value);
  return unreadCount.value > 0
    ? `${context.label.value}, ${unreadCount.value} unread`
    : context.label.value;
});

onMounted(() => {
  context.triggerElement.value = element.value;
});

onUnmounted(() => {
  if (context.triggerElement.value === element.value) context.triggerElement.value = null;
});

function onClick(event: MouseEvent): void {
  if (disabled) return;
  emit("click", event);
  if (!event.defaultPrevented) context.setOpen(!context.open.value, event);
}

type NotificationCenterTriggerSetupExpose = Omit<NotificationCenterTriggerExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  focus: (options?: FocusOptions) => element.value?.focus(options),
} satisfies NotificationCenterTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="context.triggerId.value"
    ref="element"
    type="button"
    :disabled
    :aria-label="accessibleName"
    :aria-expanded="context.inline.value ? undefined : context.open.value ? 'true' : 'false'"
    :aria-controls="context.listId.value"
    data-vize-ui="notification-center-trigger"
    part="trigger"
    :data-state="context.state.value"
    :data-unread-count="unreadCount"
    :data-has-unread="unreadCount > 0 ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
