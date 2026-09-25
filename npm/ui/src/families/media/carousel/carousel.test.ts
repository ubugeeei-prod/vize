import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { CarouselRootExpose, CarouselSlotState } from "./carousel.ts";
import CarouselAutoplayToggle from "./carousel-autoplay-toggle.vue";
import CarouselIndicator from "./carousel-indicator.vue";
import CarouselIndicatorGroup from "./carousel-indicator-group.vue";
import CarouselNext from "./carousel-next.vue";
import CarouselPrevious from "./carousel-previous.vue";
import CarouselRoot from "./carousel-root.vue";
import CarouselSlide from "./carousel-slide.vue";
import CarouselViewport from "./carousel-viewport.vue";
import { installCarouselGeometry } from "./carousel-test-utils.ts";
import type { CarouselGeometry } from "./carousel-test-utils.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitFor(condition: () => boolean, timeout = 1000): Promise<void> {
  const deadline = Date.now() + timeout;
  while (!condition()) {
    if (Date.now() > deadline) throw new Error("condition was not met in time");
    await wait(5);
  }
}

async function withGeometry(
  run: (geometry: CarouselGeometry) => Promise<void>,
  size = 100,
  slideSize = size,
): Promise<void> {
  const geometry = installCarouselGeometry(size, slideSize);
  try {
    await run(geometry);
  } finally {
    geometry.restore();
  }
}

function mountCarousel(props: Record<string, unknown> = {}, count = 3) {
  const slides = Array.from({ length: count }, (_, index) => index);
  return mountInteraction(CarouselRoot, {
    props: { id: "gallery", slideCount: count, ariaLabel: "Featured", ...props },
    record: ["update:modelValue", "change", "update:playing"],
    slots: {
      default: (state: CarouselSlotState) => [
        h("output", { "data-root-index": String(state.index) }, state.autoplay),
        h(CarouselAutoplayToggle, null, ({ autoplay }: CarouselSlotState) =>
          autoplay === "stopped" ? "Start rotation" : "Stop rotation",
        ),
        h(CarouselPrevious, null, () => "Previous"),
        h(CarouselNext, null, () => "Next"),
        h(CarouselViewport, { settleDelay: 5 }, () =>
          slides.map((index) =>
            h(CarouselSlide, { index, key: index }, ({ state }: { state: string }) =>
              h("a", { href: `#slide-${index}`, "data-slide-state": state }, `Slide ${index + 1}`),
            ),
          ),
        ),
        h(CarouselIndicatorGroup, { ariaLabel: "Choose slide" }, () =>
          slides.map((index) => h(CarouselIndicator, { index, key: index })),
        ),
      ],
    },
  });
}

function viewportOf(root: HTMLElement): HTMLDivElement {
  const viewport = root.querySelector<HTMLDivElement>('[data-vize-ui="carousel-viewport"]');
  assert.ok(viewport);
  return viewport;
}

function slidesOf(root: HTMLElement): HTMLDivElement[] {
  return [...root.querySelectorAll<HTMLDivElement>('[data-vize-ui="carousel-slide"]')];
}

function indicatorsOf(root: HTMLElement): HTMLButtonElement[] {
  return [...root.querySelectorAll<HTMLButtonElement>('[data-vize-ui="carousel-indicator"]')];
}

function pointer(type: string, init: PointerEventInit): PointerEvent {
  return new PointerEvent(type, { bubbles: true, cancelable: true, pointerId: 1, ...init });
}

