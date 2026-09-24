import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import LocaleProvider from "../../i18n/locale/locale-provider.vue";
import { mountInteraction } from "../../../testing/mount.ts";
import CalendarGrid from "./calendar-grid.vue";
import CalendarHeading from "./calendar-heading.vue";
import CalendarMonthSelect from "./calendar-month-select.vue";
import CalendarNext from "./calendar-next.vue";
import CalendarPrev from "./calendar-prev.vue";
import CalendarRoot from "./calendar-root.vue";
import CalendarYearSelect from "./calendar-year-select.vue";
import type { CalendarRootExpose, CalendarSlotState } from "./calendar-types.ts";
import { createPlainDate, formatIsoDate } from "./plain-date.ts";
import type { PlainDate } from "./plain-date.ts";

const today = createPlainDate(2026, 9, 25);

function day(root: Element, iso: string): HTMLButtonElement {
  const element = root.querySelector(
    `[data-vize-ui="calendar-day"][data-date="${iso}"]:not([data-outside-month])`,
  );
  assert.ok(element instanceof HTMLButtonElement, `missing day ${iso}`);
  return element;
}

async function key(target: Element, value: string, init: KeyboardEventInit = {}): Promise<boolean> {
  const event = new KeyboardEvent("keydown", {
    key: value,
    bubbles: true,
    cancelable: true,
    ...init,
  });
  target.dispatchEvent(event);
  await nextTick();
  await nextTick();
  return event.defaultPrevented;
}

function focusedIso(): string | null {
  const active = document.activeElement;
  return active instanceof HTMLElement ? active.getAttribute("data-date") : null;
}

test("renders an APG date grid with locale weekdays, roving focus, and today", () => {
  const handle = mountInteraction(CalendarRoot, {
    props: { id: "trip", today, locale: "en-US", defaultValue: createPlainDate(2026, 9, 10) },
  });
  const root = handle.root();
  const grid = handle.getByRole("grid", { name: "September 2026" });
  const heading = root.querySelector("[data-vize-ui='calendar-heading']");
  const headers = [...root.querySelectorAll("th")];

  assert.equal(root.getAttribute("role"), "group");
  assert.equal(root.getAttribute("aria-labelledby"), heading?.id);
  assert.equal(root.getAttribute("data-state"), "selected");
  assert.equal(root.getAttribute("data-value"), "2026-09-10");
  assert.equal(heading?.textContent?.trim(), "September 2026");
  assert.equal(heading?.getAttribute("aria-live"), "polite");
  assert.equal(grid.tagName, "TABLE");
  assert.equal(grid.id, "trip-grid-0");
  assert.deepEqual(
    headers.map((header) => header.textContent?.trim()),
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
  );
  assert.equal(headers[0]?.getAttribute("abbr"), "Sunday");
  assert.equal(root.querySelectorAll("[data-vize-ui='calendar-week']").length, 5);

  const selected = day(root, "2026-09-10");
  assert.equal(selected.getAttribute("tabindex"), "0");
  assert.equal(selected.getAttribute("aria-label"), "Thursday, September 10, 2026");
  assert.equal(selected.closest("td")?.getAttribute("aria-selected"), "true");
  assert.equal(selected.getAttribute("data-state"), "selected");
  assert.equal(day(root, "2026-09-11").getAttribute("tabindex"), "-1");
  assert.equal(day(root, "2026-09-25").getAttribute("aria-current"), "date");
  assert.equal(day(root, "2026-09-25").getAttribute("data-today"), "true");

  const outside = root.querySelector<HTMLButtonElement>(
    "[data-date='2026-08-30'][data-vize-ui='calendar-day']",
  );
  assert.equal(outside?.disabled, true);
  assert.equal(outside?.getAttribute("data-outside-month"), "true");
  assert.equal(outside?.closest("td")?.getAttribute("aria-selected"), null);
  handle.unmount();
});

