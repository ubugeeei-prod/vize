import assert from "node:assert/strict";

import { afterEach, beforeEach, test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { StickyExpose } from "./sticky.ts";
import { isStickyStuck, stickyRootMargin } from "./sticky-geometry.ts";
import Sticky from "./sticky.vue";
import { mountInteraction } from "../../../testing/mount.ts";

type ObserverCallback = (entries: readonly IntersectionObserverEntry[]) => void;

class FakeIntersectionObserver {
  static instances: FakeIntersectionObserver[] = [];
  readonly observed: Element[] = [];
  disconnected = false;
  constructor(
    readonly callback: ObserverCallback,
    readonly init: IntersectionObserverInit | undefined,
  ) {
    FakeIntersectionObserver.instances.push(this);
  }
  observe(target: Element): void {
    this.observed.push(target);
  }
  unobserve(): void {}
  disconnect(): void {
    this.disconnected = true;
  }
  fire(target: Element, ratio: number): void {
    this.callback([
      {
        boundingClientRect: target.getBoundingClientRect(),
        intersectionRatio: ratio,
        intersectionRect: target.getBoundingClientRect(),
        isIntersecting: ratio > 0,
        rootBounds: null,
        target,
        time: 0,
      },
    ]);
  }
}

const originalObserver = globalThis.IntersectionObserver;

beforeEach(() => {
  FakeIntersectionObserver.instances = [];
  Object.defineProperty(globalThis, "IntersectionObserver", {
    configurable: true,
    value: FakeIntersectionObserver,
    writable: true,
  });
});

afterEach(() => {
  Object.defineProperty(globalThis, "IntersectionObserver", {
    configurable: true,
    value: originalObserver,
    writable: true,
  });
});

function place(target: Element, top: number, height = 40): void {
  Object.defineProperty(target, "getBoundingClientRect", {
    configurable: true,
    value: () => new DOMRect(0, top, 200, height),
  });
}

function latestObserver(): FakeIntersectionObserver {
  const observer = FakeIntersectionObserver.instances.at(-1);
  assert.ok(observer, "Sticky must observe its element after mount");
  return observer;
}

test("renders native sticky positioning with offset variables and slot state", async () => {
  const handle = mountInteraction(Sticky, {
    props: { offset: 12, as: "header" },
    slots: { default: ({ state }: { readonly state: string }) => h("span", state) },
  });
  await nextTick();
  const root = handle.root();

  assert.equal(root.tagName, "HEADER");
  assert.equal(root.getAttribute("data-vize-ui"), "sticky");
  assert.equal(root.getAttribute("data-side"), "top");
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.equal(root.style.position, "sticky");
  assert.equal(root.style.top, "12px");
  assert.equal(root.style.getPropertyValue("--vize-sticky-offset"), "12px");
  assert.equal(root.textContent, "idle");
  assert.equal(latestObserver().init?.rootMargin, "-13px 0px 0px 0px");

  handle.unmount();
});

test("reports stuck when the box rests on its offset line and releases after", async () => {
  const handle = mountInteraction(Sticky, { props: { offset: 10 } });
  await nextTick();
  const root = handle.root();
  const observer = latestObserver();

  place(root, 180);
  observer.fire(root, 1);
  await nextTick();
  assert.equal(root.getAttribute("data-stuck"), null);

  place(root, 10);
  observer.fire(root, 0.9);
  await nextTick();
  assert.equal(root.getAttribute("data-stuck"), "true");
  assert.equal(root.getAttribute("data-state"), "stuck");

  place(root, -60);
  observer.fire(root, 0);
  await nextTick();
  assert.equal(root.getAttribute("data-stuck"), null);
  assert.deepEqual(handle.wrapper.emitted("stuck-change"), [[true], [false]]);

  handle.unmount();
});

test("bottom stickiness measures against the container bottom edge", async () => {
  const container = document.createElement("div");
  document.body.append(container);
  Object.defineProperty(container, "getBoundingClientRect", {
    configurable: true,
    value: () => new DOMRect(0, 0, 400, 300),
  });
  const handle = mountInteraction(Sticky, { props: { side: "bottom", root: container } });
  await nextTick();
  const root = handle.root();

  assert.equal(root.style.bottom, "0px");
  assert.equal(latestObserver().init?.root, container);
  assert.equal(latestObserver().init?.rootMargin, "0px 0px -1px 0px");
  place(root, 260);
  latestObserver().fire(root, 0.95);
  await nextTick();
  assert.equal(root.getAttribute("data-stuck"), "true");

  handle.unmount();
  container.remove();
});

test("disabled stickiness drops positioning and observation", async () => {
  const handle = mountInteraction(Sticky, { props: { offset: 4 } });
  await nextTick();
  const first = latestObserver();

  await handle.wrapper.setProps({ disabled: true });
  await nextTick();
  const root = handle.root();
  assert.equal(first.disconnected, true);
  assert.equal(root.style.position, "");
  assert.equal(root.getAttribute("data-disabled"), "true");

  await handle.wrapper.setProps({ disabled: false, offset: 20 });
  await nextTick();
  assert.equal(latestObserver().init?.rootMargin, "-21px 0px 0px 0px");

  handle.unmount();
});

test("expose refresh re-reads geometry on demand", async () => {
  let sticky: StickyExpose | null = null;
  const Probe = defineComponent({
    name: "StickyExposeProbe",
    setup: () => () =>
      h(Sticky, {
        ref: (value) => {
          sticky = value as StickyExpose | null;
        },
      }),
  });
  const handle = mountInteraction(Probe);
  await nextTick();
  if (sticky === null) assert.fail("Sticky must expose its API");
  const exposed: StickyExpose = sticky;
  assert.ok(exposed.element);
  place(exposed.element, 0);

  assert.equal(exposed.refresh(), true);
  await nextTick();
  assert.equal(exposed.stuck, true);
  assert.equal(exposed.state, "stuck");
  assert.equal(exposed.side, "top");

  handle.unmount();
});

test("geometry helpers describe the pinned band on both edges", () => {
  const container = { bottom: 500, top: 0 };
  assert.equal(isStickyStuck("top", 0, { bottom: 40, top: 0 }, container), true);
  assert.equal(isStickyStuck("top", 0, { bottom: 140, top: 100 }, container), false);
  assert.equal(isStickyStuck("top", 8, { bottom: 48, top: 8.5 }, container), true);
  assert.equal(isStickyStuck("bottom", 0, { bottom: 500, top: 460 }, container), true);
  assert.equal(isStickyStuck("bottom", 0, { bottom: 300, top: 260 }, container), false);
  assert.equal(stickyRootMargin("top", 0), "-1px 0px 0px 0px");
  assert.equal(stickyRootMargin("bottom", 5), "0px 0px -6px 0px");
});