test("renders the carousel pattern with deterministic ids, slide labels, and wired controls", () => {
  const handle = mountCarousel();
  const root = handle.root();
  const viewport = viewportOf(root);
  const slides = slidesOf(root);
  const indicators = indicatorsOf(root);

  assert.equal(root.tagName, "SECTION");
  assert.equal(root.id, "gallery");
  assert.equal(root.getAttribute("aria-roledescription"), "carousel");
  assert.equal(root.getAttribute("aria-label"), "Featured");
  assert.equal(root.getAttribute("data-vize-ui"), "carousel-root");
  assert.equal(root.getAttribute("data-orientation"), "horizontal");
  assert.equal(root.getAttribute("data-autoplay"), "stopped");
  assert.equal(root.getAttribute("data-index"), "0");
  assert.equal(viewport.id, "gallery-viewport");
  assert.equal(viewport.getAttribute("tabindex"), "0");
  assert.equal(viewport.getAttribute("aria-live"), "polite");
  assert.equal(slides.length, 3);
  assert.equal(slides[0]?.id, "gallery-slide-0");
  assert.equal(slides[0]?.getAttribute("role"), "group");
  assert.equal(slides[0]?.getAttribute("aria-roledescription"), "slide");
  assert.equal(slides[1]?.getAttribute("aria-label"), "2 of 3");
  assert.equal(slides[0]?.getAttribute("data-state"), "active");
  assert.equal(slides[1]?.getAttribute("data-state"), "inactive");
  assert.equal(slides[1]?.hasAttribute("inert"), false, "unmeasured slides stay interactive");
  const previous = handle.getByRole("button", { name: "Previous" }) as HTMLButtonElement;
  const next = handle.getByRole("button", { name: "Next" }) as HTMLButtonElement;
  assert.equal(previous.disabled, true);
  assert.equal(next.disabled, false);
  assert.equal(next.getAttribute("aria-controls"), "gallery-viewport");
  const tablist = handle.getByRole("tablist", { name: "Choose slide" });
  assert.equal(tablist.getAttribute("aria-orientation"), "horizontal");
  assert.equal(indicators[0]?.getAttribute("role"), "tab");
  assert.equal(indicators[0]?.getAttribute("aria-selected"), "true");
  assert.equal(indicators[0]?.getAttribute("tabindex"), "0");
  assert.equal(indicators[1]?.getAttribute("tabindex"), "-1");
  assert.equal(indicators[1]?.getAttribute("aria-label"), "Slide 2");
  assert.equal(indicators[1]?.getAttribute("aria-controls"), "gallery-slide-1");
  const toggle = handle.getByRole("button", { name: "Start rotation" });
  assert.equal(toggle.getAttribute("data-state"), "stopped");
  handle.unmount();
});

test("previous and next move one slide, scroll the track, and stop at the ends", async () => {
  await withGeometry(async (geometry) => {
    const handle = mountCarousel();
    const root = handle.root();
    const next = handle.getByRole("button", { name: "Next" }) as HTMLButtonElement;
    const previous = handle.getByRole("button", { name: "Previous" }) as HTMLButtonElement;

    await handle.click(next);
    await nextTick();
    assert.equal(root.getAttribute("data-index"), "1");
    assert.equal(slidesOf(root)[1]?.getAttribute("data-state"), "active");
    assert.deepEqual(geometry.scrolls.at(-1), {
      target: viewportOf(root),
      left: 100,
      top: undefined,
      behavior: "smooth",
    });
    await handle.click(next);
    await nextTick();
    assert.equal(next.disabled, true);
    await handle.click(next);
    assert.equal(root.getAttribute("data-index"), "2");
    await handle.click(previous);
    await nextTick();
    assert.equal(root.getAttribute("data-index"), "1");
    assert.deepEqual(
      handle.wrapper.emitted("change")?.map((payload) => payload.slice(0, 3)),
      [
        [1, 0, "next"],
        [2, 1, "next"],
        [1, 2, "previous"],
      ],
    );
    assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[1], [2], [1]]);
    handle.unmount();
  });
});

test("loop wraps navigation at both ends", async () => {
  const handle = mountCarousel({ loop: true });
  const root = handle.root();
  const previous = handle.getByRole("button", { name: "Previous" }) as HTMLButtonElement;
  assert.equal(previous.disabled, false);
  await handle.click(previous);
  assert.equal(root.getAttribute("data-index"), "2");
  await handle.click(handle.getByRole("button", { name: "Next" }));
  assert.equal(root.getAttribute("data-index"), "0");
  handle.unmount();

  const single = mountCarousel({ loop: true }, 1);
  assert.equal((single.getByRole("button", { name: "Next" }) as HTMLButtonElement).disabled, true);
  single.unmount();
});

