import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import PullToRefresh from "./pull-to-refresh.vue";
import PullToRefreshTrigger from "./pull-to-refresh-trigger.vue";
import type {
  PullToRefreshExpose,
  PullToRefreshSlotState,
  PullToRefreshSource,
} from "./pull-to-refresh-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function deferred() {
  let resolve: () => void = () => undefined;
  let reject: (error: unknown) => void = () => undefined;
  const promise = new Promise<void>((done, fail) => {
    resolve = done;
    reject = fail;
  });
  return { promise, resolve, reject };
}

function mountPull(props: Record<string, unknown> = {}) {
  return mountInteraction(PullToRefresh, {
    props,
    record: ["refresh", "settle"],
    slots: {
      default: (state: PullToRefreshSlotState) => [
        h("output", `${state.state}:${state.distance}:${state.progress}`),
        h(PullToRefreshTrigger, null, { default: () => "Refresh" }),
      ],
    },
  });
}

function touch(target: HTMLElement, type: string, y: number): TouchEvent {
  const touches =
    type === "touchend" || type === "touchcancel"
      ? []
      : [{ clientY: y, clientX: 0, identifier: 1 }];
  const event = new Event(type, { bubbles: true, cancelable: true });
  Object.defineProperty(event, "touches", { value: touches });
  target.dispatchEvent(event);
  return event as TouchEvent;
}

async function settle(): Promise<void> {
  for (let index = 0; index < 4; index += 1) {
    await Promise.resolve();
    await nextTick();
  }
}

const output = (root: HTMLElement) => root.querySelector("output")?.textContent;

test("pulling from the top applies resistance, arms past the threshold, and refreshes on release", async () => {
  const pending = deferred();
  const sources: PullToRefreshSource[] = [];
  const handle = mountPull({
    refreshAction: (source: PullToRefreshSource) => {
      sources.push(source);
      return pending.promise;
    },
  });
  const root = handle.root();

  touch(root, "touchstart", 100);
  const move = touch(root, "touchmove", 180);
  await nextTick();
  assert.equal(move.defaultPrevented, true, "pulling suppresses native overscroll");
  assert.equal(output(root), "pulling:40:0.625");
  assert.equal(root.style.getPropertyValue("--vize-pull-distance"), "40px");
  touch(root, "touchmove", 400);
  await nextTick();
  assert.equal(output(root), "armed:128:1", "distance is capped at maxDistance");
  assert.equal(root.getAttribute("data-state"), "armed");
  touch(root, "touchend", 400);
  await settle();
  assert.equal(output(root), "refreshing:64:1", "the indicator holds at the threshold");
  assert.equal(root.getAttribute("aria-busy"), "true");
  assert.deepEqual(sources, ["gesture"]);
  pending.resolve();
  await settle();
  assert.equal(output(root), "idle:0:0");
  assert.equal(root.getAttribute("aria-busy"), null);
  assert.deepEqual(
    handle.recorded().map((entry) => [entry.event, entry.payload[0]]),
    [
      ["refresh", "gesture"],
      ["settle", "gesture"],
    ],
  );
  handle.unmount();
});

test("short pulls, upward drags, scrolled content, and cancels never refresh", async () => {
  let calls = 0;
  const handle = mountPull({ refreshAction: () => (calls += 1), threshold: 50 });
  const root = handle.root();

  touch(root, "touchstart", 100);
  touch(root, "touchmove", 150);
  touch(root, "touchend", 150);
  await settle();
  assert.equal(output(root), "idle:0:0");

  touch(root, "touchstart", 100);
  const up = touch(root, "touchmove", 60);
  assert.equal(up.defaultPrevented, false, "upward drags keep native scrolling");
  touch(root, "touchend", 60);

  touch(root, "touchstart", 100);
  touch(root, "touchmove", 400);
  touch(root, "touchcancel", 400);
  await settle();

  root.scrollTop = 20;
  touch(root, "touchstart", 100);
  const scrolled = touch(root, "touchmove", 400);
  assert.equal(scrolled.defaultPrevented, false, "pulls only start at the top");
  touch(root, "touchend", 400);
  await settle();
  assert.equal(calls, 0);
  handle.unmount();
});

test("mouse drags are opt-in, and the trigger is the keyboard alternative", async () => {
  let calls = 0;
  const pointer = (target: HTMLElement, type: string, y: number) =>
    target.dispatchEvent(
      new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        button: 0,
        clientY: y,
        pointerId: 1,
        pointerType: "mouse",
      }),
    );
  const ignored = mountPull({ refreshAction: () => (calls += 1) });
  pointer(ignored.root(), "pointerdown", 0);
  pointer(ignored.root(), "pointermove", 300);
  pointer(ignored.root(), "pointerup", 300);
  await settle();
  assert.equal(calls, 0);
  ignored.unmount();

  const handle = mountPull({ refreshAction: () => (calls += 1), allowMouse: true });
  const root = handle.root();
  pointer(root, "pointerdown", 0);
  pointer(root, "pointermove", 300);
  pointer(root, "pointerup", 300);
  await settle();
  assert.equal(calls, 1);

  const trigger = root.querySelector("button");
  assert.ok(trigger instanceof HTMLButtonElement);
  assert.equal(trigger.getAttribute("aria-controls"), root.id);
  trigger.click();
  await settle();
  assert.equal(calls, 2);
  assert.deepEqual(
    handle
      .recorded()
      .filter((entry) => entry.event === "refresh")
      .map((entry) => entry.payload[0]),
    ["gesture", "trigger"],
  );
  handle.unmount();
});

test("refreshes run one at a time, report failures, and respect disabled", async () => {
  const pending = deferred();
  const handle = mountPull({ refreshAction: () => pending.promise });
  const api = handle.exposes<PullToRefreshExpose>();
  const first = api.refresh();
  const second = api.refresh();
  await settle();
  assert.equal(api.refreshing, true);
  const trigger = handle.root().querySelector("button");
  assert.equal(trigger?.disabled, true, "the trigger is disabled while refreshing");
  pending.reject(new Error("offline"));
  await Promise.all([first, second]);
  await settle();
  const settled = handle.recorded().filter((entry) => entry.event === "settle");
  assert.equal(settled.length, 1);
  assert.equal(settled[0]?.payload[0], "api");
  assert.ok(settled[0]?.payload[1] instanceof Error);
  assert.equal(api.state, "idle");
  handle.unmount();

  const disabled = mountPull({ disabled: true, refreshAction: () => assert.fail("never runs") });
  touch(disabled.root(), "touchstart", 0);
  touch(disabled.root(), "touchmove", 400);
  touch(disabled.root(), "touchend", 400);
  await disabled.exposes<PullToRefreshExpose>().refresh();
  assert.equal(disabled.root().getAttribute("data-disabled"), "true");
  disabled.unmount();
});

test("publishes the reduced-motion preference after mount", async () => {
  const original = window.matchMedia;
  window.matchMedia = ((query: string) => ({
    matches: query.includes("reduce"),
    media: query,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
  })) as unknown as typeof window.matchMedia;
  try {
    const handle = mountPull();
    await nextTick();
    assert.equal(handle.root().getAttribute("data-reduced-motion"), "true");
    assert.equal(handle.exposes<PullToRefreshExpose>().reducedMotion, true);
    handle.unmount();
  } finally {
    window.matchMedia = original;
  }
});

test("the trigger requires a PullToRefresh provider", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(
      () => mountInteraction(PullToRefreshTrigger),
      /VIZE_UI_CONTEXT_MISSING: PullToRefresh/,
    );
  } finally {
    console.warn = warn;
  }
});
