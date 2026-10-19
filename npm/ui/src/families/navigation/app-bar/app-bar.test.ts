import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { h, nextTick } from "vue";

import AppBar from "./app-bar.vue";
import type { AppBarExpose } from "./app-bar-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

let windowOffset = 0;
Object.defineProperty(window, "scrollY", { configurable: true, get: () => windowOffset });

afterEach(() => {
  windowOffset = 0;
});

async function scrollWindow(offset: number): Promise<void> {
  windowOffset = offset;
  window.dispatchEvent(new Event("scroll"));
  await nextTick();
}

function mountBar(props: Record<string, unknown> = {}) {
  return mountInteraction(AppBar, {
    props,
    record: ["visibilityChange"],
    slots: {
      start: () => h("button", { type: "button", "aria-label": "Open menu" }, "≡"),
      default: () => h("span", { id: "app-title" }, "Inbox"),
      end: () => h("button", { type: "button" }, "Search"),
    },
  });
}

test("renders a header landmark with start, content, and end parts", () => {
  const handle = mountBar({ ariaLabelledby: "app-title" });
  const root = handle.root();
  assert.equal(root.tagName, "HEADER");
  assert.equal(root.getAttribute("aria-labelledby"), "app-title");
  assert.equal(root.getAttribute("data-state"), "visible");
  assert.equal(root.hasAttribute("data-elevated"), false);
  assert.deepEqual(
    [...root.children].map((child) => child.getAttribute("data-vize-ui")),
    ["app-bar-start", "app-bar-content", "app-bar-end"],
  );
  handle.unmount();

  const plain = mountInteraction(AppBar, {
    props: { as: "div", ariaLabel: "Section" },
    slots: { default: () => "Title" },
  });
  assert.equal(plain.root().tagName, "DIV");
  assert.equal(plain.root().getAttribute("aria-label"), "Section");
  assert.deepEqual(
    [...plain.root().children].map((child) => child.getAttribute("data-vize-ui")),
    ["app-bar-content"],
    "empty start and end slots render no parts",
  );
  plain.unmount();
});

test("reports elevation once the window scrolls past the threshold", async () => {
  const handle = mountBar({ elevateThreshold: 8 });
  const root = handle.root();
  await scrollWindow(8);
  assert.equal(root.hasAttribute("data-elevated"), false, "the threshold itself is not past it");
  await scrollWindow(9);
  assert.equal(root.getAttribute("data-elevated"), "true");
  assert.equal(handle.exposes<AppBarExpose>().scrollTop, 9);
  await scrollWindow(0);
  assert.equal(root.hasAttribute("data-elevated"), false);
  assert.equal(root.getAttribute("data-state"), "visible", "without hideOnScroll it never hides");
  handle.unmount();
});

test("hideOnScroll hides while scrolling down past the threshold and reveals on scroll up", async () => {
  const handle = mountBar({ hideOnScroll: true, hideThreshold: 50 });
  const root = handle.root();
  await scrollWindow(40);
  assert.equal(root.getAttribute("data-state"), "visible", "within the threshold it stays");
  await scrollWindow(120);
  assert.equal(root.getAttribute("data-state"), "hidden");
  assert.equal(root.getAttribute("data-hidden"), "true");
  await scrollWindow(110);
  assert.equal(root.getAttribute("data-state"), "visible", "any upward scroll reveals it");
  await scrollWindow(200);
  await scrollWindow(30);
  assert.deepEqual(
    handle.recorded().map((entry) => entry.payload[0]),
    [true, false, true, false],
  );
  handle.unmount();
});

test("focus inside a hidden bar reveals it and keeps it visible", async () => {
  const handle = mountBar({ hideOnScroll: true, hideThreshold: 0 });
  const root = handle.root();
  await scrollWindow(100);
  assert.equal(root.getAttribute("data-state"), "hidden");
  const menu = root.querySelector<HTMLButtonElement>('[aria-label="Open menu"]');
  assert.ok(menu);
  menu.focus();
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "visible", "keyboard focus reveals the bar");
  await scrollWindow(300);
  assert.equal(root.getAttribute("data-state"), "visible", "it stays while focus is inside");
  menu.blur();
  await nextTick();
  await scrollWindow(400);
  assert.equal(root.getAttribute("data-state"), "hidden");
  handle.exposes<AppBarExpose>().show();
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "visible", "show() reveals it");
  handle.unmount();
});

test("follows an element scroll container given as a selector", async () => {
  const scroller = document.createElement("main");
  scroller.id = "scroller";
  document.body.append(scroller);
  let offset = 0;
  Object.defineProperty(scroller, "scrollTop", { configurable: true, get: () => offset });
  const handle = mountBar({ target: "#scroller", hideOnScroll: true, hideThreshold: 10 });
  const root = handle.root();
  await scrollWindow(500);
  assert.equal(root.getAttribute("data-state"), "visible", "window scrolling is ignored");
  offset = 80;
  scroller.dispatchEvent(new Event("scroll"));
  await nextTick();
  assert.equal(root.getAttribute("data-elevated"), "true");
  assert.equal(root.getAttribute("data-state"), "hidden");
  offset = 90;
  assert.equal(handle.exposes<AppBarExpose>().refresh(), 90, "refresh() re-reads the offset");
  handle.unmount();
  scroller.remove();
});

test("turning hideOnScroll off reveals a hidden bar, and invalid thresholds fall back", async () => {
  const handle = mountBar({ hideOnScroll: true, hideThreshold: Number.NaN });
  const root = handle.root();
  await scrollWindow(60);
  assert.equal(root.getAttribute("data-state"), "visible", "NaN falls back to the 64px default");
  await scrollWindow(70);
  assert.equal(root.getAttribute("data-state"), "hidden");
  await handle.wrapper.setProps({ hideOnScroll: false });
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "visible");
  handle.unmount();
});