test("controlled index wins until the parent accepts the request", async () => {
  const handle = mountCarousel({ modelValue: 1 });
  const root = handle.root();
  assert.equal(root.getAttribute("data-index"), "1");
  await handle.click(handle.getByRole("button", { name: "Next" }));
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[2]]);
  assert.equal(root.getAttribute("data-index"), "1");
  await handle.wrapper.setProps({ modelValue: 2 });
  assert.equal(root.getAttribute("data-index"), "2");
  await handle.wrapper.setProps({ modelValue: 99 });
  assert.equal(root.getAttribute("data-index"), "2", "out-of-range values clamp");
  handle.unmount();
});

test("indicators select slides and move roving focus with arrow, Home, and End keys", async () => {
  const handle = mountCarousel();
  const root = handle.root();
  const indicators = indicatorsOf(root);

  await handle.click(indicators[2] as HTMLButtonElement);
  assert.equal(root.getAttribute("data-index"), "2");
  assert.equal(indicators[2]?.getAttribute("aria-selected"), "true");
  assert.equal(indicators[2]?.getAttribute("tabindex"), "0");
  assert.equal(handle.wrapper.emitted("change")?.[0]?.[2], "indicator");

  indicators[2]?.focus();
  const home = await handle.press(indicators[2] as HTMLButtonElement, "Home");
  await nextTick();
  assert.equal(home.keydownPrevented, true);
  assert.equal(root.getAttribute("data-index"), "0");
  assert.ok(handle.activeElement() === indicators[0]);
  await handle.press(indicators[0] as HTMLButtonElement, "ArrowRight");
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "1");
  assert.ok(handle.activeElement() === indicators[1]);
  await handle.press(indicators[1] as HTMLButtonElement, "End");
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "2");
  await handle.press(indicators[2] as HTMLButtonElement, "ArrowRight");
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "2", "non-looping pickers stop at the end");
  await handle.press(indicators[2] as HTMLButtonElement, "ArrowLeft");
  await nextTick();
  assert.equal(root.getAttribute("data-index"), "1");
  handle.unmount();
});

test("the focused viewport maps arrow keys by orientation and reading direction", async () => {
  const handle = mountCarousel({ dir: "rtl" });
  const root = handle.root();
  const viewport = viewportOf(root);
  assert.equal(root.getAttribute("dir"), "rtl");

  const forward = await handle.press(viewport, "ArrowLeft");
  assert.equal(forward.keydownPrevented, true);
  assert.equal(root.getAttribute("data-index"), "1");
  await handle.press(viewport, "ArrowRight");
  assert.equal(root.getAttribute("data-index"), "0");
  await handle.press(viewport, "End");
  assert.equal(root.getAttribute("data-index"), "2");
  await handle.press(viewport, "Home");
  assert.equal(root.getAttribute("data-index"), "0");
  const inner = slidesOf(root)[0]?.querySelector("a");
  assert.ok(inner);
  const ignored = await handle.press(inner, "ArrowLeft");
  assert.equal(ignored.keydownPrevented, false, "keys inside slides belong to their content");
  handle.unmount();

  const vertical = mountCarousel({ orientation: "vertical" });
  const verticalViewport = viewportOf(vertical.root());
  assert.equal(verticalViewport.getAttribute("data-orientation"), "vertical");
  await vertical.press(verticalViewport, "ArrowDown");
  assert.equal(vertical.root().getAttribute("data-index"), "1");
  await vertical.press(verticalViewport, "ArrowUp");
  assert.equal(vertical.root().getAttribute("data-index"), "0");
  assert.equal(vertical.getByRole("tablist").getAttribute("aria-orientation"), "vertical");
  vertical.unmount();
});

test("user scrolling settles on the nearest slide without re-scrolling", async () => {
  await withGeometry(async (geometry) => {
    const handle = mountCarousel();
    const root = handle.root();
    const viewport = viewportOf(root);

    geometry.setPosition(viewport, 180);
    viewport.dispatchEvent(new Event("scroll"));
    await wait(15);
    await nextTick();
    assert.equal(root.getAttribute("data-index"), "2");
    assert.equal(handle.wrapper.emitted("change")?.[0]?.[2], "scroll");
    assert.equal(geometry.scrolls.length, 0, "user scrolls are never corrected");

    geometry.setPosition(viewport, 90);
    viewport.dispatchEvent(new Event("scrollend"));
    await nextTick();
    assert.equal(root.getAttribute("data-index"), "1");
    handle.unmount();
  });
});

