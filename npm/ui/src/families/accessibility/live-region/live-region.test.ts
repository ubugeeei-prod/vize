import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import { createLiveRegion, useLiveRegion } from "./live-region.ts";
import LiveRegion from "./live-region.vue";

async function flushAnnounce(): Promise<void> {
  await Promise.resolve();
  await nextTick();
}

test("renders an empty polite live region", () => {
  const handle = mountInteraction(LiveRegion);
  assert.equal(handle.root().getAttribute("data-vize-ui"), "live-region");
  assert.equal(handle.root().getAttribute("aria-live"), "polite");
  assert.equal(handle.root().getAttribute("role"), "status");
  assert.equal(handle.root().textContent?.trim(), "");
  handle.unmount();
});

test("announces text after clearing", async () => {
  const region = createLiveRegion();
  region.announce("Saved");
  assert.equal(region.message.value, "");
  await nextTick();
  assert.equal(region.message.value, "Saved");
  region.announce("Saved");
  assert.equal(region.message.value, "");
  await nextTick();
  assert.equal(region.message.value, "Saved");
  region.dispose();

  const handle = mountInteraction(LiveRegion);
  const exposed = handle.exposes<{
    announce: (text: string) => void;
  }>();
  exposed.announce("Saved");
  await nextTick();
  await nextTick();
  assert.equal(handle.root().textContent?.trim(), "Saved");
  handle.unmount();
});

test("switches to an assertive alert region", async () => {
  const handle = mountInteraction(LiveRegion, { props: { politeness: "assertive" } });
  assert.equal(handle.root().getAttribute("aria-live"), "assertive");
  assert.equal(handle.root().getAttribute("role"), "alert");
  handle.unmount();
});

test("clears the current announcement", async () => {
  const region = createLiveRegion();
  region.announce("Busy");
  await flushAnnounce();
  assert.equal(region.message.value, "Busy");
  region.clear();
  assert.equal(region.message.value, "");
  region.dispose();
});

test("exposes the rendered element for composition", () => {
  const handle = mountInteraction(LiveRegion);
  const exposed = handle.exposes<{ element: HTMLElement | null }>();
  assert.ok(exposed.element === handle.root());
  handle.unmount();
});

test("rejects composable use outside an effect scope", () => {
  assert.throws(() => useLiveRegion(), /VIZE_UI_LIVE_REGION_SETUP/);
});
