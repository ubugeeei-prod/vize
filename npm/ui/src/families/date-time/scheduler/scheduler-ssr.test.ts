import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import type { Component } from "vue";
import { renderToString } from "vue/server-renderer";

import { createPlainDate } from "../calendar/plain-date.ts";
import { createPlainDateTime } from "../datetime-field/plain-date-time.ts";
import type { SchedulerEvent } from "./scheduler-layout.ts";
import SchedulerRoot from "./scheduler-root.vue";

const events: readonly SchedulerEvent<{ readonly room: string }>[] = [
  {
    id: "review",
    title: "Review",
    data: { room: "A" },
    start: createPlainDateTime(2026, 9, 25, 9, 0),
    end: createPlainDateTime(2026, 9, 25, 10, 0),
  },
  {
    id: "standup",
    title: "Standup",
    data: { room: "B" },
    start: createPlainDateTime(2026, 9, 25, 9, 0),
    end: createPlainDateTime(2026, 9, 25, 9, 30),
  },
];

function probe(view: "week" | "month" | "day", withToday = true): Component {
  return defineComponent({
    name: `SchedulerSsrProbe${view}`,
    setup: () => () =>
      h(SchedulerRoot, {
        id: `ssr-${view}`,
        events,
        view,
        locale: "en-US",
        dayStartHour: 8,
        dayEndHour: 12,
        ...(withToday ? { today: createPlainDate(2026, 9, 25) } : { timeZone: "UTC" }),
      }),
  });
}

async function hydrate(component: Component): Promise<readonly string[]> {
  const html = await renderToString(createSSRApp(component));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(component);
  try {
    app.mount(host);
    await nextTick();
    assert.ok(host.firstElementChild === serverRoot);
    return diagnostics;
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
}

test("renders byte-identical week, day, and month markup with deterministic layout", async () => {
  for (const view of ["week", "day", "month"] as const) {
    const [left, right] = await Promise.all([
      renderToString(createSSRApp(probe(view))),
      renderToString(createSSRApp(probe(view))),
    ]);
    assert.equal(left, right);
    assert.match(left, /data-vize-ui="scheduler"/);
    assert.match(left, /role="grid"/);
    assert.match(left, /data-event-id="review"/);
    assert.doesNotMatch(left, /function|NaN/);
  }
  const week = await renderToString(createSSRApp(probe("week")));
  assert.match(week, /--vize-scheduler-columns:2/);
  assert.match(week, /aria-current="date"/);
});

test("hydrates every view without mismatches", async () => {
  for (const view of ["week", "day", "month"] as const) {
    assert.deepEqual(await hydrate(probe(view)), []);
  }
});

test("without today or now the server renders a pending shell and hydrates silently", async () => {
  const html = await renderToString(createSSRApp(probe("week", false)));
  assert.match(html, /data-state="pending"/);
  assert.doesNotMatch(html, /data-vize-ui="scheduler-slot"/);
  assert.deepEqual(await hydrate(probe("week", false)), []);
});
