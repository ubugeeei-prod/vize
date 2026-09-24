import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { MarqueeContentExpose, MarqueeRootExpose, MarqueeSlotState } from "./marquee.ts";
import MarqueeContent from "./marquee-content.vue";
import {
  marqueeCopies,
  marqueeDuration,
  marqueeOrientation,
  marqueeStyle,
} from "./marquee-geometry.ts";
import MarqueePauseButton from "./marquee-pause-button.vue";
import MarqueeRoot from "./marquee-root.vue";
import { resolveMarqueeMessages } from "./marquee-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

type ResizeCallback = (entries: readonly unknown[]) => void;

class FakeResizeObserver {
  static instances: FakeResizeObserver[] = [];
  readonly observed: Element[] = [];
  constructor(readonly callback: ResizeCallback) {
    FakeResizeObserver.instances.push(this);
  }
  observe(target: Element): void {
    this.observed.push(target);
  }
  unobserve(): void {}
  disconnect(): void {}
  fire(): void {
    this.callback(
      this.observed.map((target) => ({
        target,
        contentRect: { width: 0, height: 0 },
      })),
    );
  }
}

interface Layout {
  copy: number;
  gap: number;
  viewport: number;
}

async function withLayout(layout: Layout, run: () => Promise<void>): Promise<void> {
  const previousObserver = globalThis.ResizeObserver;
  const descriptor = Object.getOwnPropertyDescriptor(Element.prototype, "getBoundingClientRect");
  FakeResizeObserver.instances = [];
  globalThis.ResizeObserver = FakeResizeObserver as unknown as typeof ResizeObserver;
  Object.defineProperty(Element.prototype, "getBoundingClientRect", {
    configurable: true,
    value(this: Element): DOMRect {
      const part = this.getAttribute("data-vize-ui");
      const vertical =
        this.closest('[data-vize-ui="marquee-root"]')?.getAttribute("data-orientation") ===
        "vertical";
      let start = 0;
      let length = 0;
      if (part === "marquee-root") length = layout.viewport;
      if (part === "marquee-content") {
        start = Number(this.getAttribute("data-copy")) * (layout.copy + layout.gap);
        length = layout.copy;
      }
      const rect = vertical
        ? { x: 0, y: start, left: 0, top: start, width: 10, height: length }
        : { x: start, y: 0, left: start, top: 0, width: length, height: 10 };
      return {
        ...rect,
        right: rect.left + rect.width,
        bottom: rect.top + rect.height,
        toJSON: () => ({}),
      };
    },
  });
  try {
    await run();
  } finally {
    globalThis.ResizeObserver = previousObserver;
    if (descriptor) Object.defineProperty(Element.prototype, "getBoundingClientRect", descriptor);
  }
}

async function withReducedMotion(matches: boolean, run: () => Promise<void>): Promise<void> {
  const previous = globalThis.matchMedia;
  globalThis.matchMedia = ((query: string) => ({
    matches: matches && query.includes("reduce"),
    media: query,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
  })) as unknown as typeof globalThis.matchMedia;
  try {
    await run();
  } finally {
    globalThis.matchMedia = previous;
  }
}

function mountMarquee(props: Record<string, unknown> = {}) {
  return mountInteraction(MarqueeRoot, {
    props: { id: "news", ariaLabel: "Headlines", ...props },
    record: ["update:playing", "stateChange"],
    slots: {
      default: (state: MarqueeSlotState) => [
        h(MarqueePauseButton, null, ({ label }: { label: string }) =>
          h("span", { "data-icon": state.state }, label.length > 0 ? "" : "?"),
        ),
        h(MarqueeContent, null, ({ copy }: { copy: number }) =>
          h("a", { href: "#story", "data-copy-link": String(copy) }, "Breaking story"),
        ),
      ],
    },
  });
}

function copiesOf(root: HTMLElement): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>('[data-vize-ui="marquee-content"]')];
}

test("renders an accessible marquee region with one exposed copy and inert duplicates", () => {
  const handle = mountMarquee();
  const root = handle.root();
  assert.equal(root.id, "news");
  assert.equal(root.getAttribute("role"), "marquee");
  assert.equal(root.getAttribute("aria-label"), "Headlines");
  assert.equal(root.getAttribute("data-vize-ui"), "marquee-root");
  assert.equal(root.getAttribute("data-direction"), "left");
  assert.equal(root.getAttribute("data-orientation"), "horizontal");
  assert.equal(root.getAttribute("data-measured"), "false");
  const copies = copiesOf(root);
  assert.equal(copies.length, 2);
  assert.equal(copies[0]?.hasAttribute("aria-hidden"), false);
  assert.equal(copies[0]?.hasAttribute("inert"), false);
  assert.equal(copies[1]?.getAttribute("aria-hidden"), "true");
  assert.equal(copies[1]?.hasAttribute("inert"), true);
  assert.equal(copies[1]?.querySelector("[data-copy-link]")?.getAttribute("data-copy-link"), "1");
  const button = handle.getByRole("button", { name: "Pause scrolling content" });
  assert.equal(button.getAttribute("aria-controls"), "news");
  assert.equal(button.getAttribute("data-state"), "playing");
  handle.unmount();
});

