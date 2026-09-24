import { computed, nextTick, shallowRef, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { useDragAndDrop } from "../../interaction/drag-and-drop/drag-and-drop.ts";
import { resolveLocale, useLocaleValue } from "../../i18n/locale/locale-runtime.ts";
import { createCalendarFormatters, normalizeWeekday } from "../calendar/calendar-locale.ts";
import { useToday } from "../calendar/calendar-today.ts";
import type { DateTimeNow } from "../calendar/calendar-today.ts";
import {
  addDays,
  daysBetween,
  endOfWeek,
  formatIsoDate,
  isSameDay,
  normalizePlainDate,
  startOfWeek,
  toUtcDate,
} from "../calendar/plain-date.ts";
import type { PlainDate, Weekday } from "../calendar/plain-date.ts";
import { addMinutes, combineDateTime, minutesBetween } from "../datetime-field/plain-date-time.ts";
import type { PlainDateTime } from "../datetime-field/plain-date-time.ts";
import { schedulerContext } from "./scheduler-context.ts";
import type {
  SchedulerContextValue,
  SchedulerDragData,
  SchedulerDragKind,
} from "./scheduler-context.ts";
import {
  layoutRow,
  layoutTimeGrid,
  minuteAtPoint,
  monthWeeks,
  shiftAnchor,
  visibleDays,
} from "./scheduler-layout.ts";
import type { SchedulerEvent, SchedulerView } from "./scheduler-layout.ts";
import type {
  SchedulerDay,
  SchedulerDayColumnState,
  SchedulerEventChange,
  SchedulerRootExpose,
  SchedulerSlotRange,
  SchedulerSlotRow,
  SchedulerSlotState,
  SchedulerState,
  SchedulerWeekState,
} from "./scheduler-types.ts";

/** Props read by {@link useScheduler}; every key is present and may be undefined. */
export interface SchedulerRuntimeProps<Data> {
  readonly id: string | null | undefined;
  readonly events: readonly SchedulerEvent<Data>[] | undefined;
  readonly view: SchedulerView | undefined;
  readonly defaultView: SchedulerView | undefined;
  readonly date: PlainDate | null | undefined;
  readonly defaultDate: PlainDate | null | undefined;
  readonly today: PlainDate | null | undefined;
  readonly now: DateTimeNow | undefined;
  readonly timeZone: string | undefined;
  readonly locale: string | undefined;
  readonly dir: "ltr" | "rtl" | undefined;
  readonly weekStartsOn: Weekday | undefined;
  readonly dayStartHour: number | undefined;
  readonly dayEndHour: number | undefined;
  readonly slotMinutes: number | undefined;
  readonly snapMinutes: number | undefined;
  readonly minimumEventMinutes: number | undefined;
  readonly maxLanes: number | undefined;
  readonly disabled: boolean | undefined;
  readonly readOnly: boolean | undefined;
}

/** Emit callbacks used by {@link useScheduler}. */
export interface SchedulerEmitters<Data> {
  readonly view: (view: SchedulerView) => void;
  readonly date: (date: PlainDate) => void;
  readonly eventActivate: (event: SchedulerEvent<Data>, nativeEvent: Event) => void;
  readonly slotActivate: (range: SchedulerSlotRange, nativeEvent: Event) => void;
  readonly dayActivate: (date: PlainDate, nativeEvent: Event) => void;
  readonly eventMove: (change: SchedulerEventChange<Data>, nativeEvent: Event | null) => void;
  readonly eventResize: (change: SchedulerEventChange<Data>, nativeEvent: Event | null) => void;
}

type SchedulerSetupExpose<Data> = {
  readonly [Key in keyof SchedulerRootExpose<Data>]: SchedulerRootExpose<Data>[Key] extends (
    ...args: never[]
  ) => unknown
    ? SchedulerRootExpose<Data>[Key]
    : Key extends "root"
      ? Readonly<ShallowRef<HTMLDivElement | null>>
      : ComputedRef<SchedulerRootExpose<Data>[Key]>;
};

const dragKinds: Readonly<Record<SchedulerDragKind, string>> = {
  move: "vize-scheduler-move",
  resize: "vize-scheduler-resize",
};

function clampInteger(
  value: number | undefined,
  fallback: number,
  min: number,
  max: number,
): number {
  if (value === undefined || !Number.isFinite(value)) return fallback;
  return Math.min(max, Math.max(min, Math.trunc(value)));
}

function dayDateTime(date: PlainDate, minute: number): PlainDateTime {
  return combineDateTime(date, { hour: Math.floor(minute / 60), minute: minute % 60, second: 0 });
}

/**
 * Scheduler state machine: views, SSR-safe anchoring, pure layouts, roving
 * keyboard focus, and drag/keyboard move and resize requests. Must run in setup.
 */
export function useScheduler<Data>(
  props: SchedulerRuntimeProps<Data>,
  emit: SchedulerEmitters<Data>,
  root: Readonly<ShallowRef<HTMLDivElement | null>>,
) {
  const localeValue = useLocaleValue();
  const id = useDeterministicId({ id: () => props.id, hint: "scheduler" });
  const headingId = computed(() => deriveDeterministicId(id.value, "heading"));
  const locale = computed(() => resolveLocale(props.locale ?? localeValue.value.locale));
  const direction = computed<"ltr" | "rtl">(() =>
    props.dir === "rtl" || props.dir === "ltr" ? props.dir : localeValue.value.direction,
  );
  const formatters = computed(() =>
    createCalendarFormatters({ locale: locale.value, calendar: localeValue.value.calendar }),
  );
  const weekStartsOn = computed(() => normalizeWeekday(props.weekStartsOn, locale.value));
  const disabled = computed(() => props.disabled === true);
  const readOnly = computed(() => props.readOnly === true);
  const windowStart = computed(() => clampInteger(props.dayStartHour, 0, 0, 23) * 60);
  const windowEnd = computed(() =>
    Math.max(windowStart.value + 60, clampInteger(props.dayEndHour, 24, 1, 24) * 60),
  );
  const slotMinutes = computed(() => clampInteger(props.slotMinutes, 30, 5, 240));
  const snapMinutes = computed(() => clampInteger(props.snapMinutes, slotMinutes.value, 1, 240));
  const maxLanes = computed(() => clampInteger(props.maxLanes, 3, 1, 50));
  const today = useToday({
    today: () => props.today,
    now: () => props.now,
    timeZone: () => props.timeZone,
  });
  const viewState = useControllableState<SchedulerView>({
    value: () => props.view,
    defaultValue: () => props.defaultView ?? "week",
    onChange: (value) => emit.view(value),
  });
  const internalDate = shallowRef<PlainDate | null>(normalizePlainDate(props.defaultDate));
  const anchor = computed<PlainDate | null>(
    () => normalizePlainDate(props.date) ?? internalDate.value ?? today.value,
  );
  const view = computed(() => viewState.value.value);
  const events = computed<readonly SchedulerEvent<Data>[]>(() => props.events ?? []);
  const focusedDate = shallowRef<PlainDate | null>(null);
  const focusedMinuteState = shallowRef<number | null>(null);

  const timeFormatter = computed(() => {
    try {
      return new Intl.DateTimeFormat(locale.value, { timeStyle: "short", timeZone: "UTC" });
    } catch {
      return new Intl.DateTimeFormat("en-US", { timeStyle: "short", timeZone: "UTC" });
    }
  });
  const headerFormatter = computed(() => {
    try {
      return new Intl.DateTimeFormat(locale.value, {
        weekday: "short",
        day: "numeric",
        timeZone: "UTC",
      });
    } catch {
      return new Intl.DateTimeFormat("en-US", {
        weekday: "short",
        day: "numeric",
        timeZone: "UTC",
      });
    }
  });
  const weekdayFormatter = computed(() => {
    try {
      return new Intl.DateTimeFormat(locale.value, { weekday: "short", timeZone: "UTC" });
    } catch {
      return new Intl.DateTimeFormat("en-US", { weekday: "short", timeZone: "UTC" });
    }
  });
  const rangeFormatter = computed(() => {
    const options: Intl.DateTimeFormatOptions = {
      month: "short",
      day: "numeric",
      year: "numeric",
      timeZone: "UTC",
    };
    try {
      return new Intl.DateTimeFormat(locale.value, options);
    } catch {
      return new Intl.DateTimeFormat("en-US", options);
    }
  });

  function timeLabel(minute: number): string {
    return timeFormatter.value.format(Date.UTC(2001, 0, 1, Math.floor(minute / 60), minute % 60));
  }

  const dates = computed<readonly PlainDate[]>(() =>
    anchor.value ? visibleDays(view.value, anchor.value, weekStartsOn.value) : [],
  );

  function dayState(date: PlainDate, outsideMonth: boolean): SchedulerDay {
    const utc = toUtcDate(date);
    return {
      date,
      iso: formatIsoDate(date),
      label: headerFormatter.value.format(utc),
      weekdayLabel: weekdayFormatter.value.format(utc),
      fullLabel: formatters.value.fullDate(date),
      dayLabel: formatters.value.day(date),
      today: isSameDay(date, today.value),
      outsideMonth,
    };
  }

  const days = computed<readonly SchedulerDay[]>(() =>
    dates.value.map((date) =>
      dayState(
        date,
        view.value === "month" && anchor.value !== null && date.month !== anchor.value.month,
      ),
    ),
  );
  const slots = computed<readonly SchedulerSlotRow[]>(() => {
    const rows: SchedulerSlotRow[] = [];
    for (let minute = windowStart.value; minute < windowEnd.value; minute += slotMinutes.value) {
      rows.push({ minute, label: timeLabel(minute) });
    }
    return rows;
  });
  const columns = computed<readonly SchedulerDayColumnState<Data>[]>(() =>
    view.value === "month"
      ? []
      : days.value.map((day) => ({
          day,
          placements: layoutTimeGrid(events.value, day.date, {
            startMinute: windowStart.value,
            endMinute: windowEnd.value,
            minimumMinutes: props.minimumEventMinutes ?? 15,
          }),
        })),
  );
  const allDay = computed(() =>
    view.value === "month"
      ? []
      : layoutRow(events.value, dates.value, (event) => event.allDay === true),
  );
  const weeks = computed<readonly SchedulerWeekState<Data>[]>(() => {
    if (view.value !== "month" || !anchor.value) return [];
    return monthWeeks(dates.value, anchor.value).map((week) => {
      const weekDays = week.map((entry) => dayState(entry.date, entry.outsideMonth));
      const placements = layoutRow(
        events.value,
        week.map((entry) => entry.date),
      );
      const overflow = weekDays.map(
        (_, index) =>
          placements.filter(
            (placement) =>
              placement.lane >= maxLanes.value &&
              placement.startIndex <= index &&
              placement.endIndex >= index,
          ).length,
      );
      return { days: weekDays, placements, overflow };
    });
  });
  const heading = computed(() => {
    const first = dates.value[0];
    const last = dates.value.at(-1);
    const current = anchor.value;
    if (!first || !last || !current) return "";
    if (view.value === "day") return formatters.value.fullDate(current);
    if (view.value === "month") return formatters.value.monthYear(current);
    return rangeFormatter.value.formatRange(toUtcDate(first), toUtcDate(last));
  });
  const pending = computed(() => anchor.value === null);
  const state = computed<SchedulerState>(() => {
    if (disabled.value) return "disabled";
    if (pending.value) return "pending";
    return readOnly.value ? "readonly" : "ready";
  });
  const focusedIso = computed(() => {
    const visible = dates.value;
    const candidate = focusedDate.value ?? anchor.value;
    const match = candidate ? visible.find((date) => isSameDay(date, candidate)) : undefined;
    const fallback =
      match ?? visible.find((date) => view.value !== "month" || date.month === anchor.value?.month);
    return fallback ? formatIsoDate(fallback) : null;
  });
  const focusedMinute = computed(() => {
    const minute = focusedMinuteState.value ?? windowStart.value;
    const lastSlot = slots.value.at(-1)?.minute ?? windowStart.value;
    return Math.min(lastSlot, Math.max(windowStart.value, minute));
  });
  const slotState = computed<SchedulerSlotState<Data>>(() => ({
    view: view.value,
    date: anchor.value,
    today: today.value,
    heading: heading.value,
    days: days.value,
    slots: slots.value,
    columns: columns.value,
    allDay: allDay.value,
    weeks: weeks.value,
    locale: locale.value,
    direction: direction.value,
    disabled: disabled.value,
    readOnly: readOnly.value,
    pending: pending.value,
    state: state.value,
  }));

  function setDate(date: PlainDate): void {
    const normalized = normalizePlainDate(date);
    if (!normalized) return;
    internalDate.value = normalized;
    emit.date(normalized);
  }

  function navigate(step: -1 | 1): boolean {
    const current = anchor.value;
    if (!current || disabled.value) return false;
    const next = shiftAnchor(view.value, current, step);
    focusedDate.value = focusedDate.value ? shiftAnchor(view.value, focusedDate.value, step) : next;
    setDate(next);
    return true;
  }

  function goToToday(): boolean {
    const current = today.value;
    if (!current || disabled.value || isSameDay(current, anchor.value)) return false;
    focusedDate.value = current;
    setDate(current);
    return true;
  }

  function focusCell(): boolean {
    const iso = focusedIso.value;
    if (!iso || !root.value) return false;
    const selector =
      view.value === "month"
        ? `[data-vize-ui="scheduler-day"][data-date="${iso}"]`
        : `[data-vize-ui="scheduler-slot"][data-date="${iso}"][data-minute="${focusedMinute.value}"]`;
    const element = root.value.querySelector(selector);
    if (!(element instanceof HTMLElement)) return false;
    element.focus();
    return true;
  }

  function moveFocus(date: PlainDate, minute: number | null): void {
    focusedDate.value = date;
    if (minute !== null) focusedMinuteState.value = minute;
    const visible = dates.value;
    const first = visible[0];
    const last = visible.at(-1);
    const current = anchor.value;
    const leavesMonth =
      view.value === "month" &&
      current !== null &&
      (date.month !== current.month || date.year !== current.year);
    if (
      leavesMonth ||
      (first && last && (daysBetween(first, date) < 0 || daysBetween(last, date) > 0))
    ) {
      setDate(date);
    }
    void nextTick(() => focusCell());
  }

  function horizontal(event: KeyboardEvent): number {
    const sign = direction.value === "rtl" ? -1 : 1;
    return event.key === "ArrowRight" ? sign : -sign;
  }

  function onSlotKeydown(date: PlainDate, minute: number, event: KeyboardEvent): void {
    if (disabled.value) return;
    const lastSlot = slots.value.at(-1)?.minute ?? windowStart.value;
    switch (event.key) {
      case "ArrowUp":
        moveFocus(date, Math.max(windowStart.value, minute - slotMinutes.value));
        break;
      case "ArrowDown":
        moveFocus(date, Math.min(lastSlot, minute + slotMinutes.value));
        break;
      case "ArrowLeft":
      case "ArrowRight":
        moveFocus(addDays(date, horizontal(event)), minute);
        break;
      case "Home":
        moveFocus(date, windowStart.value);
        break;
      case "End":
        moveFocus(date, lastSlot);
        break;
      case "PageUp":
      case "PageDown":
        moveFocus(shiftAnchor(view.value, date, event.key === "PageUp" ? -1 : 1), minute);
        break;
      case "Enter":
      case " ":
        onSlotActivate(date, minute, event);
        break;
      default:
        return;
    }
    event.preventDefault();
  }

  function onSlotActivate(date: PlainDate, minute: number, event: Event): void {
    if (disabled.value) return;
    focusedDate.value = date;
    focusedMinuteState.value = minute;
    const start = dayDateTime(date, minute);
    emit.slotActivate({ start, end: addMinutes(start, slotMinutes.value) }, event);
  }

  function onDayKeydown(date: PlainDate, event: KeyboardEvent): void {
    if (disabled.value) return;
    switch (event.key) {
      case "ArrowUp":
        moveFocus(addDays(date, -7), null);
        break;
      case "ArrowDown":
        moveFocus(addDays(date, 7), null);
        break;
      case "ArrowLeft":
      case "ArrowRight":
        moveFocus(addDays(date, horizontal(event)), null);
        break;
      case "Home":
        moveFocus(startOfWeek(date, weekStartsOn.value), null);
        break;
      case "End":
        moveFocus(endOfWeek(date, weekStartsOn.value), null);
        break;
      case "PageUp":
      case "PageDown":
        moveFocus(shiftAnchor("month", date, event.key === "PageUp" ? -1 : 1), null);
        break;
      case "Enter":
      case " ":
        onDayActivate(date, event);
        break;
      default:
        return;
    }
    event.preventDefault();
  }

  function onDayActivate(date: PlainDate, event: Event): void {
    if (disabled.value) return;
    focusedDate.value = date;
    emit.dayActivate(date, event);
  }

  function findEvent(eventId: string): SchedulerEvent<Data> | undefined {
    return events.value.find((candidate) => candidate.id === eventId);
  }

  function onEventActivate(eventId: string, event: Event): void {
    const found = findEvent(eventId);
    if (found && !disabled.value) emit.eventActivate(found, event);
  }

  function requestMove(
    found: SchedulerEvent<Data>,
    start: PlainDateTime,
    event: Event | null,
  ): void {
    const duration = Math.max(0, minutesBetween(found.start, found.end));
    emit.eventMove({ event: found, start, end: addMinutes(start, duration) }, event);
  }

  function requestResize(
    found: SchedulerEvent<Data>,
    end: PlainDateTime,
    event: Event | null,
  ): void {
    const minimumEnd = addMinutes(found.start, snapMinutes.value);
    const next = minutesBetween(minimumEnd, end) < 0 ? minimumEnd : end;
    emit.eventResize({ event: found, start: found.start, end: next }, event);
  }

  function onEventKeydown(eventId: string, mode: "time" | "row", event: KeyboardEvent): void {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onEventActivate(eventId, event);
      return;
    }
    if (!event.altKey || disabled.value || readOnly.value) return;
    const found = findEvent(eventId);
    if (!found) return;
    const vertical = mode === "time" ? snapMinutes.value : 7 * 1_440;
    let delta = 0;
    if (event.key === "ArrowUp") delta = -vertical;
    else if (event.key === "ArrowDown") delta = vertical;
    else if (event.key === "ArrowLeft" || event.key === "ArrowRight")
      delta = horizontal(event) * 1_440;
    else return;
    event.preventDefault();
    if (
      event.shiftKey &&
      mode === "time" &&
      (event.key === "ArrowUp" || event.key === "ArrowDown")
    ) {
      requestResize(found, addMinutes(found.end, delta), event);
      return;
    }
    requestMove(found, addMinutes(found.start, delta), event);
  }

  let grabOffset = 0;
  const dragAndDrop = useDragAndDrop<SchedulerDragData>({
    isDisabled: () => disabled.value || readOnly.value,
  });

  function onEventPointerDown(offset: number): void {
    grabOffset = Number.isFinite(offset) ? offset : 0;
  }

  function registerEventSource(
    eventId: MaybeRefOrGetter<string>,
    kind: SchedulerDragKind,
    element: Readonly<ShallowRef<HTMLElement | null>>,
  ) {
    const key = `${kind}:${toValue(eventId)}`;
    return dragAndDrop.registerSource({
      key,
      element,
      keyboard: false,
      payload: () => ({ kind: dragKinds[kind], data: { eventId: toValue(eventId) } }),
      label: () => findEvent(toValue(eventId))?.title ?? toValue(eventId),
    });
  }

  function registerDayTarget(
    date: MaybeRefOrGetter<PlainDate>,
    mode: "time" | "day",
    element: Readonly<ShallowRef<HTMLElement | null>>,
  ) {
    const target = () => toValue(date);
    return dragAndDrop.registerTarget({
      key: `${mode}:${formatIsoDate(target())}`,
      element,
      accepts: (payload) =>
        payload !== null &&
        (payload.kind === dragKinds.move || (mode === "time" && payload.kind === dragKinds.resize)),
      onDrop: (event) => {
        const payload = event.payload;
        const found = payload ? findEvent(payload.data.eventId) : undefined;
        if (!payload || !found) return;
        const day = target();
        if (mode === "day") {
          const offset = daysBetween(found.start, day);
          requestMove(found, addMinutes(found.start, offset * 1_440), event.originalEvent);
          return;
        }
        const rect = element.value?.getBoundingClientRect();
        if (!rect || event.y === null) return;
        const pixelsPerMinute = (rect.bottom - rect.top) / (windowEnd.value - windowStart.value);
        const grabMinutes = pixelsPerMinute > 0 ? grabOffset / pixelsPerMinute : 0;
        const pointerMinute = minuteAtPoint(event.y, rect, {
          startMinute: windowStart.value,
          endMinute: windowEnd.value,
          step: snapMinutes.value,
        });
        if (payload.kind === dragKinds.resize) {
          requestResize(
            found,
            dayDateTime(day, pointerMinute + snapMinutes.value),
            event.originalEvent,
          );
          return;
        }
        const startMinute = Math.max(
          windowStart.value,
          Math.round((pointerMinute - grabMinutes) / snapMinutes.value) * snapMinutes.value,
        );
        requestMove(found, dayDateTime(day, startMinute), event.originalEvent);
      },
    });
  }

  function eventLabel(event: SchedulerEvent<unknown>): string {
    if (event.allDay === true) return event.title;
    const start = timeLabel(event.start.hour * 60 + event.start.minute);
    const end = timeLabel(event.end.hour * 60 + event.end.minute);
    return `${event.title}, ${start} – ${end}`;
  }

  const context: SchedulerContextValue = schedulerContext.provide({
    id,
    headingId,
    root,
    slotState,
    focusedIso,
    focusedMinute,
    windowStart,
    windowEnd,
    slotMinutes,
    maxLanes,
    eventLabel,
    canNavigate: () => anchor.value !== null && !disabled.value,
    navigate,
    goToToday,
    onSlotKeydown,
    onSlotActivate,
    onDayKeydown,
    onDayActivate,
    onEventActivate,
    onEventKeydown,
    onEventPointerDown,
    registerEventSource,
    registerDayTarget,
  });

  const exposed = {
    root,
    view,
    date: anchor,
    today,
    heading,
    days,
    slots,
    columns,
    allDay,
    weeks,
    locale,
    direction,
    disabled,
    readOnly,
    pending,
    state,
    navigate,
    goToToday,
    setView: (next: SchedulerView) => {
      viewState.set(next);
    },
    setDate,
    focus: () => focusCell(),
  } satisfies SchedulerSetupExpose<Data>;

  return {
    context,
    exposed,
    id,
    headingId,
    slotState,
    direction,
    dragAndDrop,
  };
}
