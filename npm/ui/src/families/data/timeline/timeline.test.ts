import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, shallowRef } from "vue";

import type { TimelineItemExpose, TimelineRootExpose } from "./timeline.ts";
import TimelineConnector from "./timeline-connector.vue";
import TimelineContent from "./timeline-content.vue";
import TimelineIndicator from "./timeline-indicator.vue";
import TimelineItem from "./timeline-item.vue";
import TimelineRoot from "./timeline-root.vue";
import TimelineTime from "./timeline-time.vue";
import { mountInteraction } from "../../../testing/mount.ts";

const events = [
  { value: "ordered", label: "Ordered", at: "2026-09-01" },
  { value: "shipped", label: "Shipped", at: "2026-09-03" },
  { value: "delivered", label: "Delivered", at: "2026-09-05" },
] as const;

function renderItems() {
  return events.map((event) =>
    h(TimelineItem, { key: event.value, value: event.value }, () => [
      h(TimelineIndicator, null, () => "●"),
      h(TimelineConnector),
      h(TimelineContent, null, ({ status, index }: { status: string | null; index: number }) => [
        h("strong", `${index}:${event.label}:${status ?? "none"}`),
        h(TimelineTime, { datetime: event.at }, () => event.at),
      ]),
    ]),
  );
}

function mountTimeline(props: Record<string, unknown> = {}) {
  return mountInteraction(TimelineRoot, {
    props: { ariaLabel: "Order history", ...props },
    slots: { default: renderItems },
  });
}

function items(root: HTMLElement): HTMLLIElement[] {
  return [...root.querySelectorAll<HTMLLIElement>('[data-vize-ui="timeline-item"]')];
}

test("renders a labelled ordered list with decorative parts and native time", () => {
  const handle = mountTimeline();
  const root = handle.root();
  assert.equal(root.tagName, "OL");
  assert.equal(root.getAttribute("aria-label"), "Order history");
  assert.equal(root.getAttribute("data-orientation"), "vertical");
  const rows = items(root);
  assert.equal(rows.length, 3);
  assert.equal(rows[0]?.tagName, "LI");
  assert.equal(rows[0]?.getAttribute("data-index"), "0");
  assert.equal(rows[2]?.getAttribute("data-last"), "true");
  assert.equal(rows[0]?.getAttribute("aria-current"), null);
  assert.equal(rows[0]?.getAttribute("data-state"), null, "untracked timelines have no status");
  const indicator = rows[0]?.querySelector('[data-vize-ui="timeline-indicator"]');
  const connector = rows[2]?.querySelector('[data-vize-ui="timeline-connector"]');
  assert.equal(indicator?.getAttribute("aria-hidden"), "true");
  assert.equal(connector?.getAttribute("data-last"), "true");
  const time = rows[1]?.querySelector("time");
  assert.equal(time?.getAttribute("datetime"), "2026-09-03");
  assert.match(rows[1]?.textContent ?? "", /1:Shipped:none/);
  handle.unmount();
});

test("derives complete, current, and upcoming states from the root value", async () => {
  const handle = mountTimeline({ value: "shipped" });
  const rows = items(handle.root());
  assert.deepEqual(
    rows.map((row) => row.getAttribute("data-state")),
    ["complete", "current", "upcoming"],
  );
  assert.equal(rows[1]?.getAttribute("aria-current"), "step");
  assert.equal(
    rows[0]?.querySelector('[data-vize-ui="timeline-connector"]')?.getAttribute("data-state"),
    "complete",
  );
  await handle.wrapper.setProps({ value: "delivered" });
  assert.deepEqual(
    rows.map((row) => row.getAttribute("data-state")),
    ["complete", "complete", "current"],
  );
  await handle.wrapper.setProps({ value: "missing" });
  assert.equal(rows[0]?.getAttribute("data-state"), null, "unknown values clear progress");
  handle.unmount();
});

test("explicit item status overrides derived progress", () => {
  const handle = mountInteraction(TimelineRoot, {
    props: { value: "a" },
    slots: {
      default: () => [
        h(TimelineItem, { value: "a" }, () => "A"),
        h(TimelineItem, { status: "complete" }, () => "B"),
      ],
    },
  });
  const rows = items(handle.root());
  assert.equal(rows[0]?.getAttribute("data-state"), "current");
  assert.equal(rows[1]?.getAttribute("data-state"), "complete");
  handle.unmount();
});

test("reversed and horizontal timelines expose native and data semantics", () => {
  const handle = mountTimeline({ orientation: "horizontal", reversed: true });
  const root = handle.root() as HTMLOListElement;
  assert.equal(root.reversed, true);
  assert.equal(root.getAttribute("data-reversed"), "true");
  assert.equal(root.getAttribute("data-orientation"), "horizontal");
  assert.equal(items(root)[0]?.getAttribute("data-orientation"), "horizontal");
  handle.unmount();
});

test("items added later join document order and Date values serialize to ISO", async () => {
  const extra = shallowRef(false);
  let rootExpose: TimelineRootExpose | null = null;
  let itemExpose: TimelineItemExpose | null = null;
  const Probe = defineComponent({
    name: "TimelineDynamicProbe",
    setup: () => () =>
      h(
        TimelineRoot,
        {
          value: "b",
          ref: (value: unknown) => {
            rootExpose = value as TimelineRootExpose | null;
          },
        },
        () => [
          h(TimelineItem, { value: "a" }, () => "A"),
          extra.value
            ? h(
                TimelineItem,
                {
                  value: "b",
                  ref: (value: unknown) => {
                    itemExpose = value as TimelineItemExpose | null;
                  },
                },
                () => h(TimelineTime, { datetime: new Date(Date.UTC(2026, 0, 2)) }, () => "Jan 2"),
              )
            : null,
          h(TimelineItem, { value: "c" }, () => "C"),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  if (rootExpose === null) assert.fail("timeline root must expose state");
  const root: TimelineRootExpose = rootExpose;
  assert.deepEqual(root.values, ["a", "c"]);
  extra.value = true;
  await nextTick();
  await nextTick();
  assert.deepEqual(root.values, ["a", "b", "c"]);
  if (itemExpose === null) assert.fail("timeline item must expose state");
  const item: TimelineItemExpose = itemExpose;
  assert.equal(item.index, 1);
  assert.equal(item.status, "current");
  assert.equal(item.last, false);
  assert.ok(item.element instanceof HTMLLIElement);
  assert.equal(
    handle.root().querySelector("time")?.getAttribute("datetime"),
    "2026-01-02T00:00:00.000Z",
  );
  handle.unmount();
});

test("compound parts require matching providers", () => {
  assert.throws(() => mountInteraction(TimelineItem), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(TimelineIndicator), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(TimelineConnector), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(TimelineContent), /VIZE_UI_CONTEXT_MISSING/);
});
