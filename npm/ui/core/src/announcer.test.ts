import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import AnnouncerProvider from "./announcer-provider.vue";
import {
  announcerContext,
  createAnnouncer,
  createBusyAnnouncement,
  useAnnouncer,
  type AnnouncerController,
} from "./announcer.ts";
import { mountInteraction } from "./testing/mount.ts";

/** Record channel-tagged announcements exactly as assistive technology hears them. */
async function transcript(announcer: AnnouncerController, ticks = 16): Promise<readonly string[]> {
  const heard: string[] = [];
  let lastPolite = announcer.politeMessage.value;
  let lastAssertive = announcer.assertiveMessage.value;
  for (let tick = 0; tick < ticks; tick++) {
    await nextTick();
    if (announcer.politeMessage.value !== lastPolite) {
      lastPolite = announcer.politeMessage.value;
      if (lastPolite !== "") heard.push(`polite:${lastPolite}`);
    }
    if (announcer.assertiveMessage.value !== lastAssertive) {
      lastAssertive = announcer.assertiveMessage.value;
      if (lastAssertive !== "") heard.push(`assertive:${lastAssertive}`);
    }
  }
  return heard;
}

test("renders one polite and one assertive region", () => {
  const handle = mountInteraction(AnnouncerProvider);
  assert.equal(handle.root().getAttribute("data-vize-announcer"), "owner");
  const regions = handle.root().querySelectorAll('[data-vize-ui="announcer-region"]');
  assert.equal(regions.length, 2);
  assert.equal(regions[0]?.getAttribute("aria-live"), "polite");
  assert.equal(regions[0]?.getAttribute("role"), "status");
  assert.equal(regions[0]?.textContent?.trim(), "");
  assert.equal(regions[1]?.getAttribute("aria-live"), "assertive");
  assert.equal(regions[1]?.getAttribute("role"), "alert");
  assert.equal(regions[1]?.textContent?.trim(), "");
  handle.unmount();
});

test("queues announcements sequentially", async () => {
  const announcer = createAnnouncer();
  assert.equal(announcer.announce("Saved"), true);
  assert.equal(announcer.announce("Synced"), true);
  assert.equal(announcer.pendingCount.value, 1);
  assert.deepEqual(await transcript(announcer), ["polite:Saved", "polite:Synced"]);
  assert.equal(announcer.pendingCount.value, 0);
  announcer.dispose();
});

test("flushes assertive announcements before polite ones", async () => {
  const announcer = createAnnouncer();
  announcer.announce("First");
  announcer.announce("Second");
  announcer.announce("Session expired", { politeness: "assertive" });
  assert.deepEqual(await transcript(announcer), [
    "polite:First",
    "assertive:Session expired",
    "polite:Second",
  ]);
  announcer.dispose();
});

test("deduplicates identical pending announcements", async () => {
  const announcer = createAnnouncer();
  assert.equal(announcer.announce("Saved"), true);
  assert.equal(announcer.announce("Saved"), false);
  assert.equal(announcer.announce("Saved", { politeness: "assertive" }), true);
  assert.deepEqual(await transcript(announcer), ["polite:Saved", "assertive:Saved"]);
  announcer.dispose();
});

test("coalesces keyed announcements", async () => {
  const announcer = createAnnouncer();
  announcer.announce("Uploading 0%", { key: "upload" });
  announcer.announce("Uploading 40%", { key: "upload" });
  announcer.announce("Uploading 90%", { key: "upload" });
  assert.deepEqual(await transcript(announcer), ["polite:Uploading 0%", "polite:Uploading 90%"]);
  assert.equal(announcer.cancel("upload"), false);
  announcer.dispose();
});