test("uncontrolled selection emits update and change with the previous date", async () => {
  const handle = mountInteraction(CalendarRoot, {
    props: { today, locale: "en-US", name: "departure" },
    record: ["update:modelValue", "change"],
  });
  const root = handle.root();
  await handle.click(day(root, "2026-09-14"));

  assert.deepEqual(
    handle.recorded().map((entry) => [entry.event, formatIsoDate(entry.payload[0] as PlainDate)]),
    [
      ["update:modelValue", "2026-09-14"],
      ["change", "2026-09-14"],
    ],
  );
  assert.equal(handle.recorded()[1]?.payload[1], null);
  assert.ok(handle.recorded()[1]?.payload[2] instanceof MouseEvent);
  assert.equal(day(root, "2026-09-14").getAttribute("data-selected"), "true");
  assert.equal(root.querySelector<HTMLInputElement>("input[type='hidden']")?.value, "2026-09-14");
  assert.equal(root.querySelector<HTMLInputElement>("input[type='hidden']")?.name, "departure");
  handle.unmount();
});

test("controlled value wins until the parent accepts the request", async () => {
  const handle = mountInteraction(CalendarRoot, {
    props: { today, locale: "en-US", modelValue: createPlainDate(2026, 9, 3) },
    record: ["update:modelValue"],
  });
  const root = handle.root();
  await handle.click(day(root, "2026-09-20"));
  assert.equal(handle.recorded().length, 1);
  assert.equal(day(root, "2026-09-03").getAttribute("data-selected"), "true");
  assert.equal(day(root, "2026-09-20").getAttribute("data-selected"), null);

  await handle.wrapper.setProps({ modelValue: createPlainDate(2026, 9, 20) });
  assert.equal(day(root, "2026-09-20").getAttribute("data-selected"), "true");
  handle.unmount();
});

test("keyboard moves focus by day, week, week edge, month, and year across months", async () => {
  const handle = mountInteraction(CalendarRoot, {
    props: { today, locale: "en-US" },
    record: ["update:focusedDate"],
  });
  const root = handle.root();
  day(root, "2026-09-25").focus();

  assert.equal(await key(day(root, "2026-09-25"), "ArrowRight"), true);
  assert.equal(focusedIso(), "2026-09-26");
  await key(day(root, "2026-09-26"), "ArrowDown");
  assert.equal(focusedIso(), "2026-10-03");
  assert.equal(
    root.querySelector("[data-vize-ui='calendar-grid']")?.getAttribute("data-month"),
    "2026-10",
  );
  await key(day(root, "2026-10-03"), "ArrowUp");
  assert.equal(focusedIso(), "2026-09-26");
  await key(day(root, "2026-09-26"), "ArrowLeft");
  assert.equal(focusedIso(), "2026-09-25");
  await key(day(root, "2026-09-25"), "Home");
  assert.equal(focusedIso(), "2026-09-20");
  await key(day(root, "2026-09-20"), "End");
  assert.equal(focusedIso(), "2026-09-26");
  await key(day(root, "2026-09-26"), "PageDown");
  assert.equal(focusedIso(), "2026-10-26");
  await key(day(root, "2026-10-26"), "PageUp", { shiftKey: true });
  assert.equal(focusedIso(), "2025-10-26");
  assert.equal(handle.getByRole("grid").getAttribute("aria-label"), "October 2025");
  assert.equal(await key(day(root, "2025-10-26"), "Tab"), false);
  assert.equal(
    handle
      .recorded()
      .map((entry) => formatIsoDate(entry.payload[0] as PlainDate))
      .at(-1),
    "2025-10-26",
  );
  handle.unmount();
});

