import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import { createPlainDate, formatIsoDate } from "../calendar/plain-date.ts";
import type { PlainDate } from "../calendar/plain-date.ts";
import { createPlainDateTime, formatIsoDateTime } from "../datetime-field/plain-date-time.ts";
import type { SchedulerEvent } from "./scheduler-layout.ts";
import SchedulerDayColumn from "./scheduler-day-column.vue";
import SchedulerEventItem from "./scheduler-event.vue";
import SchedulerHeading from "./scheduler-heading.vue";
import SchedulerMonthDay from "./scheduler-month-day.vue";
import SchedulerMonthGrid from "./scheduler-month-grid.vue";
import SchedulerNav from "./scheduler-nav.vue";
import SchedulerRoot from "./scheduler-root.vue";
import SchedulerTimeGrid from "./scheduler-time-grid.vue";
import type {
  SchedulerEventChange,
  SchedulerRootExpose,
  SchedulerSlotRange,
} from "./scheduler-types.ts";

interface Meeting {
  readonly room: string;
}

const today = createPlainDate(2026, 9, 25);

function meeting(
  id: string,
  day: number,
  start: [number, number],
  end: [number, number],
  extra: Partial<SchedulerEvent<Meeting>> = {},
): SchedulerEvent<Meeting> {
  return {
    id,
    title: id,
    data: { room: `room-${id}` },
    start: createPlainDateTime(2026, 9, day, start[0], start[1]),
    end: createPlainDateTime(2026, 9, day, end[0], end[1]),
    ...extra,
  };
}

const events: readonly SchedulerEvent<Meeting>[] = [
  meeting("standup", 25, [9, 0], [9, 30]),
  meeting("review", 25, [9, 0], [10, 0]),
  meeting("lunch", 24, [12, 0], [13, 0]),
  meeting("offsite", 22, [0, 0], [0, 0], {
    allDay: true,
    end: createPlainDateTime(2026, 9, 24, 0, 0),
  }),
];