test("programmatic scrolls ignore their own settle and respect reduced motion", async () => {
  const previousMatchMedia = globalThis.matchMedia;
  globalThis.matchMedia = ((query: string) => ({
    matches: query.includes("reduce"),
    media: query,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
  })) as unknown as typeof globalThis.matchMedia;
  try {
    await withGeometry(async (geometry) => {
      const handle = mountCarousel({ defaultValue: 2 });
      const root = handle.root();
      const viewport = viewportOf(root);
      await nextTick();
      assert.deepEqual(geometry.scrolls[0], {
        target: viewport,
        left: 200,
        top: undefined,
        behavior: "auto",
      });
      viewport.dispatchEvent(new Event("scrollend"));
      await nextTick();
      assert.equal(root.getAttribute("data-index"), "2");

      await handle.click(handle.getByRole("button", { name: "Previous" }));
      await nextTick();
      assert.equal(geometry.scrolls.at(-1)?.behavior, "auto");
      viewport.dispatchEvent(new Event("scrollend"));
      await nextTick();
      assert.equal(root.getAttribute("data-index"), "1");
      assert.equal(handle.wrapper.emitted("change")?.length, 1);
      handle.unmount();
    });
  } finally {
    globalThis.matchMedia = previousMatchMedia;
  }
});

test("scroll edges disable next when multiple slides fill the viewport", async () => {
  await withGeometry(
    async (geometry) => {
      const handle = mountCarousel({}, 4);
      const root = handle.root();
      const viewport = viewportOf(root);
      const next = handle.getByRole("button", { name: "Next" }) as HTMLButtonElement;
      await nextTick();
      assert.equal(next.disabled, false);

      geometry.setPosition(viewport, 100);
      viewport.dispatchEvent(new Event("scrollend"));
      await nextTick();
      assert.equal(root.getAttribute("data-index"), "2");
      assert.equal(next.disabled, true, "the track cannot scroll further");
      handle.unmount();
    },
    100,
    50,
  );
});

test("mouse drags move the track, page past the threshold, and suppress the trailing click", async () => {
  await withGeometry(async (geometry) => {
    const handle = mountCarousel();
    const root = handle.root();
    const viewport = viewportOf(root);
    const link = slidesOf(root)[0]?.querySelector("a");
    assert.ok(link);
    let clicks = 0;
    link.addEventListener("click", (event) => {
      event.preventDefault();
      clicks += 1;
    });

    link.dispatchEvent(pointer("pointerdown", { pointerType: "mouse", button: 0, clientX: 200 }));
    viewport.dispatchEvent(pointer("pointermove", { pointerType: "mouse", clientX: 199 }));
    assert.equal(root.hasAttribute("data-dragging"), false, "tiny moves are clicks");
    const move = pointer("pointermove", { pointerType: "mouse", clientX: 150 });
    viewport.dispatchEvent(move);
    await nextTick();
    assert.equal(move.defaultPrevented, true);
    assert.equal(geometry.position(viewport), 50);
    assert.equal(viewport.style.scrollSnapType, "none");
    assert.equal(root.getAttribute("data-dragging"), "true");
    assert.equal(viewport.getAttribute("data-dragging"), "true");
    assert.equal(root.getAttribute("data-autoplay"), "stopped");

    viewport.dispatchEvent(pointer("pointerup", { pointerType: "mouse", clientX: 150 }));
    await nextTick();
    assert.equal(viewport.style.scrollSnapType, "");
    assert.equal(root.hasAttribute("data-dragging"), false);
    assert.equal(root.getAttribute("data-index"), "1");
    assert.equal(handle.wrapper.emitted("change")?.[0]?.[2], "drag");
    assert.equal(geometry.scrolls.at(-1)?.left, 100);

    link.click();
    assert.equal(clicks, 0, "the click ending a drag is swallowed");
    link.click();
    assert.equal(clicks, 1);

    link.dispatchEvent(pointer("pointerdown", { pointerType: "touch", clientX: 100 }));
    viewport.dispatchEvent(pointer("pointermove", { pointerType: "touch", clientX: 0 }));
    assert.equal(root.hasAttribute("data-dragging"), false, "touch uses native scrolling");
    handle.unmount();

    const fixed = mountCarousel({ draggable: false });
    const fixedViewport = viewportOf(fixed.root());
    fixedViewport.dispatchEvent(pointer("pointerdown", { pointerType: "mouse", clientX: 200 }));
    fixedViewport.dispatchEvent(pointer("pointermove", { pointerType: "mouse", clientX: 100 }));
    assert.equal(fixed.root().hasAttribute("data-dragging"), false);
    fixed.unmount();
  });
});

