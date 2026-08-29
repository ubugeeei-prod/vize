import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import Alert from "./alert.vue";
import type { AlertSlotState } from "./alert.ts";
import { mountInteraction } from "./testing/mount.ts";

test("renders an assertive alert with labelling, description, variant, and open state", () => {
  const handle = mountInteraction(Alert, {
    props: {
      id: "release-alert",
      variant: "warning",
      ariaLabel: "Release warning",
      ariaDescribedby: "release-help",
    },
    slots: { default: "Deployment is delayed" },
  });
  const alert = handle.getByRole("alert", { name: "Release warning" });

  assert.equal(alert.tagName, "DIV");
  assert.equal(alert.id, "release-alert");
  assert.equal(alert.getAttribute("aria-live"), "assertive");
  assert.equal(alert.getAttribute("aria-atomic"), "true");
  assert.equal(alert.getAttribute("aria-describedby"), "release-help");
  assert.equal(alert.getAttribute("data-vize-ui"), "alert");
  assert.equal(alert.getAttribute("data-state"), "open");
  assert.equal(alert.getAttribute("data-variant"), "warning");
  assert.equal(alert.hasAttribute("hidden"), false);
  assert.equal(alert.textContent, "Deployment is delayed");
  handle.unmount();
});

test("switches to a polite status region without forcing atomic announcements", () => {
  const labelled = document.createElement("span");
  labelled.id = "sync-label";
  labelled.textContent = "Sync status";
  document.body.append(labelled);
  const handle = mountInteraction(Alert, {
    props: {
      role: "status",
      variant: "success",
      atomic: false,
      ariaLabelledby: "sync-label",
    },
    slots: { default: "All files saved" },
  });
  const status = handle.getByRole("status", { name: "Sync status" });

  assert.equal(status.getAttribute("aria-live"), "polite");
  assert.equal(status.getAttribute("aria-atomic"), "false");
  assert.equal(status.getAttribute("aria-labelledby"), "sync-label");
  assert.equal(status.getAttribute("data-state"), "open");
  assert.equal(status.getAttribute("data-variant"), "success");
  handle.unmount();
  labelled.remove();
});

test("closed alerts stay mounted but are hidden from user agents", async () => {
  const handle = mountInteraction(Alert, {
    props: { open: false, ariaLabel: "Offline", variant: "danger" },
    slots: { default: "Connection lost" },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("role"), "alert");
  assert.equal(root.getAttribute("hidden"), "");
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.equal(root.getAttribute("data-variant"), "danger");
  assert.ok((await handle.tab()) === null, "a non-interactive alert must not join tab order");
  handle.unmount();
});

test("exposes slot state for application-owned chrome and dismissal", async () => {
  const handle = mountInteraction(Alert, {
    props: { role: "status", variant: "info", open: true },
    slots: {
      default: (state: AlertSlotState) =>
        `${state.role}:${state.variant}:${state.state}:${state.open}`,
    },
  });

  assert.equal(handle.root().textContent, "status:info:open:true");
  await handle.wrapper.setProps({ open: false, variant: "success" });
  assert.equal(handle.root().textContent, "status:success:closed:false");
  assert.ok(
    handle.exposes<{ element: HTMLDivElement | null }>().element === handle.root(),
    "the exposed element must be the rendered alert root",
  );
  handle.unmount();
});