function changeOf(
  entry: { readonly payload: readonly unknown[] } | undefined,
): SchedulerEventChange<Meeting> {
  assert.ok(entry, "missing recorded emit");
  return entry.payload[0] as SchedulerEventChange<Meeting>;
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

function slot(root: Element, iso: string, minute: number): HTMLButtonElement {
  const element = root.querySelector(
    `[data-vize-ui="scheduler-slot"][data-date="${iso}"][data-minute="${minute}"]`,
  );
  assert.ok(element instanceof HTMLButtonElement, `missing slot ${iso} ${minute}`);
  return element;
}

function eventElement(root: Element, id: string): HTMLElement {
  const element = root.querySelector(`[data-vize-ui="scheduler-event"][data-event-id="${id}"]`);
  assert.ok(element instanceof HTMLElement, `missing event ${id}`);
  return element;
}

test("week view renders an APG slot grid, day columns, laid-out events, and all-day lanes", () => {
  const handle = mountInteraction(SchedulerRoot, {
    props: {
      events,
      today,
      locale: "en-US",
      dayStartHour: 8,
      dayEndHour: 18,
      ariaLabel: "Team calendar",
    },
  });
  const root = handle.root();
  assert.equal(root.getAttribute("role"), "region");
  assert.equal(root.getAttribute("data-view"), "week");
  assert.equal(root.getAttribute("data-state"), "ready");
  const heading = root.querySelector("[data-vize-ui='scheduler-heading']");
  assert.equal(heading?.textContent?.trim(), "Sep 20\u2009–\u200926, 2026");
  const grid = root.querySelector("table[role='grid']");
  assert.equal(grid?.getAttribute("aria-labelledby"), heading?.id);
  const headers = [...root.querySelectorAll("[data-vize-ui='scheduler-day-header']")];
  assert.equal(headers.length, 7);
  assert.equal(headers[5]?.getAttribute("aria-current"), "date");
  assert.equal(headers[5]?.textContent?.trim(), "25 Fri");
  assert.equal(root.querySelectorAll("[data-vize-ui='scheduler-slot-row']").length, 20);
  assert.equal(
    root.querySelector("[data-vize-ui='scheduler-time']")?.textContent?.trim(),
    "8:00 AM",
  );
  assert.equal(slot(root, "2026-09-25", 480).getAttribute("tabindex"), "0");
  assert.equal(slot(root, "2026-09-20", 480).getAttribute("tabindex"), "-1");
  assert.equal(
    slot(root, "2026-09-25", 480).getAttribute("aria-label"),
    "Friday, September 25, 2026 8:00 AM",
  );

  const review = eventElement(root, "review");
  const standup = eventElement(root, "standup");
  assert.equal(
    review.closest("[data-vize-ui='scheduler-day-column']")?.getAttribute("data-date"),
    "2026-09-25",
  );
  assert.equal(review.getAttribute("aria-label"), "review, 9:00 AM – 10:00 AM");
  assert.equal(review.style.getPropertyValue("--vize-scheduler-column"), "0");
  assert.equal(standup.style.getPropertyValue("--vize-scheduler-column"), "1");
  assert.equal(standup.style.getPropertyValue("--vize-scheduler-columns"), "2");
  assert.equal(review.style.getPropertyValue("--vize-scheduler-top"), "10%");
  assert.ok(review.querySelector("[data-vize-ui='scheduler-event-resize']"));
  const offsite = eventElement(root, "offsite");
  assert.equal(
    offsite.closest("[data-vize-ui='scheduler-all-day']")?.getAttribute("aria-label"),
    "All day",
  );
  assert.equal(offsite.style.getPropertyValue("--vize-scheduler-start"), "3");
  assert.equal(offsite.style.getPropertyValue("--vize-scheduler-end"), "5");
  handle.unmount();
});

test("slot keyboard navigation moves by slot, day, edges, and period, and activation emits a range", async () => {
  const handle = mountInteraction(SchedulerRoot, {
    props: { events, today, locale: "en-US", dayStartHour: 8, dayEndHour: 10, weekStartsOn: 1 },
    record: ["slot-activate", "update:date"],
  });
  const root = handle.root();
  slot(root, "2026-09-21", 480).focus();
  await key(slot(root, "2026-09-21", 480), "ArrowDown");
  assert.equal(document.activeElement, slot(root, "2026-09-21", 510));
  await key(slot(root, "2026-09-21", 510), "ArrowRight");
  assert.equal(document.activeElement, slot(root, "2026-09-22", 510));
  await key(slot(root, "2026-09-22", 510), "End");
  assert.equal(document.activeElement, slot(root, "2026-09-22", 570));
  await key(slot(root, "2026-09-22", 570), "Home");
  await key(slot(root, "2026-09-22", 480), "ArrowLeft");
  await key(slot(root, "2026-09-21", 480), "ArrowLeft");
  assert.equal(document.activeElement, slot(root, "2026-09-20", 480));
  assert.equal(formatIsoDate(handle.recorded().at(-1)?.payload[0] as PlainDate), "2026-09-20");
  await key(slot(root, "2026-09-20", 480), "PageDown");
  assert.equal(document.activeElement, slot(root, "2026-09-27", 480));
  await handle.press(slot(root, "2026-09-27", 480), "Enter");
  const activation = handle
    .recorded()
    .filter((entry) => entry.event === "slot-activate")
    .at(-1);
  const range = activation?.payload[0] as SchedulerSlotRange;
  assert.equal(formatIsoDateTime(range.start), "2026-09-27T08:00");
  assert.equal(formatIsoDateTime(range.end), "2026-09-27T08:30");
  handle.unmount();
});

test("events activate and Alt+Arrow keys request typed moves and resizes", async () => {
  const handle = mountInteraction(SchedulerRoot, {
    props: { events, today, locale: "en-US", snapMinutes: 15 },
    record: ["event-activate", "event-move", "event-resize"],
  });
  const root = handle.root();
  const review = eventElement(root, "review");
  await handle.click(review);
  const activated = handle.recorded()[0]?.payload[0] as SchedulerEvent<Meeting>;
  assert.equal(activated.data.room, "room-review");
  await key(review, "ArrowDown", { altKey: true });
  const moved = handle.recorded().at(-1)?.payload[0] as SchedulerEventChange<Meeting>;
  assert.equal(formatIsoDateTime(moved.start), "2026-09-25T09:15");
  assert.equal(formatIsoDateTime(moved.end), "2026-09-25T10:15");
  await key(review, "ArrowRight", { altKey: true });
  const nextDay = handle.recorded().at(-1)?.payload[0] as SchedulerEventChange<Meeting>;
  assert.equal(formatIsoDateTime(nextDay.start), "2026-09-26T09:00");
  await key(review, "ArrowUp", { altKey: true, shiftKey: true });
  const resized = handle.recorded().at(-1);
  assert.equal(resized?.event, "event-resize");
  assert.equal(formatIsoDateTime(changeOf(resized).end), "2026-09-25T09:45");
  await handle.wrapper.setProps({ events: [meeting("review", 25, [9, 0], [9, 15])] });
  await key(eventElement(root, "review"), "ArrowUp", { altKey: true, shiftKey: true });
  assert.equal(formatIsoDateTime(changeOf(handle.recorded().at(-1)).end), "2026-09-25T09:15");
  assert.equal(await key(eventElement(root, "review"), "ArrowDown"), false);
  await handle.wrapper.setProps({ readOnly: true });
  const count = handle.recorded().length;
  await key(review, "ArrowDown", { altKey: true });
  assert.equal(handle.recorded().length, count);
  assert.equal(
    eventElement(root, "review").querySelector("[data-vize-ui='scheduler-event-resize']"),
    null,
  );
  handle.unmount();
});

test("pointer drags move and resize events through drop targets", async () => {
  const handle = mountInteraction(SchedulerRoot, {
    props: { events, today, locale: "en-US", view: "day", dayStartHour: 8, dayEndHour: 12 },
    record: ["event-move", "event-resize"],
  });
  const root = handle.root();
  const column = root.querySelector<HTMLElement>("[data-vize-ui='scheduler-day-column']");
  assert.ok(column);
  column.getBoundingClientRect = () => new DOMRect(0, 0, 100, 240);
  const review = eventElement(root, "review");
  review.getBoundingClientRect = () => new DOMRect(0, 60, 50, 60);
  const pointer = (type: string, x: number, y: number) =>
    new PointerEvent(type, {
      bubbles: true,
      cancelable: true,
      clientX: x,
      clientY: y,
      pointerId: 1,
      pointerType: "mouse",
      button: 0,
      isPrimary: true,
    });
  review.dispatchEvent(pointer("pointerdown", 10, 70));
  document.dispatchEvent(pointer("pointermove", 10, 100));
  document.dispatchEvent(pointer("pointermove", 10, 130));
  document.dispatchEvent(pointer("pointerup", 10, 130));
  await nextTick();
  const moved = handle.recorded().at(-1);
  assert.equal(moved?.event, "event-move");
  assert.equal(formatIsoDateTime(changeOf(moved).start), "2026-09-25T10:00");
  assert.equal(formatIsoDateTime(changeOf(moved).end), "2026-09-25T11:00");

  const handleElement = review.querySelector<HTMLElement>(
    "[data-vize-ui='scheduler-event-resize']",
  );
  assert.ok(handleElement);
  handleElement.dispatchEvent(pointer("pointerdown", 10, 119));
  document.dispatchEvent(pointer("pointermove", 10, 150));
  document.dispatchEvent(pointer("pointermove", 10, 185));
  document.dispatchEvent(pointer("pointerup", 10, 185));
  await nextTick();
  const resized = handle.recorded().at(-1);
  assert.equal(resized?.event, "event-resize");
  assert.equal(formatIsoDateTime(changeOf(resized).end), "2026-09-25T11:30");
  handle.unmount();
});

test("month view lays out week rows, overflow, keyboard day navigation, and day drops", async () => {
  const busy = [1, 2, 3, 4].map((index) =>
    meeting(`m${index}`, 16, [9 + index, 0], [10 + index, 0]),
  );
  const handle = mountInteraction(SchedulerRoot, {
    props: { events: [...events, ...busy], today, locale: "en-US", view: "month", maxLanes: 2 },
    record: ["day-activate", "event-move", "update:date"],
  });
  const root = handle.root();
  assert.equal(
    root.querySelector("[data-vize-ui='scheduler-heading']")?.textContent?.trim(),
    "September 2026",
  );
  assert.equal(root.querySelectorAll("[data-vize-ui='scheduler-week']").length, 5);
  assert.deepEqual(
    [...root.querySelectorAll("[data-vize-ui='scheduler-weekday']")].map((cell) =>
      cell.textContent?.trim(),
    ),
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
  );
  const cell16 = root.querySelector(
    "[data-vize-ui='scheduler-month-cell'][data-date='2026-09-16']",
  );
  assert.equal(cell16?.querySelectorAll("[data-vize-ui='scheduler-event']").length, 2);
  assert.equal(cell16?.querySelector("[data-vize-ui='scheduler-more']")?.textContent?.trim(), "+2");
  assert.equal(
    root
      .querySelector("[data-date='2026-08-30'][data-vize-ui='scheduler-month-cell']")
      ?.getAttribute("data-outside-month"),
    "true",
  );

  const day25 = root.querySelector<HTMLButtonElement>(
    "[data-vize-ui='scheduler-day'][data-date='2026-09-25']",
  );
  assert.equal(day25?.getAttribute("tabindex"), "0");
  day25?.focus();
  await key(day25 as HTMLElement, "ArrowDown");
  assert.equal(document.activeElement?.getAttribute("data-date"), "2026-10-02");
  assert.equal(
    root.querySelector("[data-vize-ui='scheduler-heading']")?.textContent?.trim(),
    "October 2026",
  );
  await handle.press(document.activeElement as HTMLElement, "Enter");
  assert.equal(
    formatIsoDate(
      handle.recorded().find((entry) => entry.event === "day-activate")?.payload[0] as PlainDate,
    ),
    "2026-10-02",
  );
  await key(document.activeElement as HTMLElement, "PageUp");
  const lunch = eventElement(root, "lunch");
  await key(lunch, "ArrowDown", { altKey: true });
  const moved = handle.recorded().at(-1)?.payload[0] as SchedulerEventChange<Meeting>;
  assert.equal(formatIsoDateTime(moved.start), "2026-10-01T12:00");
  handle.unmount();
});

test("navigation controls, controlled view, pending without a clock, and the exposed API", async () => {
  const Composed = defineComponent({
    setup: () => () =>
      h(SchedulerRoot<Meeting>, { events, today, locale: "en-US", defaultView: "day" }, () => [
        h(SchedulerNav, { action: "previous" }),
        h(SchedulerNav, { action: "next", ariaLabel: "Following day" }),
        h(SchedulerNav, { action: "today" }),
        h(SchedulerHeading),
        h(SchedulerTimeGrid),
      ]),
  });
  const handle = mountInteraction(Composed);
  const root = handle.root();
  const heading = () =>
    root.querySelector("[data-vize-ui='scheduler-heading']")?.textContent?.trim();
  assert.equal(heading(), "Friday, September 25, 2026");
  await handle.click(handle.getByRole("button", { name: "Following day" }));
  assert.equal(heading(), "Saturday, September 26, 2026");
  await handle.click(handle.getByRole("button", { name: "Today" }));
  assert.equal(heading(), "Friday, September 25, 2026");
  await handle.click(handle.getByRole("button", { name: "Previous period" }));
  assert.equal(heading(), "Thursday, September 24, 2026");
  handle.unmount();

  const pending = mountInteraction(SchedulerRoot, { props: { locale: "en-US", timeZone: "UTC" } });
  await nextTick();
  assert.equal(pending.root().getAttribute("data-state"), "ready");
  pending.unmount();

  const controlled = mountInteraction(SchedulerRoot, {
    props: { events, today, locale: "en-US", view: "week" },
    record: ["update:view"],
  });
  const api = controlled.exposes<SchedulerRootExpose<Meeting>>();
  api.setView("month");
  assert.deepEqual(controlled.recorded()[0]?.payload, ["month"]);
  assert.equal(api.view, "week");
  assert.equal(api.columns.length, 7);
  api.setDate(createPlainDate(2027, 1, 1));
  await nextTick();
  assert.equal(api.days[0]?.iso, "2026-12-27");
  assert.equal(api.navigate(-1), true);
  await nextTick();
  assert.equal(api.focus(), true);
  assert.equal(api.goToToday(), true);
  controlled.unmount();

  const disabled = mountInteraction(SchedulerRoot, {
    props: { events, today, locale: "en-US", disabled: true },
  });
  assert.equal(disabled.root().getAttribute("data-state"), "disabled");
  assert.equal(
    disabled.root().querySelector<HTMLButtonElement>("[data-vize-ui='scheduler-slot']")?.disabled,
    true,
  );
  disabled.unmount();
});

test("custom composition exposes typed slot state and individual parts", () => {
  let rooms: string[] = [];
  const Composed = defineComponent({
    setup: () => () =>
      h(
        SchedulerRoot<Meeting>,
        { events, today, locale: "en-US", view: "month" },
        {
          default: (state: {
            readonly weeks: readonly {
              readonly placements: readonly { readonly event: SchedulerEvent<Meeting> }[];
            }[];
          }) => {
            rooms = state.weeks.flatMap((week) =>
              week.placements.map((placement) => placement.event.data.room),
            );
            return h(SchedulerMonthGrid, null, {
              event: (placement: { event: { title: string } }) => `★${placement.event.title}`,
            });
          },
        },
      ),
  });
  const handle = mountInteraction(Composed);
  assert.ok(rooms.includes("room-lunch"));
  assert.match(eventElement(handle.root(), "lunch").textContent ?? "", /★lunch/);
  handle.unmount();
  void SchedulerDayColumn;
  void SchedulerEventItem;
  void SchedulerMonthDay;
});