test("short drags snap back to the starting slide", async () => {
  await withGeometry(async (geometry) => {
    const handle = mountCarousel({ defaultValue: 1 });
    const root = handle.root();
    const viewport = viewportOf(root);
    await nextTick();
    geometry.scrolls.length = 0;
    viewport.dispatchEvent(
      pointer("pointerdown", { pointerType: "mouse", button: 0, clientX: 100 }),
    );
    viewport.dispatchEvent(pointer("pointermove", { pointerType: "mouse", clientX: 90 }));
    viewport.dispatchEvent(pointer("pointerup", { pointerType: "mouse", clientX: 90 }));
    await nextTick();
    assert.equal(root.getAttribute("data-index"), "1");
    assert.equal(geometry.scrolls.at(-1)?.left, 100, "the track re-aligns to the slide");
    assert.equal(handle.wrapper.emitted("change"), undefined);
    handle.unmount();
  });
});

test("autoplay advances on an interval, rewinds at the end, and silences the live region", async () => {
  const handle = mountCarousel({ autoplay: true, interval: 15 });
  const root = handle.root();
  const viewport = viewportOf(root);
  assert.equal(root.getAttribute("data-autoplay"), "playing");
  assert.equal(viewport.getAttribute("aria-live"), "off");
  assert.ok(handle.getByRole("button", { name: "Stop rotation" }));

  await waitFor(() => root.getAttribute("data-index") === "1");
  await waitFor(() => root.getAttribute("data-index") === "2");
  await waitFor(() => root.getAttribute("data-index") === "0");
  assert.deepEqual(
    handle.wrapper.emitted("change")?.map((payload) => payload.slice(0, 2)),
    [
      [1, 0],
      [2, 1],
      [0, 2],
    ],
    "non-looping autoplay rewinds",
  );
  assert.ok(handle.wrapper.emitted("change")?.every((payload) => payload[2] === "autoplay"));
  handle.unmount();
});

test("hover and pointer focus pause rotation while keyboard focus stops it", async () => {
  const handle = mountCarousel({ autoplay: true, interval: 15 });
  const root = handle.root();

  root.dispatchEvent(pointer("pointerenter", { pointerType: "mouse", bubbles: false }));
  await nextTick();
  assert.equal(root.getAttribute("data-autoplay"), "paused");
  await wait(25);
  assert.equal(root.getAttribute("data-index"), "0");
  root.dispatchEvent(pointer("pointerleave", { pointerType: "mouse", bubbles: false }));
  await nextTick();
  assert.equal(root.getAttribute("data-autoplay"), "playing");

  const toggle = handle.getByRole("button", { name: "Stop rotation" });
  toggle.dispatchEvent(pointer("pointerdown", { pointerType: "mouse" }));
  toggle.focus();
  await nextTick();
  assert.equal(root.getAttribute("data-autoplay"), "playing", "pointer focus does not stop");
  await handle.click(toggle);
  assert.equal(root.getAttribute("data-autoplay"), "stopped");
  assert.deepEqual(handle.wrapper.emitted("update:playing"), [[false]]);
  await handle.click(handle.getByRole("button", { name: "Start rotation" }));
  assert.equal(root.getAttribute("data-autoplay"), "playing");
  toggle.blur();
  await wait(5);

  viewportOf(root).focus();
  await nextTick();
  assert.equal(root.getAttribute("data-autoplay"), "stopped", "keyboard focus stops rotation");
  assert.deepEqual(handle.wrapper.emitted("update:playing"), [[false], [true], [false]]);
  handle.unmount();

  const pausing = mountCarousel({ autoplay: true, focusBehavior: "pause" });
  viewportOf(pausing.root()).focus();
  await nextTick();
  assert.equal(pausing.root().getAttribute("data-autoplay"), "paused");
  viewportOf(pausing.root()).blur();
  await nextTick();
  assert.equal(pausing.root().getAttribute("data-autoplay"), "playing");
  pausing.unmount();

  const touch = mountCarousel({ autoplay: true });
  touch.root().dispatchEvent(pointer("pointerenter", { pointerType: "touch", bubbles: false }));
  await nextTick();
  assert.equal(touch.root().getAttribute("data-autoplay"), "playing");
  touch.unmount();
});

