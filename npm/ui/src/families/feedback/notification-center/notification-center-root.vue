<script setup lang="ts" generic="Data = unknown">
import { computed, provide, shallowRef } from "vue";
import type { ComputedRef, InjectionKey } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { notificationCenterContext, notificationStoreKey } from "./notification-center-context.ts";
import { createNotificationStore } from "./notification-center-store.ts";
import type {
  NotificationCenterRootExpose,
  NotificationCenterSlotState,
  NotificationCenterState,
  NotificationInput,
  NotificationStore,
  NotificationStoreOptions,
} from "./notification-center-types.ts";

const {
  store = undefined,
  initial = undefined,
  maxLength = 100,
  now = undefined,
  id = undefined,
  label = "Notifications",
  inline = false,
  open = undefined,
  defaultOpen = false,
} = defineProps<{
  /**
   * External store, for example one shared with a router guard or fed by
   * `connectNotificationSource`. `undefined` creates a store per root.
   *
   * @default undefined
   */
  readonly store?: NotificationStore<Data>;

  /**
   * Notifications seeded into the root-owned store.
   *
   * @default undefined
   */
  readonly initial?: readonly NotificationInput<Data>[];

  /**
   * History limit of the root-owned store.
   *
   * @default 100
   */
  readonly maxLength?: number;

  /**
   * Clock of the root-owned store; inject one for deterministic SSR.
   *
   * @default Date.now
   */
  readonly now?: () => number;

  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Accessible name of the feed and base label of the trigger.
   *
   * @default "Notifications"
   */
  readonly label?: string;

  /**
   * Render the feed permanently (inbox page) instead of as a disclosure.
   *
   * @default false
   */
  readonly inline?: boolean;

  /**
   * Controlled disclosure state. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly open?: boolean;

  /**
   * Initial disclosure state for uncontrolled use.
   *
   * @default false
   */
  readonly defaultOpen?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the disclosure requests a controlled open value. */
  "update:open": [value: boolean];

  /** Fired after any distinct open-state request. */
  "open-change": [value: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Trigger, list, and empty-state children. */
  default?(props: NotificationCenterSlotState): unknown;
}>();

function createOwnedStore(): NotificationStore<Data> {
  const options: NotificationStoreOptions<Data> = { maxLength };
  return createNotificationStore<Data>({
    ...options,
    ...(initial === undefined ? {} : { initial }),
    ...(now === undefined ? {} : { now }),
  });
}

// The store identity is fixed for the lifetime of the root.
const ownedStore = store ?? createOwnedStore();
const baseId = useDeterministicId({ id: () => id, hint: "notification-center" });
const listId = computed(() => deriveDeterministicId(baseId.value, "list"));
const triggerId = computed(() => deriveDeterministicId(baseId.value, "trigger"));
const labelState = computed(() => label);
const inlineState = computed(() => inline);
const openState = useControllableState({ value: () => open, defaultValue: () => defaultOpen });
const isOpen = computed(() => inline || openState.value.value);
const state = computed<NotificationCenterState>(() => (isOpen.value ? "open" : "closed"));
const triggerElement = shallowRef<HTMLButtonElement | null>(null);
const slotState = computed<NotificationCenterSlotState>(() => ({
  count: ownedStore.notifications.value.length,
  open: isOpen.value,
  state: state.value,
  unreadCount: ownedStore.unreadCount.value,
}));

const unreadCount = computed<number>(() => ownedStore.unreadCount.value);

function readOpen(): boolean {
  return isOpen.value;
}

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  if (inline) return false;
  const previous = readOpen();
  if (!openState.set(value)) return false;
  emit("update:open", value);
  emit("open-change", value, previous, nativeEvent);
  return true;
}

const storeKey: InjectionKey<NotificationStore<Data>> = notificationStoreKey;
provide(storeKey, ownedStore);
notificationCenterContext.provide({
  inline: inlineState,
  itemId: (value) => deriveDeterministicId(baseId.value, `item-${itemSegment(value)}`),
  label: labelState,
  listId,
  open: isOpen,
  setOpen,
  state,
  store: ownedStore,
  triggerElement,
  triggerId,
});

function itemSegment(value: string): string {
  return /^[A-Za-z0-9][A-Za-z0-9_-]*$/.test(value)
    ? value
    : `x${[...value].map((char) => char.charCodeAt(0).toString(36)).join("")}`;
}

type NotificationCenterRootSetupExpose = Omit<
  NotificationCenterRootExpose<Data>,
  "count" | "listId" | "open" | "state" | "unreadCount"
> & {
  readonly count: ComputedRef<number>;
  readonly listId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<NotificationCenterState>;
  readonly unreadCount: ComputedRef<number>;
};

const exposed = {
  count: computed(() => ownedStore.notifications.value.length),
  listId,
  open: isOpen,
  setOpen,
  state,
  store: ownedStore,
  unreadCount: ownedStore.unreadCount,
} satisfies NotificationCenterRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="notification-center-root"
    part="root"
    :data-state="state"
    :data-inline="inline ? 'true' : undefined"
    :data-unread-count="unreadCount"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
