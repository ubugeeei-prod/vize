import { shallowReadonly, shallowRef } from "vue";

import type {
  ToastActionOptions,
  ToastDismissReason,
  ToastId,
  ToastOptions,
  ToastPauseReason,
  ToastPriority,
  ToastPromiseMessage,
  ToastPromiseOptions,
  ToastRecord,
  ToastStore,
  ToastStoreOptions,
  ToastType,
  ToastVariantOptions,
} from "./toast-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_TOAST_OPTION";
const toastTypes = new Set<ToastType>([
  "default",
  "error",
  "info",
  "loading",
  "success",
  "warning",
]);
const priorityRank: Readonly<Record<ToastPriority, number>> = { high: 2, normal: 1, low: 0 };

interface ToastEntry<Data> {
  record: ToastRecord<Data>;
  explicitDuration: number | undefined;
  shown: boolean;
  shownOrder: number;
  remaining: number;
  timer: ReturnType<typeof setTimeout> | null;
  timerStartedAt: number;
  onDismiss: ToastOptions<Data>["onDismiss"];
  onAutoClose: ToastOptions<Data>["onAutoClose"];
}

function readDuration(value: number | undefined, name: string): number | undefined {
  if (value === undefined) return undefined;
  if (typeof value !== "number" || Number.isNaN(value) || value < 0) {
    throw new TypeError(`${invalidOptionDiagnostic}: ${name} must be a non-negative number`);
  }
  return value;
}

function readLimit(value: number | undefined): number | undefined {
  if (value === undefined) return undefined;
  if (!Number.isInteger(value) || value < 1) {
    throw new TypeError(`${invalidOptionDiagnostic}: limit must be a positive integer`);
  }
  return value;
}

function readType(value: ToastType | undefined): ToastType | undefined {
  if (value !== undefined && !toastTypes.has(value)) {
    throw new TypeError(`${invalidOptionDiagnostic}: unknown toast type`);
  }
  return value;
}

function readPriority(value: ToastPriority | undefined): ToastPriority | undefined {
  if (value !== undefined && !(value in priorityRank)) {
    throw new TypeError(`${invalidOptionDiagnostic}: unknown toast priority`);
  }
  return value;
}

function readAction(value: ToastActionOptions | undefined): ToastActionOptions | undefined {
  if (value === undefined) return undefined;
  if (value.altText.trim().length === 0) {
    throw new TypeError(`${invalidOptionDiagnostic}: action altText must not be empty`);
  }
  return value;
}

function isQueuedBefore<Data>(left: ToastEntry<Data>, right: ToastEntry<Data>): number {
  const rank = priorityRank[right.record.priority] - priorityRank[left.record.priority];
  return rank === 0 ? left.record.sequence - right.record.sequence : rank;
}

function resolvePhase<Input, Data>(
  message: ToastPromiseMessage<Input, Data>,
  input: Input,
): ToastVariantOptions<Data> {
  const resolved = typeof message === "function" ? message(input) : message;
  return typeof resolved === "string" ? { title: resolved } : resolved;
}

function toOptions<Data>(input: string | ToastVariantOptions<Data>): ToastVariantOptions<Data> {
  return typeof input === "string" ? { title: input } : input;
}

/**
 * Create a typed, request-local toast queue.
 *
 * The store is pure data until {@link ToastStore.start} runs, so creating
 * toasts during server rendering is deterministic and never schedules timers.
 */