test("measures copies to publish distance, duration, and a viewport-filling repeat count", async () => {
  await withLayout({ copy: 120, gap: 30, viewport: 400 }, async () => {
    const handle = mountMarquee({ speed: 75 });
    const root = handle.root();
    await nextTick();
    assert.equal(root.getAttribute("data-measured"), "true");
    assert.equal(root.style.getPropertyValue("--vize-ui-marquee-distance"), "150px");
    assert.equal(root.style.getPropertyValue("--vize-ui-marquee-duration"), "2s");
    assert.equal(copiesOf(root).length, 4, "ceil(400 / 150) + 1 copies");
    assert.equal(root.style.getPropertyValue("--vize-ui-marquee-copies"), "4");
    const observer = FakeResizeObserver.instances[0];
    assert.ok(observer);
    assert.equal(observer.observed.length, 2);

    await handle.wrapper.setProps({ repeat: 3, speed: 150 });
    assert.equal(copiesOf(root).length, 3);
    assert.equal(root.style.getPropertyValue("--vize-ui-marquee-duration"), "1s");
    handle.unmount();
  });
});

test("vertical directions measure along the block axis and resize re-measures", async () => {
  const layout = { copy: 40, gap: 0, viewport: 100 };
  await withLayout(layout, async () => {
    const handle = mountMarquee({ direction: "up", speed: 20 });
    const root = handle.root();
    await nextTick();
    assert.equal(root.getAttribute("data-orientation"), "vertical");
    assert.equal(root.style.getPropertyValue("--vize-ui-marquee-distance"), "40px");
    assert.equal(copiesOf(root).length, 4);
    layout.copy = 200;
    FakeResizeObserver.instances[0]?.fire();
    await nextTick();
    assert.equal(root.style.getPropertyValue("--vize-ui-marquee-distance"), "200px");
    assert.equal(copiesOf(root).length, 2);
    const content = handle.wrapper.findComponent(MarqueeContent);
    const exposed = content.vm as unknown as MarqueeContentExpose;
    assert.ok(exposed.element?.getAttribute("data-vize-ui") === "marquee-track");
    handle.unmount();
  });
});

test("hover and keyboard focus pause and resume the animation", async () => {
  const handle = mountMarquee();
  const root = handle.root();
  root.dispatchEvent(new PointerEvent("pointerenter", { pointerType: "mouse" }));
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "paused");
  assert.equal(root.getAttribute("data-pause-reason"), "hover");
  root.dispatchEvent(new PointerEvent("pointerleave", { pointerType: "mouse" }));
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "running");

  root.dispatchEvent(new PointerEvent("pointerenter", { pointerType: "touch" }));
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "running", "touch never hovers");

  root.querySelector<HTMLAnchorElement>("a")?.focus();
  await nextTick();
  assert.equal(root.getAttribute("data-pause-reason"), "focus");
  root.querySelector<HTMLAnchorElement>("a")?.blur();
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "running");
  assert.deepEqual(handle.wrapper.emitted("stateChange"), [
    ["paused", "hover"],
    ["running", null],
    ["paused", "focus"],
    ["running", null],
  ]);
  handle.unmount();

  const fixed = mountMarquee({ pauseOnHover: false, pauseOnFocus: false });
  fixed.root().dispatchEvent(new PointerEvent("pointerenter", { pointerType: "mouse" }));
  fixed.root().querySelector<HTMLAnchorElement>("a")?.focus();
  await nextTick();
  assert.equal(fixed.root().getAttribute("data-state"), "running");
  fixed.unmount();
});

