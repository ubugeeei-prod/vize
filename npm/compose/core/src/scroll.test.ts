import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { useScroll, useWindowScroll } from "./scroll.ts";
import { asElement, FakeDocument, FakeWindow } from "./testing/fake-dom.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

function manualScheduler(): TimeoutScheduler & { readonly flush: () => void; pending: number } {
  let queued: (() => void) | undefined;
  const scheduler = {
    pending: 0,
    setTimeout: (callback: () => void) => {
      queued = callback;
      scheduler.pending += 1;
      return scheduler.pending;
    },
    clearTimeout: () => {
      queued = undefined;
    },
    flush: () => {
      const callback = queued;
      queued = undefined;
      callback?.();
    },
  };
  return scheduler;
}

function scrollable() {
  const document = new FakeDocument();
  const element = document.createElement();
  element.scrollWidth = 1000;
  element.scrollHeight = 2000;
  element.clientWidth = 100;
  element.clientHeight = 200;
  return element;
}

void test("server and unresolved targets report an unscrolled page", () => {
  const scroll = useScroll(null);
  assert.equal(scroll.x.value, 0);
  assert.deepEqual(
    { ...scroll.arrivedState },
    { left: true, right: false, top: true, bottom: false },
  );
  assert.equal(scroll.isScrolling.value, false);
  scroll.scrollTo({ top: 10 });
});

void test("tracks offsets, edges, directions, and idle settling for an element", () => {
  const element = scrollable();
  const scheduler = manualScheduler();
  const scope = effectScope();
  const stops: string[] = [];
  const scroll = scope.run(() =>
    useScroll(asElement(element), {
      scheduler,
      offset: { bottom: 10 },
      onStop: (event) => stops.push(event.type),
    }),
  );
  assert.ok(scroll);

  element.scrollTop = 1795;
  element.dispatchEvent(new Event("scroll"));
  assert.equal(scroll.y.value, 1795);
  assert.equal(scroll.isScrolling.value, true);
  assert.equal(scroll.directions.bottom, true);
  assert.equal(scroll.arrivedState.bottom, true);
  assert.equal(scroll.arrivedState.top, false);

  element.scrollTop = 1000;
  element.dispatchEvent(new Event("scroll"));
  assert.equal(scroll.directions.top, true);
  assert.equal(scroll.directions.bottom, false);
  assert.equal(scroll.arrivedState.bottom, false);

  scheduler.flush();
  assert.equal(scroll.isScrolling.value, false);
  assert.equal(scroll.directions.top, false);
  assert.deepEqual(stops, ["scroll"]);

  scope.stop();
  element.scrollTop = 0;
  element.dispatchEvent(new Event("scroll"));
  assert.equal(scroll.y.value, 1000);
});

void test("scrollend settles immediately and writable refs scroll smoothly", () => {
  const element = scrollable();
  const scope = effectScope();
  const scroll = scope.run(() =>
    useScroll(asElement(element), { scheduler: manualScheduler(), behavior: "smooth" }),
  );
  assert.ok(scroll);
  scroll.y.value = 50;
  assert.deepEqual(element.scrollCalls.at(-1), { left: 0, top: 50, behavior: "smooth" });
  assert.equal(scroll.isScrolling.value, true);
  element.dispatchEvent(new Event("scrollend"));
  assert.equal(scroll.isScrolling.value, false);
  scroll.x.value = 20;
  assert.deepEqual(element.scrollCalls.at(-1), { left: 20, top: 50, behavior: "smooth" });
  scope.stop();
});

void test("window scroll reads document metrics", () => {
  const host = new FakeWindow();
  host.document.documentElement.scrollHeight = 1000;
  host.document.documentElement.clientHeight = 500;
  const scope = effectScope();
  const scroll = scope.run(() => useWindowScroll({ host, scheduler: manualScheduler() }));
  host.scrollTo({ top: 500 });
  assert.equal(scroll?.y.value, 500);
  assert.equal(scroll?.arrivedState.bottom, true);
  scope.stop();
});