test("reduced motion and hidden documents pause rotation", async () => {
  const previousMatchMedia = globalThis.matchMedia;
  globalThis.matchMedia = ((query: string) => ({
    matches: true,
    media: query,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
  })) as unknown as typeof globalThis.matchMedia;
  try {
    const reduced = mountCarousel({ autoplay: true });
    await nextTick();
    assert.equal(reduced.root().getAttribute("data-autoplay"), "paused");
    reduced.unmount();
    const ignored = mountCarousel({ autoplay: true, respectReducedMotion: false });
    assert.equal(ignored.root().getAttribute("data-autoplay"), "playing");
    ignored.unmount();
  } finally {
    globalThis.matchMedia = previousMatchMedia;
  }

  const descriptor = Object.getOwnPropertyDescriptor(Document.prototype, "visibilityState");
  let visibility: DocumentVisibilityState = "visible";
  Object.defineProperty(document, "visibilityState", {
    configurable: true,
    get: () => visibility,
  });
  try {
    const handle = mountCarousel({ autoplay: true });
    visibility = "hidden";
    document.dispatchEvent(new Event("visibilitychange"));
    await nextTick();
    assert.equal(handle.root().getAttribute("data-autoplay"), "paused");
    visibility = "visible";
    document.dispatchEvent(new Event("visibilitychange"));
    await nextTick();
    assert.equal(handle.root().getAttribute("data-autoplay"), "playing");
    handle.unmount();
  } finally {
    Reflect.deleteProperty(document, "visibilityState");
    if (descriptor && !Object.getOwnPropertyDescriptor(Document.prototype, "visibilityState")) {
      Object.defineProperty(Document.prototype, "visibilityState", descriptor);
    }
  }
});

test("controlled playing and single-slide carousels keep rotation stopped", async () => {
  const handle = mountCarousel({ playing: false, autoplay: true });
  assert.equal(handle.root().getAttribute("data-autoplay"), "stopped");
  await handle.click(handle.getByRole("button", { name: "Start rotation" }));
  assert.deepEqual(handle.wrapper.emitted("update:playing"), [[true]]);
  assert.equal(handle.root().getAttribute("data-autoplay"), "stopped");
  await handle.wrapper.setProps({ playing: true });
  assert.equal(handle.root().getAttribute("data-autoplay"), "playing");
  handle.unmount();

  const single = mountCarousel({ autoplay: true }, 1);
  assert.equal(single.root().getAttribute("data-autoplay"), "stopped");
  const toggle = single
    .root()
    .querySelector<HTMLButtonElement>('[data-vize-ui="carousel-autoplay-toggle"]');
  assert.equal(toggle?.disabled, true);
  single.unmount();
});

test("slides measured out of view become inert while the active slide stays reachable", async () => {
  type Callback = (
    entries: readonly { target: Element; isIntersecting: boolean; intersectionRatio: number }[],
  ) => void;
  const instances: { callback: Callback; init: IntersectionObserverInit; observed: Element[] }[] =
    [];
  class FakeIntersectionObserver {
    readonly observed: Element[] = [];
    constructor(
      readonly callback: Callback,
      readonly init: IntersectionObserverInit,
    ) {
      instances.push(this);
    }
    observe(target: Element): void {
      if (!this.observed.includes(target)) this.observed.push(target);
    }
    unobserve(): void {}
    disconnect(): void {}
  }
  const previous = globalThis.IntersectionObserver;
  globalThis.IntersectionObserver =
    FakeIntersectionObserver as unknown as typeof IntersectionObserver;
  try {
    const handle = mountCarousel();
    const root = handle.root();
    await nextTick();
    const observer = instances[0];
    assert.ok(observer);
    assert.equal(observer.init.threshold, 0.5);
    assert.ok(observer.init.root === viewportOf(root));
    const slides = slidesOf(root);
    assert.equal(observer.observed.length, 3);
    observer.callback(
      slides.map((target, index) => ({
        target,
        isIntersecting: index === 1,
        intersectionRatio: index === 1 ? 1 : 0,
      })),
    );
    await nextTick();
    assert.equal(slides[0]?.hasAttribute("inert"), false, "the active slide is never inert");
    assert.equal(slides[1]?.getAttribute("data-in-view"), "true");
    assert.equal(slides[2]?.hasAttribute("inert"), true);
    assert.equal(slides[2]?.getAttribute("data-in-view"), "false");
    handle.unmount();
  } finally {
    globalThis.IntersectionObserver = previous;
  }
});

