import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import type { Component } from "vue";
import { renderToString } from "vue/server-renderer";

import CalendarGrid from "./calendar-grid.vue";
import CalendarHeading from "./calendar-heading.vue";
import CalendarMonthSelect from "./calendar-month-select.vue";
import CalendarRoot from "./calendar-root.vue";
import CalendarYearSelect from "./calendar-year-select.vue";
import { createPlainDate } from "./plain-date.ts";

const today = createPlainDate(2026, 9, 25);

const SsrProbe = defineComponent({
  name: "CalendarSsrProbe",
  setup: () => () =>
    h(
      CalendarRoot,
      {
        today,
        locale: "en-US",
        defaultValue: createPlainDate(2026, 9, 10),
        numberOfMonths: 2,
        name: "trip",
      },
      () => [
        h(CalendarHeading),
        h(CalendarMonthSelect),
        h(CalendarYearSelect),
        h(CalendarGrid, { monthIndex: 0 }),
        h(CalendarGrid, { monthIndex: 1 }),
      ],
    ),
});

async function hydrate(component: Component): Promise<{
  readonly host: HTMLElement;
  readonly diagnostics: readonly string[];
  readonly serverRoot: Element | null;
  readonly cleanup: () => void;
}> {
  const serverHtml = await renderToString(createSSRApp(component));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(component);
  app.mount(host);
  await nextTick();
  return {
    host,
    diagnostics,
    serverRoot,
    cleanup: () => {
      app.unmount();
      host.remove();
      console.warn = originalWarn;
      console.error = originalError;
    },
  };
}

test("renders byte-identical calendar markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="calendar"/);
  assert.match(left, /role="group"/);
  assert.match(left, /role="grid"/);
  assert.match(left, /aria-label="September 2026"/);
  assert.match(left, /aria-label="October 2026"/);
  assert.match(left, /aria-current="date"/);
  assert.match(left, /data-date="2026-09-10"[^>]*data-state="selected"/);
  assert.match(left, /<option[^>]*value="9"[^>]*selected/);
  assert.match(left, /<option[^>]*value="2026"[^>]*selected/);
  assert.match(left, /type="hidden" name="trip" value="2026-09-10"/);
  assert.doesNotMatch(left, /data-pending|function|NaN/);
});

test("hydrates a calendar with an explicit today without mismatches", async () => {
  const { host, diagnostics, serverRoot, cleanup } = await hydrate(SsrProbe);
  try {
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(diagnostics, []);
    const option = host.querySelector<HTMLOptionElement>(
      "[data-vize-ui='calendar-month-option'][value='9']",
    );
    assert.equal(option?.hasAttribute("selected"), true);
  } finally {
    cleanup();
  }
});

test("without today or now SSR renders a pending shell and the client fills it after mount", async () => {
  const PendingProbe = defineComponent({
    name: "CalendarPendingProbe",
    setup: () => () => h(CalendarRoot, { id: "pending", locale: "en-US", timeZone: "UTC" }),
  });
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(PendingProbe)),
    renderToString(createSSRApp(PendingProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-state="pending"/);
  assert.match(left, /data-pending="true"/);
  assert.doesNotMatch(left, /data-vize-ui="calendar-day"/);

  const { host, diagnostics, cleanup } = await hydrate(PendingProbe);
  try {
    assert.deepEqual(diagnostics, []);
    await nextTick();
    const root = host.querySelector("[data-vize-ui='calendar']");
    assert.notEqual(root?.getAttribute("data-state"), "pending");
    assert.equal(host.querySelectorAll("[data-today='true']").length, 1);
  } finally {
    cleanup();
  }
});

test("an injected now renders the same today on server and client", async () => {
  const instant = Date.UTC(2026, 11, 31, 20, 0);
  const NowProbe = defineComponent({
    name: "CalendarNowProbe",
    setup: () => () =>
      h(CalendarRoot, { locale: "en-US", now: () => instant, timeZone: "Asia/Tokyo" }),
  });
  const html = await renderToString(createSSRApp(NowProbe));
  assert.match(html, /aria-label="January 2027"/);
  assert.match(html, /data-date="2027-01-01"[^>]*data-today="true"/);
  const { diagnostics, cleanup } = await hydrate(NowProbe);
  try {
    assert.deepEqual(diagnostics, []);
  } finally {
    cleanup();
  }
});