test("RTL flips horizontal arrows and locale week start follows the region", async () => {
  const handle = mountInteraction(CalendarRoot, {
    props: { today, locale: "ar-EG", dir: "rtl", weekdayFormat: "narrow" },
  });
  const root = handle.root();
  assert.equal(root.getAttribute("dir"), "rtl");
  assert.equal(root.querySelector("th")?.getAttribute("data-weekday"), "6");
  day(root, "2026-09-25").focus();
  await key(day(root, "2026-09-25"), "ArrowLeft");
  assert.equal(focusedIso(), "2026-09-26");
  await key(day(root, "2026-09-26"), "ArrowRight");
  assert.equal(focusedIso(), "2026-09-25");
  handle.unmount();

  const german = mountInteraction(CalendarRoot, { props: { today, locale: "de-DE" } });
  const headers = [...german.root().querySelectorAll("th")].map((header) =>
    header.getAttribute("data-weekday"),
  );
  assert.deepEqual(headers, ["1", "2", "3", "4", "5", "6", "0"]);
  assert.equal(german.getByRole("grid").getAttribute("aria-label"), "September 2026");
  german.unmount();

  const override = mountInteraction(CalendarRoot, {
    props: { today, locale: "en-US", weekStartsOn: 3 },
  });
  assert.equal(override.root().querySelector("th")?.getAttribute("data-weekday"), "3");
  override.unmount();
});

test("min, max, and unavailable dates constrain focus, navigation, and selection", async () => {
  const handle = mountInteraction(CalendarRoot, {
    props: {
      today,
      locale: "en-US",
      min: createPlainDate(2026, 9, 10),
      max: createPlainDate(2026, 10, 5),
      isDateUnavailable: (date: PlainDate) => date.day === 15,
    },
    record: ["update:modelValue"],
  });
  const root = handle.root();
  const prev = root.querySelector<HTMLButtonElement>("[data-vize-ui='calendar-prev']");
  const next = root.querySelector<HTMLButtonElement>("[data-vize-ui='calendar-next']");
  assert.equal(prev?.disabled, true);
  assert.equal(next?.disabled, false);
  assert.equal(day(root, "2026-09-09").disabled, true);
  assert.equal(day(root, "2026-09-09").getAttribute("data-state"), "disabled");
  const unavailable = day(root, "2026-09-15");
  assert.equal(unavailable.disabled, false);
  assert.equal(unavailable.getAttribute("aria-disabled"), "true");
  assert.equal(unavailable.getAttribute("data-unavailable"), "true");

  await handle.click(unavailable);
  assert.equal(handle.recorded().length, 0);
  assert.equal(unavailable.getAttribute("tabindex"), "0");

  day(root, "2026-09-15").focus();
  await key(day(root, "2026-09-15"), "Home");
  await key(day(root, "2026-09-13"), "ArrowUp");
  assert.equal(focusedIso(), "2026-09-10");
  await key(day(root, "2026-09-10"), "PageDown", { shiftKey: true });
  assert.equal(focusedIso(), "2026-10-05");
  assert.equal(next?.disabled, true);
  assert.equal(prev?.disabled, false);
  handle.unmount();
});

test("previous and next controls page by month or year and keep focus date in view", async () => {
  const handle = mountInteraction(CalendarRoot, {
    props: { today, locale: "en-US", numberOfMonths: 2, pagedNavigation: true },
    record: ["update:focusedDate"],
  });
  const root = handle.root();
  const grids = () =>
    [...root.querySelectorAll("[data-vize-ui='calendar-grid']")].map((grid) =>
      grid.getAttribute("aria-label"),
    );
  assert.deepEqual(grids(), ["September 2026", "October 2026"]);
  assert.equal(
    root.querySelector("[data-vize-ui='calendar-heading']")?.textContent?.trim(),
    "September\u2009–\u2009October 2026",
  );
  assert.equal(root.getAttribute("data-months"), "2");

  await handle.click(root.querySelector("[data-vize-ui='calendar-next']") as Element);
  assert.deepEqual(grids(), ["November 2026", "December 2026"]);
  assert.equal(formatIsoDate(handle.recorded().at(-1)?.payload[0] as PlainDate), "2026-11-25");
  await handle.click(root.querySelector("[data-vize-ui='calendar-prev']") as Element);
  assert.deepEqual(grids(), ["September 2026", "October 2026"]);

  day(root, "2026-10-25").focus();
  await key(day(root, "2026-10-25"), "ArrowDown");
  assert.deepEqual(grids(), ["October 2026", "November 2026"]);
  await key(day(root, "2026-11-01"), "ArrowUp");
  assert.deepEqual(grids(), ["October 2026", "November 2026"]);
  handle.unmount();

  const Years = defineComponent({
    setup: () => () =>
      h(CalendarRoot, { today, locale: "en-US" }, () => [
        h(CalendarPrev, { unit: "year" }),
        h(CalendarHeading),
        h(CalendarNext, { unit: "year", ariaLabel: "Following year" }),
        h(CalendarGrid),
      ]),
  });
  const years = mountInteraction(Years);
  const yearNext = years.getByRole("button", { name: "Following year" });
  assert.equal(
    years.getByRole("button", { name: "Previous year" }).getAttribute("data-unit"),
    "year",
  );
  await years.click(yearNext);
  assert.equal(years.getByRole("grid").getAttribute("aria-label"), "September 2027");
  years.unmount();
});

