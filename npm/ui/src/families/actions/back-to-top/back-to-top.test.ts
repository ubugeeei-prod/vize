import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { BackToTopExpose } from "./back-to-top.ts";
import BackToTop from "./back-to-top.vue";
import { mountInteraction } from "../../../testing/mount.ts";

interface ScrollRequest {
  readonly top: number | undefined;
  readonly behavior: ScrollBehavior | undefined;
}

function scroller(id: string): { element: HTMLDivElement; requests: ScrollRequest[] } {
  const element = document.createElement("div");
  element.id = id;
  document.body.append(element);
  const requests: ScrollRequest[] = [];
  Object.defineProperty(element, "scrollTo", {
    configurable: true,
    value: (options: ScrollToOptions) => {
      requests.push({ behavior: options.behavior, top: options.top });
      element.scrollTop = options.top ?? 0;
      element.dispatchEvent(new Event("scroll"));
    },
  });
  return { element, requests };
}

async function scrollTo(element: HTMLElement, top: number): Promise<void> {
  element.scrollTop = top;
  element.dispatchEvent(new Event("scroll"));
  await nextTick();
}

test("stays hidden until the container passes the threshold", async () => {
  const { element } = scroller("feed");
  const handle = mountInteraction(BackToTop, { props: { target: "#feed", threshold: 300 } });
  await nextTick();
  const button = handle.root();

  assert.ok(button instanceof HTMLButtonElement);
  assert.equal(button.type, "button");
  assert.equal(button.textContent, "Back to top");
  assert.equal(button.hidden, true);
  assert.equal(button.getAttribute("data-state"), "hidden");

  await scrollTo(element, 299);
  assert.equal(button.hidden, true);
  await scrollTo(element, 300);
  assert.equal(button.hidden, false);
  assert.equal(button.getAttribute("data-state"), "visible");

  handle.unmount();
  element.remove();
});

test("activation scrolls smoothly to the top and focuses the container", async () => {
  const { element, requests } = scroller("article");
  const handle = mountInteraction(BackToTop, {
    props: { target: element, threshold: 10 },
  });
  await nextTick();
  await scrollTo(element, 900);
  const button = handle.root();
  button.focus();

  await handle.click(button);
  assert.deepEqual(requests, [{ behavior: "smooth", top: 0 }]);
  assert.ok(handle.activeElement() === element, "focus moves to the top of the container");
  assert.equal(element.getAttribute("tabindex"), "-1");
  assert.equal(handle.wrapper.emitted("scroll-top")?.[0]?.[0], element);
  await nextTick();
  assert.equal(button.hidden, true, "the button hides once it loses focus at the top");

  handle.unmount();
  element.remove();
});

test("focusTarget receives focus and a focused button stays visible", async () => {
  const { element } = scroller("list");
  const heading = document.createElement("h1");
  heading.id = "page-title";
  document.body.append(heading);
  const handle = mountInteraction(BackToTop, {
    props: { target: element, threshold: 10, focusTarget: "#page-title", ariaLabel: "Top" },
  });
  await nextTick();
  await scrollTo(element, 50);
  const button = handle.getByRole("button", { name: "Top" });
  button.focus();
  await scrollTo(element, 0);
  assert.equal(button.hidden, false, "a focused button is not yanked out of the tab order");

  await handle.click(button);
  assert.ok(handle.activeElement() === heading);

  handle.unmount();
  element.remove();
  heading.remove();
});

test("reduced motion downgrades smooth scrolling and click is preventable", async () => {
  const { element, requests } = scroller("reduced");
  const originalMatchMedia = window.matchMedia;
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: (query: string) => ({ matches: query.includes("reduce"), media: query }),
  });
  try {
    const handle = mountInteraction(BackToTop, { props: { target: element, threshold: 1 } });
    await nextTick();
    await scrollTo(element, 40);
    await handle.click(handle.root());
    assert.deepEqual(requests, [{ behavior: "auto", top: 0 }]);
    handle.unmount();

    const prevented = mountInteraction(BackToTop, {
      props: {
        target: element,
        threshold: 1,
        onClick: (event: MouseEvent) => event.preventDefault(),
      },
    });
    await nextTick();
    await scrollTo(element, 40);
    await prevented.click(prevented.root());
    assert.equal(requests.length, 1);
    assert.equal(prevented.wrapper.emitted("scroll-top"), undefined);
    prevented.unmount();
  } finally {
    Object.defineProperty(window, "matchMedia", { configurable: true, value: originalMatchMedia });
    element.remove();
  }
});

test("window scrolling is the default container", async () => {
  const handle = mountInteraction(BackToTop, { props: { threshold: 100 } });
  await nextTick();
  Object.defineProperty(window, "scrollY", { configurable: true, value: 250 });
  window.dispatchEvent(new Event("scroll"));
  await nextTick();
  assert.equal(handle.root().hidden, false);
  Object.defineProperty(window, "scrollY", { configurable: true, value: 0 });
  window.dispatchEvent(new Event("scroll"));
  await nextTick();
  assert.equal(handle.root().hidden, true);
  handle.unmount();
});

test("expose reads offsets and scrolls on demand", async () => {
  const { element, requests } = scroller("exposed");
  let exposed: BackToTopExpose | null = null;
  const Probe = defineComponent({
    name: "BackToTopExposeProbe",
    setup: () => () =>
      h(BackToTop, {
        target: element,
        ref: (value) => {
          exposed = value as BackToTopExpose | null;
        },
      }),
  });
  const handle = mountInteraction(Probe);
  await nextTick();
  if (exposed === null) assert.fail("BackToTop must expose its API");
  const api: BackToTopExpose = exposed;

  element.scrollTop = 800;
  assert.equal(api.refresh(), 800);
  await nextTick();
  assert.equal(api.visible, true);
  assert.equal(api.state, "visible");
  assert.equal(api.scrollToTop(), true);
  assert.equal(requests.length, 1);
  assert.ok(api.element instanceof HTMLButtonElement);

  handle.unmount();
  element.remove();
});