test("announces busy work without flooding", async () => {
  const announcer = createAnnouncer();
  const busy = createBusyAnnouncement(announcer, { label: "Loading results" });
  assert.equal(busy.isBusy.value, true);
  busy.update("Loaded 10 of 40");
  busy.update("Loaded 30 of 40");
  busy.end("40 results loaded");
  assert.equal(busy.isBusy.value, false);
  assert.deepEqual(await transcript(announcer), [
    "polite:Loading results",
    "polite:40 results loaded",
  ]);
  busy.end("Ignored: already ended");
  assert.equal(announcer.pendingCount.value, 0);
  announcer.dispose();
});

test("rejects progress after a busy announcement ends", () => {
  const announcer = createAnnouncer();
  const busy = createBusyAnnouncement(announcer, { label: "Exporting" });
  busy.end();
  assert.throws(() => busy.update("50%"), /VIZE_UI_ANNOUNCER_BUSY/);
  assert.throws(() => createBusyAnnouncement(announcer, { label: "  " }), /VIZE_UI_ANNOUNCER_BUSY/);
  announcer.dispose();
});

test("nested providers reuse the owner's regions", async () => {
  const AnnouncingChild = defineComponent({
    name: "AnnouncerChildProbe",
    setup() {
      const announcer = announcerContext.use();
      announcer.announce("From the nested island");
      return () => h("span", "child");
    },
  });
  const NestedProvider = defineComponent({
    name: "AnnouncerNestedProbe",
    setup() {
      return () => h(AnnouncerProvider, null, { default: () => h(AnnouncingChild) });
    },
  });
  const handle = mountInteraction(AnnouncerProvider, {
    slots: { default: () => h(NestedProvider) },
  });

  const regions = handle.root().querySelectorAll('[data-vize-ui="announcer-region"]');
  assert.equal(regions.length, 2);
  const delegate = handle.root().querySelector('[data-vize-announcer="delegate"]');
  assert.ok(delegate instanceof HTMLElement);
  assert.equal(delegate.querySelectorAll('[data-vize-ui="announcer-region"]').length, 0);

  const politeRegion = handle.root().querySelector('[aria-live="polite"]');
  for (let tick = 0; tick < 8; tick++) await nextTick();
  assert.equal(politeRegion?.textContent?.trim(), "From the nested island");
  handle.unmount();
});

test("clears pending announcements and both channels", async () => {
  const announcer = createAnnouncer({ politeness: "assertive" });
  announcer.announce("Failed");
  await transcript(announcer, 4);
  assert.equal(announcer.assertiveMessage.value, "Failed");
  announcer.announce("One");
  announcer.announce("Two", { politeness: "polite" });
  announcer.clear();
  assert.equal(announcer.pendingCount.value, 0);
  assert.equal(announcer.politeMessage.value, "");
  assert.equal(announcer.assertiveMessage.value, "");
  assert.deepEqual(await transcript(announcer), []);
  announcer.dispose();
});

test("rejects use after dispose", () => {
  const announcer = createAnnouncer();
  announcer.dispose();
  announcer.dispose();
  assert.throws(() => announcer.announce("Saved"), /VIZE_UI_ANNOUNCER_DISPOSED/);
  assert.throws(() => announcer.clear(), /VIZE_UI_ANNOUNCER_DISPOSED/);
  assert.throws(() => announcer.cancel("key"), /VIZE_UI_ANNOUNCER_DISPOSED/);
});

test("rejects composable use outside an effect scope", () => {
  assert.throws(() => useAnnouncer(), /VIZE_UI_ANNOUNCER_SETUP/);
});

test("rejects invalid options", () => {
  const announcer = createAnnouncer();
  assert.throws(
    () => announcer.announce("Saved", { politeness: "off" as never }),
    /VIZE_UI_ANNOUNCER_OPTION/,
  );
  assert.throws(() => announcer.announce("Saved", { key: "" }), /VIZE_UI_ANNOUNCER_OPTION/);
  assert.throws(() => createAnnouncer({ politeness: "off" as never }), /VIZE_UI_ANNOUNCER_OPTION/);
  announcer.dispose();
});
