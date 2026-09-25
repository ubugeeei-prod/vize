import { computed, shallowRef } from "vue";

import type {
  NotificationGroup,
  NotificationInput,
  NotificationPatch,
  NotificationRecord,
  NotificationSource,
  NotificationStore,
  NotificationStoreOptions,
  NotificationType,
} from "./notification-center-types.ts";

const optionDiagnostic = "VIZE_UI_NOTIFICATION_OPTION";
const notificationTypes = new Set<NotificationType>([
  "error",
  "info",
  "neutral",
  "success",
  "warning",
]);

const patchedFields = ["data", "description", "group", "read", "title", "type"] as const;

function readMaxLength(value: number | undefined): number {
  if (value === undefined) return 100;
  if (!Number.isInteger(value) || value < 1) {
    throw new TypeError(`${optionDiagnostic}: maxLength must be a positive integer`);
  }
  return value;
}

function readType(value: NotificationType | undefined): NotificationType {
  if (value === undefined) return "neutral";
  if (!notificationTypes.has(value)) {
    throw new TypeError(`${optionDiagnostic}: type must be a NotificationType`);
  }
  return value;
}

function newestFirst<Data>(left: NotificationRecord<Data>, right: NotificationRecord<Data>) {
  return right.createdAt - left.createdAt;
}

/**
 * Create an SSR-safe notification history store.
 *
 * The store owns no timers and touches no DOM, so it can be created per
 * request on the server. Every mutation replaces immutable snapshots.
 */
export function createNotificationStore<Data = unknown>(
  options: NotificationStoreOptions<Data> = {},
): NotificationStore<Data> {
  const maxLength = readMaxLength(options.maxLength);
  const now = options.now ?? Date.now;
  const idPrefix = options.idPrefix ?? "notification";
  const records = shallowRef<readonly NotificationRecord<Data>[]>([]);
  let sequence = 0;

  const all = computed(() => [...records.value].sort(newestFirst));
  const notifications = computed(() => all.value.filter((record) => !record.archived));
  const archived = computed(() => all.value.filter((record) => record.archived));
  const unreadCount = computed(() => notifications.value.filter((record) => !record.read).length);
  const groups = computed<readonly NotificationGroup<Data>[]>(() => {
    const map = new Map<string | null, NotificationRecord<Data>[]>();
    for (const record of notifications.value) {
      const bucket = map.get(record.group);
      if (bucket) bucket.push(record);
      else map.set(record.group, [record]);
    }
    return [...map].map(([key, members]) => ({
      key,
      notifications: members,
      unreadCount: members.filter((record) => !record.read).length,
    }));
  });

  function nextId(): string {
    sequence += 1;
    return `${idPrefix}-${sequence}`;
  }

  function evict(list: readonly NotificationRecord<Data>[]): readonly NotificationRecord<Data>[] {
    if (list.length <= maxLength) return list;
    const keep = new Set([...list].sort(newestFirst).slice(0, maxLength));
    return list.filter((record) => keep.has(record));
  }

  function replace(
    id: string,
    change: (record: NotificationRecord<Data>) => NotificationRecord<Data>,
  ): boolean {
    let changed = false;
    records.value = records.value.map((record) => {
      if (record.id !== id) return record;
      const next = change(record);
      changed = next !== record;
      return next;
    });
    return changed;
  }

  function add(input: NotificationInput<Data>): string {
    if (typeof input.title !== "string") {
      throw new TypeError(`${optionDiagnostic}: title must be a string`);
    }
    const id = input.id ?? nextId();
    const timestamp = now();
    const record: NotificationRecord<Data> = Object.freeze({
      archived: false,
      createdAt: input.createdAt ?? timestamp,
      data: input.data,
      description: input.description ?? null,
      group: input.group ?? null,
      id,
      read: input.read ?? false,
      title: input.title,
      type: readType(input.type),
      updatedAt: timestamp,
    });
    const rest = records.value.filter((candidate) => candidate.id !== id);
    records.value = evict([...rest, record]);
    return id;
  }

  function update(id: string, patch: NotificationPatch<Data>): boolean {
    return replace(id, (record) => {
      const next: NotificationRecord<Data> = {
        ...record,
        data: "data" in patch ? patch.data : record.data,
        description: patch.description === undefined ? record.description : patch.description,
        group: patch.group === undefined ? record.group : patch.group,
        read: patch.read ?? record.read,
        title: patch.title ?? record.title,
        type: patch.type === undefined ? record.type : readType(patch.type),
      };
      const same = patchedFields.every((field) => Object.is(next[field], record[field]));
      return same ? record : Object.freeze({ ...next, updatedAt: now() });
    });
  }

  function setFlag(id: string, key: "archived" | "read", value: boolean): boolean {
    return replace(id, (record) =>
      record[key] === value ? record : Object.freeze({ ...record, [key]: value, updatedAt: now() }),
    );
  }

  const store: NotificationStore<Data> = {
    add,
    archive: (id) => setFlag(id, "archived", true),
    archived,
    clear: () => {
      records.value = [];
    },
    get: (id) => records.value.find((record) => record.id === id),
    groups,
    markAllRead: () => {
      let count = 0;
      const timestamp = now();
      records.value = records.value.map((record) => {
        if (record.read || record.archived) return record;
        count += 1;
        return Object.freeze({ ...record, read: true, updatedAt: timestamp });
      });
      return count;
    },
    markRead: (id) => setFlag(id, "read", true),
    markUnread: (id) => setFlag(id, "read", false),
    notifications,
    remove: (id) => {
      const before = records.value.length;
      records.value = records.value.filter((record) => record.id !== id);
      return records.value.length !== before;
    },
    unarchive: (id) => setFlag(id, "archived", false),
    unreadCount,
    update,
  };

  for (const input of options.initial ?? []) add(input);
  return Object.freeze(store);
}

/**
 * Feed a store from any push-based source (a toast store's dismissal hook,
 * a server event stream, a BroadcastChannel). Returns the source's unsubscribe.
 */
export function connectNotificationSource<Data>(
  store: NotificationStore<Data>,
  source: NotificationSource<Data>,
): () => void {
  return source((input) => {
    store.add(input);
  });
}
