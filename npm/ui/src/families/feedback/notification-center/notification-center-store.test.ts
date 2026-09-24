import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { connectNotificationSource, createNotificationStore } from "./notification-center-store.ts";
import type { NotificationInput } from "./notification-center-types.ts";

function clock(start = 1_000) {
  let time = start;
  return { now: () => time, tick: (ms = 1) => (time += ms) };
}

test("adds notifications newest first with generated ids and injected timestamps", () => {
  const time = clock();
  const store = createNotificationStore<{ readonly href: string }>({ now: time.now });
  const first = store.add({ title: "Build passed", data: { href: "/builds/1" } });
  time.tick();
  const second = store.add({ title: "Review requested", type: "info", group: "reviews" });

  assert.equal(first, "notification-1");
  assert.equal(second, "notification-2");
  assert.deepEqual(
    store.notifications.value.map((record) => record.id),
    [second, first],
  );
  const record = store.get(first);
  assert.equal(record?.createdAt, 1_000);
  assert.equal(record?.type, "neutral");
  assert.equal(record?.description, null);
  assert.equal(record?.data?.href, "/builds/1");
  assert.ok(Object.isFrozen(record));
  assert.equal(store.unreadCount.value, 2);
});

test("re-adding an id replaces it in place", () => {
  const store = createNotificationStore({ now: () => 0 });
  store.add({ id: "sync", title: "Syncing" });
  store.add({ id: "sync", title: "Synced", type: "success" });
  assert.equal(store.notifications.value.length, 1);
  assert.equal(store.get("sync")?.title, "Synced");
});

test("read, archive, update, and remove report whether anything changed", () => {
  const time = clock();
  const store = createNotificationStore({ now: time.now });
  const id = store.add({ title: "Deploy" });

  assert.equal(store.markRead(id), true);
  assert.equal(store.markRead(id), false);
  assert.equal(store.unreadCount.value, 0);
  assert.equal(store.markUnread(id), true);
  time.tick(5);
  assert.equal(store.update(id, { title: "Deploy finished", type: "success" }), true);
  assert.equal(store.get(id)?.updatedAt, 1_005);
  assert.equal(store.update(id, { title: "Deploy finished" }), false);
  assert.equal(store.update("missing", { title: "x" }), false);
  assert.equal(store.archive(id), true);
  assert.deepEqual(store.notifications.value, []);
  assert.equal(store.archived.value.length, 1);
  assert.equal(store.unreadCount.value, 0, "archived notifications are not counted");
  assert.equal(store.unarchive(id), true);
  assert.equal(store.remove(id), true);
  assert.equal(store.remove(id), false);
});

test("markAllRead only touches active unread notifications", () => {
  const store = createNotificationStore({ now: () => 1 });
  store.add({ id: "a", title: "A" });
  store.add({ id: "b", title: "B", read: true });
  store.add({ id: "c", title: "C" });
  store.archive("c");
  assert.equal(store.markAllRead(), 1);
  assert.equal(store.get("c")?.read, false);
});

test("groups preserve first appearance order with per-group unread counts", () => {
  const time = clock();
  const store = createNotificationStore({ now: time.now });
  store.add({ title: "PR 1", group: "reviews" });
  time.tick();
  store.add({ title: "Incident", group: "alerts" });
  time.tick();
  store.add({ title: "PR 2", group: "reviews", read: true });

  assert.deepEqual(
    store.groups.value.map((group) => [group.key, group.notifications.length, group.unreadCount]),
    [
      ["reviews", 2, 1],
      ["alerts", 1, 1],
    ],
  );
});

test("maxLength evicts the oldest notifications", () => {
  const time = clock();
  const store = createNotificationStore({ maxLength: 2, now: time.now });
  for (const title of ["one", "two", "three"]) {
    store.add({ title });
    time.tick();
  }
  assert.deepEqual(
    store.notifications.value.map((record) => record.title),
    ["three", "two"],
  );
});

test("initial notifications and clear", () => {
  const store = createNotificationStore({
    initial: [
      { id: "old", title: "Old", createdAt: 1 },
      { id: "new", title: "New", createdAt: 2 },
    ],
  });
  assert.deepEqual(
    store.notifications.value.map((record) => record.id),
    ["new", "old"],
  );
  store.clear();
  assert.equal(store.notifications.value.length, 0);
});

test("connectNotificationSource feeds the store until unsubscribed", () => {
  const store = createNotificationStore({ now: () => 0 });
  let emit: ((input: NotificationInput) => void) | null = null;
  let unsubscribed = false;
  const disconnect = connectNotificationSource(store, (push) => {
    emit = push;
    return () => {
      unsubscribed = true;
    };
  });
  if (emit === null) assert.fail("source must receive an emitter");
  const push: (input: NotificationInput) => void = emit;
  push({ title: "Toast dismissed", group: "toasts" });
  assert.equal(store.notifications.value.length, 1);
  disconnect();
  assert.equal(unsubscribed, true);
});

test("invalid options and inputs throw diagnostics", () => {
  assert.throws(() => createNotificationStore({ maxLength: 0 }), /VIZE_UI_NOTIFICATION_OPTION/);
  const store = createNotificationStore();
  assert.throws(
    () => store.add({ title: "x", type: "fatal" as never }),
    /VIZE_UI_NOTIFICATION_OPTION/,
  );
});
