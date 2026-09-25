import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import BottomNavigation from "./bottom-navigation.vue";
import BottomNavigationItem from "./bottom-navigation-item.vue";
import { mountInteraction } from "../../../testing/mount.ts";

const destinations = ["home", "search", "inbox"] as const;

function mountNav(props: Record<string, unknown> = {}) {
  return mountInteraction(BottomNavigation, {
    props: { destinations, ...props },
    record: ["update:modelValue", "select"],
    slots: {
      default: () => [
        h(BottomNavigationItem, { value: "home", href: "/" }, { default: () => "Home" }),
        h(BottomNavigationItem, { value: "search" }, { default: () => "Search" }),
        h(BottomNavigationItem, { value: "inbox", badge: 3 }, { default: () => "Inbox" }),
      ],
    },
  });
}

function items(root: HTMLElement): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>('[data-vize-ui="bottom-navigation-item"]')];
}

test("renders a labelled navigation landmark with links, buttons, and badges", () => {
  const handle = mountNav({ defaultValue: "home" });
  const root = handle.root();
  const [home, search, inbox] = items(root);

  assert.equal(root.tagName, "NAV");
  assert.equal(root.getAttribute("aria-label"), "Primary");
  assert.equal(root.getAttribute("data-count"), "3");
  assert.equal(home?.tagName, "A");
  assert.equal(home?.getAttribute("href"), "/");
  assert.equal(home?.getAttribute("aria-current"), "page");
  assert.equal(search?.tagName, "BUTTON");
  assert.equal(search?.getAttribute("type"), "button");
  assert.equal(search?.getAttribute("aria-current"), null);
  assert.equal(inbox?.querySelector('[data-vize-ui="bottom-navigation-badge"]')?.textContent, "3");
  handle.unmount();
});

test("activating an item moves aria-current and emits typed selections", async () => {
  const handle = mountNav();
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

test("controlled values win, and disabled or unknown items do not activate", async () => {
  const handle = mountInteraction(BottomNavigation, {
    props: { destinations, modelValue: "home", ariaLabelledby: "nav-title" },
    record: ["update:modelValue"],
    slots: {
      default: () => [
        h(BottomNavigationItem, { value: "inbox", disabled: true }, { default: () => "Inbox" }),
        h(
          BottomNavigationItem,
          { value: "search", href: "/s", disabled: true },
          { default: () => "Search" },
        ),
        h(
          BottomNavigationItem,
          { value: "unknown", href: "javascript:alert(1)" },
          { default: () => "?" },
        ),
        h(BottomNavigationItem, { value: "home" }, { default: () => "Home" }),
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

test("items require a BottomNavigation provider", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(
      () => mountInteraction(BottomNavigationItem, { props: { value: "a" } }),
      /VIZE_UI_CONTEXT_MISSING: BottomNavigation/,
    );
  } finally {
    console.warn = warn;
  }
});