test("controls honor preventDefault from click listeners", async () => {
  const blocked = (event: MouseEvent) => event.preventDefault();
  const handle = mountInteraction(CarouselRoot, {
    props: { slideCount: 3 },
    slots: {
      default: () => [
        h(CarouselNext, { onClick: blocked }, () => "Next"),
        h(CarouselPrevious, { onClick: blocked }, () => "Previous"),
        h(CarouselIndicator, { index: 2, onClick: blocked }),
        h(CarouselAutoplayToggle, { onClick: blocked }, () => "Toggle"),
      ],
    },
  });
  for (const button of handle.root().querySelectorAll("button")) {
    button.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  }
  await nextTick();
  assert.equal(handle.root().getAttribute("data-index"), "0");
  assert.equal(handle.root().getAttribute("data-autoplay"), "stopped");
  handle.unmount();
});

test("exposes typed state and imperative navigation and rotation controls", async () => {
  const handle = mountCarousel({ loop: true });
  const exposed = handle.exposes<CarouselRootExpose>();
  assert.equal(exposed.index, 0);
  assert.equal(exposed.slideCount, 3);
  assert.equal(exposed.canScrollPrev, true);
  assert.equal(exposed.autoplay, "stopped");
  assert.equal(exposed.dragging, false);
  assert.equal(exposed.orientation, "horizontal");
  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.scrollPrev(), true);
  assert.equal(exposed.index, 2);
  assert.equal(exposed.scrollNext(), true);
  assert.equal(exposed.scrollTo(1), true);
  assert.equal(exposed.scrollTo(1), false);
  assert.equal(exposed.scrollTo(-4), true);
  assert.equal(exposed.index, 0);
  assert.equal(exposed.play(), true);
  assert.equal(exposed.play(), false);
  await nextTick();
  assert.equal(exposed.autoplay, "playing");
  assert.equal(exposed.stop(), true);
  assert.deepEqual(
    handle.wrapper.emitted("change")?.map((payload) => payload[2]),
    ["api", "api", "api", "api"],
  );
  handle.unmount();
});

test("messages localize role descriptions and slide and indicator names", () => {
  const handle = mountCarousel({
    messages: {
      carousel: "カルーセル",
      slide: "スライド",
      slideLabel: (position: number, count: number) => `${count}枚中${position}枚目`,
      indicatorLabel: (position: number) => `スライド${position}`,
    },
  });
  const root = handle.root();
  assert.equal(root.getAttribute("aria-roledescription"), "カルーセル");
  assert.equal(slidesOf(root)[1]?.getAttribute("aria-roledescription"), "スライド");
  assert.equal(slidesOf(root)[1]?.getAttribute("aria-label"), "3枚中2枚目");
  assert.equal(indicatorsOf(root)[2]?.getAttribute("aria-label"), "スライド3");
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  for (const [part, props] of [
    [CarouselViewport, {}],
    [CarouselSlide, { index: 0 }],
    [CarouselPrevious, {}],
    [CarouselNext, {}],
    [CarouselIndicatorGroup, {}],
    [CarouselIndicator, { index: 0 }],
    [CarouselAutoplayToggle, {}],
  ] as const) {
    assert.throws(
      () => mountInteraction(part, { props }),
      /VIZE_UI_CONTEXT_MISSING: Carousel requires a matching provider/,
    );
  }
});
