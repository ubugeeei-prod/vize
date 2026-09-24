/** Compile-only assertions for the public NotificationCenter contract. */

import type {
  NotificationCenterItemSlotState,
  NotificationCenterRootExpose,
  NotificationCenterSlotState,
  NotificationGroup,
  NotificationInput,
  NotificationReadState,
  NotificationRecord,
  NotificationStore,
  NotificationType,
} from "./notification-center.ts";
import {
  NotificationCenter,
  NotificationCenterEmpty,
  NotificationCenterItem,
  NotificationCenterList,
  NotificationCenterRoot,
  NotificationCenterTrigger,
  connectNotificationSource,
  createNotificationStore,
  useNotificationCenter,
} from "./notification-center.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Payload {
  readonly href: string;
}

const store = createNotificationStore<Payload>({ maxLength: 50, now: () => 0 });
declare const record: NotificationRecord<Payload>;
declare const itemSlot: NotificationCenterItemSlotState<Payload>;
declare const rootExpose: NotificationCenterRootExpose<Payload>;
declare const rootSlot: NotificationCenterSlotState;

type _Type = Expect<Equal<NotificationType, "error" | "info" | "neutral" | "success" | "warning">>;
type _Read = Expect<Equal<NotificationReadState, "read" | "unread">>;
type _Store = Expect<Equal<typeof store, NotificationStore<Payload>>>;
type _Data = Expect<Equal<typeof record.data, Payload | undefined>>;
type _List = Expect<
  Equal<typeof store.notifications.value, readonly NotificationRecord<Payload>[]>
>;
type _Groups = Expect<Equal<typeof store.groups.value, readonly NotificationGroup<Payload>[]>>;
type _Unread = Expect<Equal<typeof store.unreadCount.value, number>>;
type _SlotNotification = Expect<Equal<typeof itemSlot.notification, NotificationRecord<Payload>>>;
type _ExposeStore = Expect<Equal<typeof rootExpose.store, NotificationStore<Payload>>>;
type _Injected = Expect<
  Equal<ReturnType<typeof useNotificationCenter<Payload>>, NotificationStore<Payload>>
>;
type _RootSlot = Expect<Equal<typeof rootSlot.unreadCount, number>>;

const id: string = store.add({ title: "Deployed", data: { href: "/deploys/1" }, type: "success" });
store.update(id, { read: true, data: { href: "/deploys/2" } });
const disconnect: () => void = connectNotificationSource(store, (emit) => {
  emit({ title: "From source" });
  return () => undefined;
});

// @ts-expect-error payloads follow the store Data type.
store.add({ title: "Bad", data: { href: 1 } });

// @ts-expect-error title is required.
const missingTitle: NotificationInput = { description: "no title" };

// @ts-expect-error notification types are closed.
store.add({ title: "Bad", type: "fatal" });

const itemProps: InstanceType<typeof NotificationCenterList>["$props"] = {
  busy: true,
  closeOnEscape: false,
  markReadOnActivate: false,
};
const triggerProps: InstanceType<typeof NotificationCenterTrigger>["$props"] = {
  formatLabel: (label: string, count: number) => `${label} (${count})`,
};

void NotificationCenter;
void NotificationCenterEmpty;
void NotificationCenterItem;
void NotificationCenterRoot;
void disconnect;
void itemProps;
void missingTitle;
void triggerProps;