export function createToastStore<Data = unknown>(
  options: ToastStoreOptions = {},
): ToastStore<Data> {
  const duration = shallowRef(readDuration(options.duration, "duration") ?? 5000);
  const limit = shallowRef(readLimit(options.limit) ?? 3);
  const idPrefix = options.idPrefix ?? "toast";
  const entries = new Map<ToastId, ToastEntry<Data>>();
  const toasts = shallowRef<readonly ToastRecord<Data>[]>([]);
  const visibleToasts = shallowRef<readonly ToastRecord<Data>[]>([]);
  const queuedToasts = shallowRef<readonly ToastRecord<Data>[]>([]);
  const paused = shallowRef(false);
  const pauseReasons = new Set<ToastPauseReason>();
  let started = false;
  let sequence = 0;
  let generatedId = 0;
  let shownOrder = 0;

  const resolveDuration = (type: ToastType, explicit: number | undefined): number =>
    explicit ?? (type === "loading" ? Number.POSITIVE_INFINITY : duration.value);

  const shouldRun = (entry: ToastEntry<Data>): boolean =>
    started &&
    !paused.value &&
    entry.shown &&
    entry.record.open &&
    Number.isFinite(entry.record.duration);

  const suspend = (entry: ToastEntry<Data>): void => {
    if (entry.timer === null) return;
    clearTimeout(entry.timer);
    entry.timer = null;
    entry.remaining = Math.max(0, entry.remaining - (Date.now() - entry.timerStartedAt));
  };

  const expire = (id: ToastId): void => {
    const entry = entries.get(id);
    if (!entry) return;
    entry.timer = null;
    entry.remaining = 0;
    entry.onAutoClose?.(entry.record);
    dismissOne(entry, "timeout");
    commit();
  };

  const refreshTimers = (): void => {
    for (const entry of entries.values()) {
      if (!shouldRun(entry)) {
        suspend(entry);
      } else if (entry.timer === null) {
        entry.timerStartedAt = Date.now();
        const id = entry.record.id;
        entry.timer = setTimeout(() => expire(id), entry.remaining);
      }
    }
  };

  const promote = (): void => {
    const all = [...entries.values()];
    let openShown = all.filter((entry) => entry.shown && entry.record.open).length;
    const queue = all.filter((entry) => !entry.shown && entry.record.open).sort(isQueuedBefore);
    for (const entry of queue) {
      if (openShown >= limit.value) break;
      entry.shown = true;
      entry.shownOrder = ++shownOrder;
      openShown += 1;
    }
  };

  const commit = (): void => {
    promote();
    const all = [...entries.values()];
    toasts.value = all
      .map((entry) => entry.record)
      .sort((left, right) => left.sequence - right.sequence);
    visibleToasts.value = all
      .filter((entry) => entry.shown)
      .sort((left, right) => left.shownOrder - right.shownOrder)
      .map((entry) => entry.record);
    queuedToasts.value = all
      .filter((entry) => !entry.shown && entry.record.open)
      .sort(isQueuedBefore)
      .map((entry) => entry.record);
    refreshTimers();
  };

  const removeEntry = (entry: ToastEntry<Data>): void => {
    suspend(entry);
    entries.delete(entry.record.id);
  };

  function dismissOne(entry: ToastEntry<Data>, reason: ToastDismissReason): boolean {
    if (!entry.record.open) return false;
    suspend(entry);
    entry.record = Object.freeze({
      ...entry.record,
      dismissReason: reason,
      open: false,
      state: "closed",
    });
    const record = entry.record;
    if (!entry.shown) removeEntry(entry);
    entry.onDismiss?.(record, reason);
    return true;
  }

  const upsert = (input: ToastOptions<Data>): ToastId => {
    const type = readType(input.type);
    const priority = readPriority(input.priority);
    const explicitDuration = readDuration(input.duration, "duration");
    const action = readAction(input.action);
    const id = input.id ?? `${idPrefix}-${++generatedId}`;
    const existing = entries.get(id);

    if (existing) {
      const nextType = type ?? existing.record.type;
      const nextExplicit =
        input.duration === undefined
          ? type === undefined
            ? existing.explicitDuration
            : undefined
          : explicitDuration;
      const nextDuration = resolveDuration(nextType, nextExplicit);
      suspend(existing);
      existing.explicitDuration = nextExplicit;
      existing.remaining = nextDuration;
      if (input.onDismiss !== undefined) existing.onDismiss = input.onDismiss;
      if (input.onAutoClose !== undefined) existing.onAutoClose = input.onAutoClose;
      existing.record = Object.freeze({
        ...existing.record,
        title: "title" in input ? input.title : existing.record.title,
        description: "description" in input ? input.description : existing.record.description,
        type: nextType,
        priority: priority ?? existing.record.priority,
        duration: nextDuration,
        dismissible: input.dismissible ?? existing.record.dismissible,
        action: "action" in input ? action : existing.record.action,
        data: "data" in input ? input.data : existing.record.data,
        open: true,
        state: "open",
        dismissReason: null,
        revision: existing.record.revision + 1,
      });
      commit();
      return id;
    }

    const resolvedType = type ?? "default";
    const resolvedDuration = resolveDuration(resolvedType, explicitDuration);
    entries.set(id, {
      record: Object.freeze({
        id,
        title: input.title,
        description: input.description,
        type: resolvedType,
        priority: priority ?? "normal",
        duration: resolvedDuration,
        dismissible: input.dismissible ?? true,
        action,
        data: input.data,
        open: true,
        state: "open",
        dismissReason: null,
        sequence: ++sequence,
        revision: 0,
      }),
      explicitDuration,
      shown: false,
      shownOrder: 0,
      remaining: resolvedDuration,
      timer: null,
      timerStartedAt: 0,
      onDismiss: input.onDismiss,
      onAutoClose: input.onAutoClose,
    });
    commit();
    return id;
  };

  const toast = (input: string | ToastOptions<Data>): ToastId =>
    upsert(typeof input === "string" ? { title: input } : input);

  const variant =
    (type: ToastType) =>
    (input: string | ToastVariantOptions<Data>): ToastId =>
      upsert({ ...toOptions(input), type });

  const promise = <Value>(
    source: Promise<Value>,
    phases: ToastPromiseOptions<Value, Data>,
  ): Promise<Value> => {
    const loading = toOptions(phases.loading);
    const id = upsert({
      ...loading,
      ...(phases.id === undefined ? {} : { id: phases.id }),
      type: "loading",
    });
    source.then(
      (value) => {
        upsert({ ...resolvePhase(phases.success, value), id, type: "success" });
      },
      (reason: unknown) => {
        upsert({ ...resolvePhase(phases.error, reason), id, type: "error" });
      },
    );
    return source;
  };

  const update = (id: ToastId, patch: Omit<ToastOptions<Data>, "id">): boolean => {
    if (!entries.has(id)) return false;
    upsert({ ...patch, id });
    return true;
  };

  const dismiss = (id?: ToastId, reason: ToastDismissReason = "api"): boolean => {
    let changed = false;
    if (id === undefined) {
      for (const entry of entries.values()) changed = dismissOne(entry, reason) || changed;
    } else {
      const entry = entries.get(id);
      changed = entry ? dismissOne(entry, reason) : false;
    }
    if (changed) commit();
    return changed;
  };

  const remove = (id: ToastId): boolean => {
    const entry = entries.get(id);
    if (!entry) return false;
    if (entry.record.open) dismissOne(entry, "api");
    removeEntry(entry);
    commit();
    return true;
  };

  const clear = (): void => {
    for (const entry of entries.values()) suspend(entry);
    entries.clear();
    commit();
  };

  const pause = (reason: ToastPauseReason = "manual"): void => {
    pauseReasons.add(reason);
    if (!paused.value) {
      paused.value = true;
      refreshTimers();
    }
  };

  const resume = (reason: ToastPauseReason = "manual"): void => {
    pauseReasons.delete(reason);
    if (paused.value && pauseReasons.size === 0) {
      paused.value = false;
      refreshTimers();
    }
  };

  const configure = (next: Omit<ToastStoreOptions, "idPrefix">): void => {
    const nextDuration = readDuration(next.duration, "duration");
    const nextLimit = readLimit(next.limit);
    if (nextDuration !== undefined) duration.value = nextDuration;
    if (nextLimit !== undefined) limit.value = nextLimit;
    commit();
  };

  return Object.freeze({
    toasts: shallowReadonly(toasts),
    visibleToasts: shallowReadonly(visibleToasts),
    queuedToasts: shallowReadonly(queuedToasts),
    paused: shallowReadonly(paused),
    duration: shallowReadonly(duration),
    limit: shallowReadonly(limit),
    toast,
    success: variant("success"),
    error: variant("error"),
    warning: variant("warning"),
    info: variant("info"),
    loading: variant("loading"),
    promise,
    update,
    dismiss,
    remove,
    clear,
    get: (id: ToastId) => entries.get(id)?.record,
    pause,
    resume,
    configure,
    start: () => {
      if (started) return;
      started = true;
      refreshTimers();
    },
    stop: () => {
      started = false;
      refreshTimers();
    },
  });
}
