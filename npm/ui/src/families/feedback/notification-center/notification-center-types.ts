import type { ComputedRef } from "vue";

/** Semantic tone of one notification, mirrored to `data-type`. */
export type NotificationType = "error" | "info" | "neutral" | "success" | "warning";

/** Read state mirrored to the NotificationCenterItem data contract. */
export type NotificationReadState = "read" | "unread";

/** Open state of a disclosure-style notification center. */
export type NotificationCenterState = "closed" | "open";

/** Input accepted by {@link NotificationStore.add}. */
export interface NotificationInput<Data = unknown> {
  /**
   * Stable id. Adding an existing id replaces that notification in place.
   *
   * @default a store-generated `"{idPrefix}-{n}"` id
   */
  readonly id?: string;

  /** Short headline used as the item accessible name. */
  readonly title: string;

  /**
   * Supporting text used as the item description.
   *
   * @default null
   */
  readonly description?: string | null;

  /**
   * Semantic tone.
   *
   * @default "neutral"
   */
  readonly type?: NotificationType;

  /**
   * Grouping key, for example a source, thread, or day bucket.
   *
   * @default null
   */
  readonly group?: string | null;

  /**
   * Initial read state.
   *
   * @default false
   */
  readonly read?: boolean;

  /**
   * Creation timestamp in milliseconds.
   *
   * @default the store `now()` at insertion
   */
  readonly createdAt?: number;

  /**
   * Consumer payload rendered by custom item slots.
   *
   * @default undefined
   */
  readonly data?: Data;
}

/** Mutable fields accepted by {@link NotificationStore.update}. */
export type NotificationPatch<Data = unknown> = Partial<
  Pick<NotificationRecord<Data>, "data" | "description" | "group" | "read" | "title" | "type">
>;

/** Immutable snapshot of one stored notification. */
export interface NotificationRecord<Data = unknown> {
  /** Stable notification id. */
  readonly id: string;
  /** Headline. */
  readonly title: string;
  /** Supporting text, or `null`. */
  readonly description: string | null;
  /** Semantic tone. */
  readonly type: NotificationType;
  /** Grouping key, or `null`. */
  readonly group: string | null;
  /** Whether the user has read the notification. */
  readonly read: boolean;
  /** Whether the notification left the active feed for the archive. */
  readonly archived: boolean;
  /** Creation timestamp in milliseconds. */
  readonly createdAt: number;
  /** Timestamp of the latest change in milliseconds. */
  readonly updatedAt: number;
  /** Consumer payload. */
  readonly data: Data | undefined;
}

/** One group of active notifications, newest first. */
export interface NotificationGroup<Data = unknown> {
  /** Grouping key, or `null` for ungrouped notifications. */
  readonly key: string | null;
  /** Active notifications in this group, newest first. */
  readonly notifications: readonly NotificationRecord<Data>[];
  /** Unread notifications in this group. */
  readonly unreadCount: number;
}

/** Options accepted by {@link createNotificationStore}. */
export interface NotificationStoreOptions<Data = unknown> {
  /**
   * Maximum number of retained notifications (active and archived). The
   * oldest entries are evicted first.
   *
   * @default 100
   */
  readonly maxLength?: number;

  /**
   * Clock used for timestamps. Inject a fixed clock for deterministic SSR and tests.
   *
   * @default Date.now
   */
  readonly now?: () => number;

  /**
   * Prefix for generated ids.
   *
   * @default "notification"
   */
  readonly idPrefix?: string;

  /**
   * Notifications present when the store is created, newest first or in any
   * order; the store sorts by `createdAt`.
   *
   * @default []
   */
  readonly initial?: readonly NotificationInput<Data>[];
}

/**
 * SSR-safe notification history. Member functions use method syntax so a
 * store of a concrete `Data` can be provided through a shared context.
 */
export interface NotificationStore<Data = unknown> {
  /** Active (not archived) notifications, newest first. */
  readonly notifications: ComputedRef<readonly NotificationRecord<Data>[]>;
  /** Archived notifications, newest first. */
  readonly archived: ComputedRef<readonly NotificationRecord<Data>[]>;
  /** Unread active notifications. */
  readonly unreadCount: ComputedRef<number>;
  /** Active notifications grouped by `group`, in first-appearance order. */
  readonly groups: ComputedRef<readonly NotificationGroup<Data>[]>;
  /** Add or replace a notification and return its id. */
  add(input: NotificationInput<Data>): string;
  /** Patch a notification. Returns whether it exists and changed. */
  update(id: string, patch: NotificationPatch<Data>): boolean;
  /** Read one notification snapshot. */
  get(id: string): NotificationRecord<Data> | undefined;
  /** Mark one notification read. Returns whether it changed. */
  markRead(id: string): boolean;
  /** Mark one notification unread. Returns whether it changed. */
  markUnread(id: string): boolean;
  /** Mark every active notification read and return how many changed. */
  markAllRead(): number;
  /** Move a notification to the archive. Returns whether it changed. */
  archive(id: string): boolean;
  /** Restore an archived notification. Returns whether it changed. */
  unarchive(id: string): boolean;
  /** Delete a notification. Returns whether it existed. */
  remove(id: string): boolean;
  /** Delete every notification. */
  clear(): void;
}

/**
 * Push-based source of notifications, for example a toast store's dismissal
 * hook or a server event stream. It receives an `emit` callback and returns
 * an unsubscribe function.
 */
export type NotificationSource<Data = unknown> = (
  emit: (input: NotificationInput<Data>) => void,
) => () => void;

/** State exposed to NotificationCenterRoot slots. */
export interface NotificationCenterSlotState {
  /** Whether the list is shown (always `true` for inline centers). */
  readonly open: boolean;
  /** Stable state token. */
  readonly state: NotificationCenterState;
  /** Unread active notifications. */
  readonly unreadCount: number;
  /** Active notifications. */
  readonly count: number;
}

/** State exposed to NotificationCenterItem slots. */
export interface NotificationCenterItemSlotState<Data = unknown> {
  /** The rendered notification. */
  readonly notification: NotificationRecord<Data>;
  /** One-based position in the feed. */
  readonly position: number;
  /** Feed size. */
  readonly setSize: number;
  /** Id to put on the visible headline. */
  readonly titleId: string;
  /** Id to put on the visible description. */
  readonly descriptionId: string;
  /** Mark this notification read. */
  readonly markRead: () => boolean;
  /** Mark this notification unread. */
  readonly markUnread: () => boolean;
  /** Archive this notification. */
  readonly archive: () => boolean;
  /** Delete this notification. */
  readonly remove: () => boolean;
}

/** Public instance exposed by NotificationCenterRoot. */
export interface NotificationCenterRootExpose<Data = unknown> extends NotificationCenterSlotState {
  /** The store driving this center. */
  readonly store: NotificationStore<Data>;
  /** Id of the feed element. */
  readonly listId: string;
  /** Request an open value. Returns whether it changed. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;
}

/** Public instance exposed by NotificationCenterTrigger. */
export interface NotificationCenterTriggerExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;
  /** Move focus to the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by NotificationCenterList. */
export interface NotificationCenterListExpose {
  /** Rendered feed element. */
  readonly element: HTMLElement | null;
  /** Focus the article at a zero-based index. Returns whether it moved focus. */
  readonly focusItem: (index: number) => boolean;
}

/** Public instance exposed by NotificationCenterItem. */
export interface NotificationCenterItemExpose {
  /** Rendered article. */
  readonly element: HTMLElement | null;
}
