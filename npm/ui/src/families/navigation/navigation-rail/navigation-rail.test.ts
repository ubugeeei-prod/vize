import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import NavigationRail from "./navigation-rail.vue";
import NavigationRailItem from "./navigation-rail-item.vue";
import NavigationRailToggle from "./navigation-rail-toggle.vue";
import type { NavigationRailExpose } from "./navigation-rail-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

const destinations = ["home", "search", "inbox"] as const;

function mountRail(props: Record<string, unknown> = {}) {
  return mountInteraction(NavigationRail, {
    props: { destinations, id: "rail", ...props },
    record: ["update:modelValue", "select", "update:expanded"],
    slots: {
      default: () => [
        h(NavigationRailToggle, { ariaLabel: "Toggle navigation" }),
        h(NavigationRailItem, { value: "home", href: "/" }, { default: () => "Home" }),
        h(NavigationRailItem, { value: "search" }, { default: () => "Search" }),
        h(NavigationRailItem, { value: "inbox", badge: 3 }, { default: () => "Inbox" }),
      ],
    },
  });
}

function items(root: HTMLElement): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>('[data-vize-ui="navigation-rail-item"]')];
}

function toggle(root: HTMLElement): HTMLButtonElement {
  const button = root.querySelector<HTMLButtonElement>('[data-vize-ui="navigation-rail-toggle"]');
  assert.ok(button);
  return button;
}

test("renders a labelled navigation landmark with links, buttons, and badges", () => {
  const handle = mountRail({ defaultValue: "home" });
  const root = handle.root();
  const [home, search, inbox] = items(root);

  assert.equal(root.tagName, "NAV");
  assert.equal(root.id, "rail");
  assert.equal(root.getAttribute("aria-label"), "Primary");
  assert.equal(root.getAttribute("data-count"), "3");
  assert.equal(root.getAttribute("data-expanded"), "false");
  assert.equal(home?.tagName, "A");
  assert.equal(home?.getAttribute("href"), "/");
  assert.equal(home?.getAttribute("aria-current"), "page");
  assert.equal(search?.tagName, "BUTTON");
  assert.equal(search?.getAttribute("type"), "button");
  assert.equal(search?.getAttribute("aria-current"), null);
  assert.equal(inbox?.querySelector('[data-vize-ui="navigation-rail-badge"]')?.textContent, "3");
  handle.unmount();
});

test("activating an item moves aria-current and emits typed selections", async () => {
  const handle = mountRail();
  const [, search] = items(handle.root());
  search?.click();
  await nextTick();
  assert.equal(search?.getAttribute("aria-current"), "page");
  assert.equal(search?.getAttribute("data-state"), "active");
  assert.deepEqual(
    handle.recorded().map((entry) => [entry.event, entry.payload[0]]),
    [
      ["update:modelValue", "search"],
      ["select", "search"],
    ],
  );
  handle.unmount();
});

test("the toggle expands and collapses the rail and names what it controls", async () => {
  const handle = mountRail();
  const root = handle.root();
  const button = toggle(root);
  assert.equal(button.getAttribute("type"), "button");
  assert.equal(button.getAttribute("aria-label"), "Toggle navigation");
  assert.equal(button.getAttribute("aria-controls"), "rail");
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(button.getAttribute("data-state"), "collapsed");

  button.click();
  await nextTick();
  assert.equal(button.getAttribute("aria-expanded"), "true");
  assert.equal(button.getAttribute("data-state"), "expanded");
  assert.equal(root.getAttribute("data-expanded"), "true");
  assert.deepEqual(
    items(root).map((item) => item.getAttribute("data-expanded")),
    ["true", "true", "true"],
  );
  assert.equal(items(root)[1]?.textContent, "Search", "labels stay in the DOM in both states");

  button.click();
  await nextTick();
  assert.equal(root.getAttribute("data-expanded"), "false");
  assert.deepEqual(
    handle.recorded().map((entry) => [entry.event, entry.payload[0]]),
    [
      ["update:expanded", true],
      ["update:expanded", false],
    ],
  );
  handle.unmount();
});

test("controlled expansion wins and the exposed surface requests changes", async () => {
  const handle = mountRail({ expanded: true });
  const root = handle.root();
  toggle(root).click();
  await nextTick();
  assert.equal(root.getAttribute("data-expanded"), "true", "the controlled value wins");
  assert.deepEqual(
    handle.recorded().map((entry) => [entry.event, entry.payload[0]]),
    [["update:expanded", false]],
  );
  handle.unmount();

  const uncontrolled = mountRail({ defaultExpanded: true });
  const exposed = uncontrolled.exposes<NavigationRailExpose>();
  assert.equal(exposed.id, "rail");
  assert.equal(exposed.expanded, true);
  exposed.toggle();
  await nextTick();
  assert.equal(uncontrolled.root().getAttribute("data-expanded"), "false");
  exposed.setExpanded(true);
  await nextTick();
  assert.equal(uncontrolled.root().getAttribute("data-expanded"), "true");
  uncontrolled.unmount();
});

test("a disabled toggle keeps the expanded state", async () => {
  const handle = mountInteraction(NavigationRail, {
    props: { destinations },
    record: ["update:expanded"],
    slots: {
      default: () => [h(NavigationRailToggle, { disabled: true }, { default: () => "Menu" })],
    },
  });
  const button = toggle(handle.root());
  assert.equal(button.disabled, true);
  button.click();
  await nextTick();
  assert.equal(handle.root().getAttribute("data-expanded"), "false");
  assert.deepEqual(handle.recorded(), []);
  assert.equal(button.getAttribute("aria-controls"), handle.root().id);
  assert.notEqual(handle.root().id, "", "a deterministic id is generated without an id prop");
  handle.unmount();
});

test("controlled values win, and disabled or unknown items do not activate", async () => {
  const handle = mountInteraction(NavigationRail, {
    props: { destinations, modelValue: "home", ariaLabelledby: "nav-title" },
    record: ["update:modelValue"],
    slots: {
      default: () => [
        h(NavigationRailItem, { value: "inbox", disabled: true }, { default: () => "Inbox" }),
        h(
          NavigationRailItem,
          { value: "search", href: "/s", disabled: true },
          { default: () => "Search" },
        ),
        h(
          NavigationRailItem,
          { value: "unknown", href: "javascript:alert(1)" },
          { default: () => "?" },
        ),
        h(NavigationRailItem, { value: "home" }, { default: () => "Home" }),
      ],
    },
  });
  const root = handle.root();
  const [inbox, search, unknown, home] = items(root);
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.getAttribute("aria-labelledby"), "nav-title");
  assert.ok(inbox instanceof HTMLButtonElement && inbox.disabled);
  assert.equal(search?.getAttribute("href"), null);
  assert.equal(search?.getAttribute("aria-disabled"), "true");
  assert.equal(unknown?.getAttribute("href"), null, "script URLs are never rendered");
  search?.click();
  unknown?.click();
  await nextTick();
  assert.deepEqual(handle.recorded(), []);
  assert.equal(home?.getAttribute("aria-current"), "page");
  handle.unmount();
});

test("items and the toggle require a NavigationRail provider", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(
      () => mountInteraction(NavigationRailItem, { props: { value: "a" } }),
      /VIZE_UI_CONTEXT_MISSING: NavigationRail/,
    );
    assert.throws(
      () => mountInteraction(NavigationRailToggle),
      /VIZE_UI_CONTEXT_MISSING: NavigationRail/,
    );
  } finally {
    console.warn = warn;
  }
});