test("month and year selects jump the view and respect bounds", async () => {
  const Selects = defineComponent({
    setup: () => () =>
      h(
        CalendarRoot,
        {
          today,
          locale: "en-US",
          min: createPlainDate(2025, 3, 1),
          max: createPlainDate(2027, 6, 30),
        },
        () => [h(CalendarMonthSelect), h(CalendarYearSelect), h(CalendarGrid)],
      ),
  });
  const handle = mountInteraction(Selects);
  const root = handle.root();
  const month = root.querySelector<HTMLSelectElement>("[data-vize-ui='calendar-month-select']");
  const year = root.querySelector<HTMLSelectElement>("[data-vize-ui='calendar-year-select']");
  assert.ok(month && year);
  assert.equal(month.value, "9");
  assert.equal(month.getAttribute("aria-label"), "Month");
  assert.equal(month.options[0]?.textContent?.trim(), "January");
  assert.deepEqual(
    [...year.options].map((option) => option.value),
    ["2025", "2026", "2027"],
  );

  month.value = "2";
  month.dispatchEvent(new Event("change"));
  await nextTick();
  assert.equal(handle.getByRole("grid").getAttribute("aria-label"), "February 2026");
  assert.equal(day(root, "2026-02-25").getAttribute("tabindex"), "0");

  year.value = "2025";
  year.dispatchEvent(new Event("change"));
  await nextTick();
  assert.equal(handle.getByRole("grid").getAttribute("aria-label"), "March 2025");
  assert.equal(day(root, "2025-03-01").getAttribute("tabindex"), "0");
  assert.equal(month.options[1]?.disabled, true);
  handle.unmount();
});

test("disabled and read-only calendars keep availability semantics", async () => {
  const disabled = mountInteraction(CalendarRoot, {
    props: { today, locale: "en-US", disabled: true },
    record: ["update:modelValue"],
  });
  assert.equal(disabled.root().getAttribute("data-state"), "disabled");
  assert.equal(disabled.root().getAttribute("aria-disabled"), "true");
  assert.equal(day(disabled.root(), "2026-09-10").disabled, true);
  assert.equal(
    disabled.root().querySelector<HTMLButtonElement>("[data-vize-ui='calendar-next']")?.disabled,
    true,
  );
  disabled.unmount();

  const readOnly = mountInteraction(CalendarRoot, {
    props: { today, locale: "en-US", readOnly: true },
    record: ["update:modelValue"],
  });
  const root = readOnly.root();
  assert.equal(root.getAttribute("data-state"), "readonly");
  assert.equal(readOnly.getByRole("grid").getAttribute("aria-readonly"), "true");
  await readOnly.click(day(root, "2026-09-10"));
  assert.equal(readOnly.recorded().length, 0);
  assert.equal(day(root, "2026-09-10").getAttribute("tabindex"), "0");
  assert.equal(day(root, "2026-09-10").getAttribute("aria-disabled"), "true");
  readOnly.unmount();
});