test("the pause button toggles the play intent per WCAG 2.2.2", async () => {
  const handle = mountMarquee();
  const root = handle.root();
  const button = handle.getByRole("button", { name: "Pause scrolling content" });
  button.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  await nextTick();
  assert.equal(root.getAttribute("data-pause-reason"), "user");
  assert.equal(button.getAttribute("aria-label"), "Play scrolling content");
  assert.equal(button.getAttribute("data-state"), "paused");
  await handle.click(button);
  assert.equal(root.getAttribute("data-state"), "running");
  assert.deepEqual(handle.wrapper.emitted("update:playing"), [[false], [true]]);

  const blocked = (event: MouseEvent) => event.preventDefault();
  const guarded = mountInteraction(MarqueeRoot, {
    slots: { default: () => h(MarqueePauseButton, { onClick: blocked }) },
  });
  guarded
    .root()
    .querySelector("button")
    ?.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  await nextTick();
  assert.equal(guarded.root().getAttribute("data-state"), "running");
  guarded.unmount();
  handle.unmount();
});

test("reduced motion keeps the marquee paused until the user opts in", async () => {
  await withReducedMotion(true, async () => {
    const handle = mountMarquee();
    const root = handle.root();
    await nextTick();
    assert.equal(root.getAttribute("data-pause-reason"), "reduced-motion");
    const button = handle.getByRole("button", { name: "Play scrolling content" });
    await handle.click(button);
    assert.equal(root.getAttribute("data-state"), "running", "pressing play is explicit consent");
    assert.equal(handle.wrapper.emitted("update:playing"), undefined, "intent was already on");
    await handle.click(button);
    assert.equal(root.getAttribute("data-pause-reason"), "user");
    handle.unmount();

    const ignored = mountMarquee({ respectReducedMotion: false });
    await nextTick();
    assert.equal(ignored.root().getAttribute("data-state"), "running");
    ignored.unmount();
  });
});

test("controlled playing, localized messages, and the exposed instance", async () => {
  const handle = mountMarquee({
    playing: false,
    messages: { play: "再生", pause: "一時停止" },
    direction: "right",
  });
  const root = handle.root();
  const exposed = handle.exposes<MarqueeRootExpose>();
  assert.equal(root.getAttribute("data-direction"), "right");
  assert.equal(handle.getByRole("button").getAttribute("aria-label"), "再生");
  assert.equal(exposed.play(), true);
  assert.deepEqual(handle.wrapper.emitted("update:playing"), [[true]]);
  assert.equal(exposed.state, "paused", "the parent owns the intent");
  await handle.wrapper.setProps({ playing: true });
  assert.equal(exposed.state, "running");
  assert.equal(exposed.playing, true);
  assert.equal(exposed.copies, 2);
  assert.equal(exposed.duration, null);
  assert.equal(exposed.direction, "right");
  assert.equal(exposed.pauseReason, null);
  assert.ok(exposed.element === root);
  assert.equal(handle.getByRole("button").getAttribute("aria-label"), "一時停止");
  assert.equal(exposed.toggle(), true);
  assert.equal(exposed.pause(), true);
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  for (const part of [MarqueeContent, MarqueePauseButton]) {
    assert.throws(
      () => mountInteraction(part),
      /VIZE_UI_CONTEXT_MISSING: Marquee requires a matching provider/,
    );
  }
});

test("derives orientation, copy counts, durations, and custom properties", () => {
  assert.equal(marqueeOrientation("left"), "horizontal");
  assert.equal(marqueeOrientation("down"), "vertical");
  assert.equal(marqueeCopies("auto", null, 300), 2);
  assert.equal(marqueeCopies("auto", 100, 300), 4);
  assert.equal(marqueeCopies("auto", 400, 300), 2);
  assert.equal(marqueeCopies("auto", 0.001, 300), 32, "zero-size content is capped");
  assert.equal(marqueeCopies(1, 100, 300), 2, "fixed repeats keep the seamless minimum");
  assert.equal(marqueeCopies(5.7, null, null), 5);
  assert.equal(marqueeCopies(Number.NaN, null, null), 2);
  assert.equal(marqueeCopies(99, null, null), 32);
  assert.equal(marqueeDuration(100, 50), 2);
  assert.equal(marqueeDuration(100, 3), 33.333);
  assert.equal(marqueeDuration(null, 50), null);
  assert.equal(marqueeDuration(100, 0), null);
  assert.equal(marqueeDuration(100, Number.POSITIVE_INFINITY), null);
  assert.equal(marqueeStyle(null, null, 2), "--vize-ui-marquee-copies: 2");
  assert.equal(
    marqueeStyle(120.456, 2.4, 3),
    "--vize-ui-marquee-copies: 3; --vize-ui-marquee-distance: 120.46px; --vize-ui-marquee-duration: 2.4s",
  );
  assert.equal(resolveMarqueeMessages(undefined).pause, "Pause scrolling content");
  assert.equal(resolveMarqueeMessages({ play: "Go" }).play, "Go");
});
