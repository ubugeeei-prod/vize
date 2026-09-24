import { computed, nextTick, shallowRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { resolveLocale, useLocaleValue } from "../../i18n/locale/locale-runtime.ts";
import { calendarContext } from "./calendar-context.ts";
import type { CalendarContextValue } from "./calendar-context.ts";
import { createCalendarFormatters, normalizeWeekday } from "./calendar-locale.ts";
import { useToday } from "./calendar-today.ts";
import type {
  CalendarDayState,
  CalendarDayStateToken,
  CalendarDirection,
  CalendarMonthState,
  CalendarNavigationUnit,
  CalendarRootExposeBase,
  CalendarSelectionMode,
  CalendarSharedProps,
  CalendarSlotState,
  CalendarState,
} from "./calendar-types.ts";
import {
  addDays,
  addMonths,
  addYears,
  clampDate,
  compareDates,
  dayOfWeek,
  daysBetween,
  endOfMonth,
  endOfWeek,
  formatIsoDate,
  isDateWithin,
  isSameDay,
  isSameMonth,
  monthsBetween,
  normalizePlainDate,
  shiftYearMonth,
  startOfMonth,
  startOfWeek,
  toYearMonth,
} from "./plain-date.ts";
import type { PlainDate, PlainYearMonth } from "./plain-date.ts";

/** Props read by the shared calendar runtime; every key is present and may be undefined. */
export type CalendarCoreProps = {
  readonly [Key in keyof CalendarSharedProps]-?: CalendarSharedProps[Key] | undefined;
};

/** Selection-derived flags for one day. */
export interface CalendarDaySelection {
  readonly selected: boolean;
  readonly rangeStart: boolean;
  readonly rangeEnd: boolean;
  readonly inRange: boolean;
  readonly preview: boolean;
}

/** Selection model plugged into the shared calendar runtime by each root. */
export interface CalendarSelectionAdapter {
  readonly mode: CalendarSelectionMode;
  /** Date that seeds the focus target when no focus date is known. */
  readonly anchorDate: () => PlainDate | null;
  /** Whether any value is selected. */
  readonly hasSelection: () => boolean;
  /** Selection flags for one date. */
  readonly daySelection: (date: PlainDate) => CalendarDaySelection;
  /** Commit a user selection. */
  readonly select: (date: PlainDate, event: Event) => void;
  /** Preview target while pointing or moving focus; `null` clears. */
  readonly hover: (date: PlainDate | null) => void;
  /** Cancel a pending gesture; returns whether anything was cancelled. */
  readonly cancel: () => boolean;
}

type FocusedDateEmit = (date: PlainDate) => void;

type CalendarCoreSetupExpose = {
  readonly [Key in keyof CalendarRootExposeBase]: Key extends
    | "focus"
    | "navigate"
    | "setFocusedDate"
    | "setVisibleMonth"
    ? CalendarRootExposeBase[Key]
    : Key extends "root"
      ? Readonly<ShallowRef<HTMLDivElement | null>>
      : ComputedRef<CalendarRootExposeBase[Key]>;
};

const maximumVisibleMonths = 12;

function clampMonthCount(value: number | undefined): number {
  if (value === undefined || !Number.isFinite(value)) return 1;
  return Math.min(maximumVisibleMonths, Math.max(1, Math.trunc(value)));
}

/**
 * Shared calendar state machine for single and range roots.
 *
 * Publishes {@link calendarContext} so CalendarGrid, CalendarHeading, and the
 * navigation controls work with either selection model. Must run in setup.
 */
export function useCalendarCore(
  props: CalendarCoreProps,
  selection: CalendarSelectionAdapter,
  root: Readonly<ShallowRef<HTMLDivElement | null>>,
  emitFocusedDate: FocusedDateEmit,
) {
  const localeValue = useLocaleValue();
  const id = useDeterministicId({ id: () => props.id, hint: "calendar" });
  const headingId = computed(() => deriveDeterministicId(id.value, "heading"));
  const locale = computed(() => resolveLocale(props.locale ?? localeValue.value.locale));
  const direction = computed<CalendarDirection>(() =>
    props.dir === "rtl" || props.dir === "ltr" ? props.dir : localeValue.value.direction,
  );
  const formatters = computed(() =>
    createCalendarFormatters({
      locale: locale.value,
      calendar: props.calendar ?? localeValue.value.calendar,
      numberingSystem: props.numberingSystem ?? localeValue.value.numberingSystem,
    }),
  );
  const labelCache = computed(() => {
    void formatters.value;
    return new Map<string, readonly [string, string]>();
  });
  const weekStartsOn = computed(() => normalizeWeekday(props.weekStartsOn, locale.value));
  const weekdays = computed(() =>
    formatters.value.weekdays(weekStartsOn.value, props.weekdayFormat ?? "short"),
  );
  const bounds = computed(() => {
    const min = normalizePlainDate(props.min);
    const max = normalizePlainDate(props.max);
    return min && max && compareDates(min, max) > 0 ? { min: max, max: min } : { min, max };
  });
  const min = computed(() => bounds.value.min);
  const max = computed(() => bounds.value.max);
  const monthCount = computed(() => clampMonthCount(props.numberOfMonths));
  const disabled = computed(() => props.disabled === true);
  const readOnly = computed(() => props.readOnly === true);
  const today = useToday({
    today: () => props.today,
    now: () => props.now,
    timeZone: () => props.timeZone,
  });
  const internalFocused = shallowRef<PlainDate | null>(null);
  const internalStart = shallowRef<PlainYearMonth | null>(null);
  const focusedDate = computed(() => {
    const candidate =
      normalizePlainDate(props.focusedDate) ??
      internalFocused.value ??
      selection.anchorDate() ??
      today.value;
    return candidate ? clampDate(candidate, min.value, max.value) : null;
  });

  function inWindow(date: PlainYearMonth, start: PlainYearMonth): boolean {
    const offset = monthsBetween(start, date);
    return offset >= 0 && offset < monthCount.value;
  }

  function alignStart(focused: PlainDate, previous: PlainYearMonth | null): PlainYearMonth {
    const month = toYearMonth(focused);
    if (previous && monthsBetween(previous, month) >= monthCount.value) {
      return shiftYearMonth(month, -(monthCount.value - 1));
    }
    return month;
  }

  const visibleStart = computed<PlainYearMonth | null>(() => {
    const focused = focusedDate.value;
    if (!focused) return null;
    const start = internalStart.value;
    return start && inWindow(focused, start) ? start : alignStart(focused, start);
  });

  // Snapshot the aligned window so moving focus back inside it never scrolls the view.
  watch(visibleStart, (start) => {
    internalStart.value = start;
  });
  watch(
    () => {
      const anchor = selection.anchorDate();
      return anchor ? formatIsoDate(anchor) : "";
    },
    () => {
      // Follow external value changes (for example typing into a DateField) into
      // view, but keep keyboard focus where it is when the new value is visible.
      const anchor = selection.anchorDate();
      const start = visibleStart.value;
      if (anchor && !(start && inWindow(anchor, start))) internalFocused.value = anchor;
    },
  );

  function labels(date: PlainDate, iso: string): readonly [string, string] {
    const cache = labelCache.value;
    let value = cache.get(iso);
    if (!value) {
      value = [formatters.value.day(date), formatters.value.fullDate(date)];
      cache.set(iso, value);
    }
    return value;
  }

  function dayState(date: PlainDate, monthIndex: number, outsideMonth: boolean): CalendarDayState {
    const iso = formatIsoDate(date);
    const [label, fullLabel] = labels(date, iso);
    const dayDisabled = disabled.value || !isDateWithin(date, min.value, max.value);
    const unavailable = !dayDisabled && props.isDateUnavailable?.(date) === true;
    const flags = selection.daySelection(date);
    const state: CalendarDayStateToken = outsideMonth
      ? "outside"
      : dayDisabled
        ? "disabled"
        : unavailable
          ? "unavailable"
          : flags.selected
            ? "selected"
            : flags.inRange
              ? "range-middle"
              : "idle";
    return {
      date,
      iso,
      label,
      fullLabel,
      weekday: dayOfWeek(date),
      monthIndex,
      outsideMonth,
      today: isSameDay(date, today.value),
      focused: !outsideMonth && isSameDay(date, focusedDate.value),
      ...flags,
      disabled: dayDisabled,
      unavailable,
      readOnly: readOnly.value,
      state,
    };
  }

  function monthState(month: PlainYearMonth, index: number): CalendarMonthState {
    const first = startOfMonth(month);
    const gridStart = startOfWeek(first, weekStartsOn.value);
    const gridEnd = endOfWeek(endOfMonth(month), weekStartsOn.value);
    const weekCount = props.fixedWeeks === true ? 6 : (daysBetween(gridStart, gridEnd) + 1) / 7;
    const weeks = Array.from({ length: weekCount }, (_, week) =>
      Array.from({ length: 7 }, (_, column) => {
        const date = addDays(gridStart, week * 7 + column);
        return dayState(date, index, !isSameMonth(date, month));
      }),
    );
    return {
      index,
      year: month.year,
      month: month.month,
      label: formatters.value.monthYear(month),
      weeks,
    };
  }

  const months = computed<readonly CalendarMonthState[]>(() => {
    const start = visibleStart.value;
    if (!start) return [];
    return Array.from({ length: monthCount.value }, (_, index) =>
      monthState(shiftYearMonth(start, index), index),
    );
  });
  const heading = computed(() => {
    const start = visibleStart.value;
    if (!start) return "";
    return formatters.value.monthRange(start, shiftYearMonth(start, monthCount.value - 1));
  });
  const pending = computed(() => focusedDate.value === null);
  const state = computed<CalendarState>(() => {
    if (disabled.value) return "disabled";
    if (pending.value) return "pending";
    if (readOnly.value) return "readonly";
    return selection.hasSelection() ? "selected" : "empty";
  });
  const slotState = computed<CalendarSlotState>(() => ({
    mode: selection.mode,
    months: months.value,
    weekdays: weekdays.value,
    heading: heading.value,
    focusedDate: focusedDate.value,
    today: today.value,
    locale: locale.value,
    direction: direction.value,
    weekStartsOn: weekStartsOn.value,
    disabled: disabled.value,
    readOnly: readOnly.value,
    pending: pending.value,
    state: state.value,
  }));

  function dayButton(date: PlainDate): HTMLElement | null {
    const element = root.value?.querySelector(
      `[data-vize-ui="calendar-day"][data-date="${formatIsoDate(date)}"]:not([data-outside-month])`,
    );
    return element instanceof HTMLElement ? element : null;
  }

  function focus(options?: FocusOptions): boolean {
    const date = focusedDate.value;
    const target = date ? dayButton(date) : null;
    if (!target || target.hasAttribute("disabled")) return false;
    target.focus(options);
    return target.ownerDocument.activeElement === target;
  }

  function commitFocusedDate(date: PlainDate, start?: PlainYearMonth): void {
    const next = clampDate(date, min.value, max.value);
    const previous = focusedDate.value;
    internalStart.value = start ?? visibleStart.value;
    internalFocused.value = next;
    internalStart.value = visibleStart.value;
    if (!isSameDay(previous, next)) emitFocusedDate(next);
  }

  function setFocusedDate(date: PlainDate): void {
    const normalized = normalizePlainDate(date);
    if (normalized) commitFocusedDate(normalized);
  }

  function navigationDelta(unit: CalendarNavigationUnit): number {
    if (unit === "year") return 12;
    return props.pagedNavigation === true ? monthCount.value : 1;
  }

  function canNavigate(unit: CalendarNavigationUnit, step: -1 | 1): boolean {
    const start = visibleStart.value;
    if (!start || disabled.value) return false;
    const nextStart = shiftYearMonth(start, navigationDelta(unit) * step);
    const nextEnd = shiftYearMonth(nextStart, monthCount.value - 1);
    if (min.value && compareDates(endOfMonth(nextEnd), min.value) < 0) return false;
    if (max.value && compareDates(startOfMonth(nextStart), max.value) > 0) return false;
    return true;
  }

  function navigate(unit: CalendarNavigationUnit, step: -1 | 1): boolean {
    const start = visibleStart.value;
    const focused = focusedDate.value;
    if (!start || !focused || !canNavigate(unit, step)) return false;
    const delta = navigationDelta(unit) * step;
    commitFocusedDate(addMonths(focused, delta), shiftYearMonth(start, delta));
    return true;
  }

  function setVisibleMonth(month: PlainYearMonth): void {
    if (!Number.isInteger(month.year) || !Number.isInteger(month.month)) return;
    const normalized = shiftYearMonth(month, 0);
    const day = Math.min(focusedDate.value?.day ?? 1, endOfMonth(normalized).day);
    commitFocusedDate(
      Object.freeze({ year: normalized.year, month: normalized.month, day }),
      normalized,
    );
  }

  function moveFocus(target: PlainDate): void {
    commitFocusedDate(target);
    selection.hover(focusedDate.value);
    void nextTick(() => focus());
  }

  function onDayKeydown(date: PlainDate, event: KeyboardEvent): void {
    if (disabled.value) return;
    const rtl = direction.value === "rtl";
    let target: PlainDate | null = null;
    switch (event.key) {
      case "ArrowLeft":
        target = addDays(date, rtl ? 1 : -1);
        break;
      case "ArrowRight":
        target = addDays(date, rtl ? -1 : 1);
        break;
      case "ArrowUp":
        target = addDays(date, -7);
        break;
      case "ArrowDown":
        target = addDays(date, 7);
        break;
      case "Home":
        target = startOfWeek(date, weekStartsOn.value);
        break;
      case "End":
        target = endOfWeek(date, weekStartsOn.value);
        break;
      case "PageUp":
        target = event.shiftKey ? addYears(date, -1) : addMonths(date, -1);
        break;
      case "PageDown":
        target = event.shiftKey ? addYears(date, 1) : addMonths(date, 1);
        break;
      case "Escape":
        if (selection.cancel()) {
          event.preventDefault();
          event.stopPropagation();
        }
        return;
      default:
        return;
    }
    event.preventDefault();
    moveFocus(target);
  }

  function isSelectable(date: PlainDate): boolean {
    return (
      !disabled.value &&
      !readOnly.value &&
      isDateWithin(date, min.value, max.value) &&
      props.isDateUnavailable?.(date) !== true
    );
  }

  function onDayClick(date: PlainDate, event: MouseEvent): void {
    if (disabled.value || !isDateWithin(date, min.value, max.value)) return;
    commitFocusedDate(date);
    if (!isSelectable(date)) return;
    selection.select(date, event);
  }

  function onDayHover(date: PlainDate | null): void {
    if (disabled.value || readOnly.value) return;
    selection.hover(date && isDateWithin(date, min.value, max.value) ? date : null);
  }

  const context: CalendarContextValue = calendarContext.provide({
    root,
    id,
    headingId,
    slotState,
    formatters,
    visibleStart,
    min,
    max,
    canNavigate,
    navigate,
    setVisibleMonth,
    onDayClick,
    onDayKeydown,
    onDayHover,
  });

  const exposed = {
    root,
    focus,
    setFocusedDate,
    setVisibleMonth,
    navigate,
    mode: computed(() => selection.mode),
    months,
    weekdays,
    heading,
    focusedDate,
    today,
    locale,
    direction,
    weekStartsOn,
    disabled,
    readOnly,
    pending,
    state,
  } satisfies CalendarCoreSetupExpose;

  return {
    context,
    exposed,
    id,
    headingId,
    slotState,
    isSelectable,
    direction,
    state,
    min,
    max,
    focusedDate,
    today,
  };
}