test("without today or now the calendar is pending until mount reads the injected time zone clock", async () => {
  const handle = mountInteraction(CalendarRoot, {
    props: { locale: "en-US", timeZone: "Pacific/Kiritimati" },
  });
  await nextTick();
  const root = handle.root();
  assert.equal(root.getAttribute("data-pending"), null);
  assert.equal(root.querySelectorAll("[data-today='true']").length, 1);
  handle.unmount();

  const clock = mountInteraction(CalendarRoot, {
    props: { locale: "en-US", now: () => Date.UTC(2026, 0, 31, 23, 30), timeZone: "Asia/Tokyo" },
  });
  assert.equal(
    clock.root().querySelector("[data-today='true']")?.getAttribute("data-date"),
    "2026-02-01",
  );
  clock.unmount();
});

test("LocaleProvider supplies locale, direction, calendar display, and numbering", () => {
  const Probe = defineComponent({
    setup: () => () =>
      h(LocaleProvider, { locale: "ja-JP", calendar: "japanese" }, () =>
        h(CalendarRoot, { today }),
      ),
  });
  const handle = mountInteraction(Probe);
  const heading =
    handle.root().querySelector("[data-vize-ui='calendar-heading']")?.textContent ?? "";
  assert.match(heading, /令和8年9月/u);
  assert.equal(handle.root().querySelector("th")?.getAttribute("data-weekday"), "0");
  handle.unmount();
});

test("exposes focus, setValue, setFocusedDate, setVisibleMonth, and navigate", async () => {
  const handle = mountInteraction(CalendarRoot, { props: { today, locale: "en-US" } });
  const api = handle.exposes<CalendarRootExpose>();
  const root = handle.root();
  assert.equal(api.focus(), true);
  assert.equal(focusedIso(), "2026-09-25");
  assert.equal(api.setValue(createPlainDate(2026, 12, 24)), true);
  await nextTick();
  assert.equal(formatIsoDate(api.value as PlainDate), "2026-12-24");
  assert.equal(handle.getByRole("grid").getAttribute("aria-label"), "December 2026");
  api.setFocusedDate(createPlainDate(2027, 2, 3));
  await nextTick();
  assert.equal(day(root, "2027-02-03").getAttribute("tabindex"), "0");
  api.setVisibleMonth({ year: 2030, month: 5 });
  await nextTick();
  assert.equal(handle.getByRole("grid").getAttribute("aria-label"), "May 2030");
  assert.equal(api.navigate("year", -1), true);
  await nextTick();
  assert.equal(handle.getByRole("grid").getAttribute("aria-label"), "May 2029");
  assert.equal(api.months.length, 1);
  assert.equal(api.state, "selected");
  handle.unmount();
});

test("custom composition receives typed slot state and renders fixed weeks", () => {
  const seen = ref<CalendarSlotState | null>(null);
  const Probe = defineComponent({
    setup: () => () =>
      h(
        CalendarRoot,
        { today, locale: "en-US", fixedWeeks: true, numberOfMonths: 2 },
        {
          default: (state: CalendarSlotState) => {
            seen.value = state;
            return state.months.map((month) =>
              h(
                CalendarGrid,
                { key: month.index, monthIndex: month.index },
                {
                  day: (cell: { label: string; outsideMonth: boolean }) =>
                    cell.outsideMonth ? "" : `d${cell.label}`,
                },
              ),
            );
          },
        },
      ),
  });
  const handle = mountInteraction(Probe);
  assert.equal(seen.value?.months.length, 2);
  assert.equal(seen.value?.weekStartsOn, 0);
  assert.equal(
    handle.root().querySelectorAll("[data-month-index='1'] [data-vize-ui='calendar-week']").length,
    6,
  );
  assert.equal(day(handle.root(), "2026-10-01").textContent, "d1");
  handle.unmount();
});
