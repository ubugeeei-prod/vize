import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import { mountInteraction } from "./testing/mount.ts";
import Portal from "./portal.vue";

test("renders slotted content", () => {
  const handle = mountInteraction(Portal, {
    props: { disabled: true },
    slots: { default: () => h("button", { type: "button" }, "Open") },
  });

  try {
    assert.equal(handle.root().getAttribute("data-vize-ui"), "portal-host");
    const control = handle.getByRole("button", { name: "Open" });
    assert.equal(control.textContent, "Open");
  } finally {
    handle.unmount();
  }
});

test("moves content into the document body", async () => {
  const handle = mountInteraction(Portal, {
    slots: { default: () => h("p", "Portalled") },
  });

  try {
    await nextTick();
    const portal = [...document.body.querySelectorAll('[data-vize-ui="portal"]')].find(
      (node) => node.textContent === "Portalled",
    );
    assert.ok(portal instanceof HTMLElement);
    assert.equal(portal.parentElement, document.body);
  } finally {
    handle.unmount();
  }
});

test("keeps content in place when disabled", async () => {
  const handle = mountInteraction(Portal, {
    props: { disabled: true },
    slots: { default: "Inline" },
  });

  try {
    await nextTick();
    const portal = handle.root().querySelector('[data-vize-ui="portal"]');
    assert.ok(portal instanceof HTMLElement);
    assert.equal(portal.textContent, "Inline");
    assert.ok(handle.root().contains(portal));
  } finally {
    handle.unmount();
  }
});

test("exposes the rendered element for composition", () => {
  const handle = mountInteraction(Portal, {
    props: { disabled: true },
    slots: { default: "Host" },
  });
  try {
    const exposed = handle.exposes<{ element: HTMLElement | null }>();
    const portal = handle.root().querySelector('[data-vize-ui="portal"]');
    assert.ok(exposed.element === portal);
  } finally {
    handle.unmount();
  }
});
