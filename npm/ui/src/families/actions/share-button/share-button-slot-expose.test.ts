import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import { settle } from "./share-button-test-utils.ts";
import ShareButton from "./share-button.vue";
import type { ShareButtonExpose, ShareButtonSlotState } from "./share-button.ts";

test("supports custom labels and slot rendering", async () => {
  const handle = mountInteraction(ShareButton, {
    props: {
      action: () => {},
      idleLabel: "Share report",
      sharedLabel: "Report shared",
      sharingLabel: "Sharing report",
      text: "Report body",
      title: "Report",
    },
    slots: {
      default: (state: ShareButtonSlotState) =>
        h("span", { "data-slot-state": state.state }, `${state.label}:${state.payload.title}`),
    },
  });
  const button = handle.getByRole("button", { name: "Share report:Report" });

  assert.equal(button.textContent, "Share report:Report");
  assert.equal(button.querySelector("[data-slot-state]")?.getAttribute("data-slot-state"), "idle");
  await handle.click(button);
  await settle();
  assert.equal(button.getAttribute("data-state"), "shared");
  assert.equal(button.textContent, "Report shared:Report");
  assert.equal(
    button.querySelector("[data-slot-state]")?.getAttribute("data-slot-state"),
    "shared",
  );
  handle.unmount();
});

test("exposes live state and focus", async () => {
  const handle = mountInteraction(ShareButton, {
    props: {
      action: () => {},
      ariaLabel: "Share invite link",
      sharedLabel: "Invite shared",
      title: "Invite",
      url: "https://vize.dev/invite",
    },
  });
  const exposed = handle.exposes<ShareButtonExpose>();
  const button = handle.getByRole("button", { name: "Share invite link" });

  assert.ok(exposed.element === button);
  assert.equal(exposed.disabled, false);
  assert.equal(exposed.sharing, false);
  assert.equal(exposed.unavailable, false);
  assert.equal(exposed.state, "idle");
  assert.deepEqual(exposed.payload, {
    title: "Invite",
    url: "https://vize.dev/invite",
  });
  assert.equal(exposed.label, "Share");
  exposed.focus();
  assert.ok(handle.activeElement() === button);

  await handle.click(button);
  await settle();
  assert.equal(exposed.state, "shared");
  assert.equal(exposed.sharing, false);
  assert.equal(exposed.label, "Invite shared");
  handle.unmount();
});
