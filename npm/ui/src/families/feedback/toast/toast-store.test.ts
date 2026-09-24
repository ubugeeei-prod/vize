import assert from "node:assert/strict";

import { afterEach, beforeEach, test, vi } from "vite-plus/test";

import type { ToastDismissReason, ToastRecord } from "./toast.ts";
import { createToastStore, formatToastHotkey, matchesToastHotkey } from "./toast.ts";

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

test("store is inert until started and generates deterministic ids", () => {
  const store = createToastStore();
  const first = store.toast("Saved");
  const second = store.success({ title: "Uploaded", description: "3 files" });

  assert.equal(first, "toast-1");
  assert.equal(second, "toast-2");
  assert.equal(store.get(second)?.type, "success");
  assert.equal(store.get(second)?.duration, 5000);
  vi.advanceTimersByTime(60_000);
  assert.equal(store.visibleToasts.value.length, 2);
  assert.equal(store.get(first)?.open, true);
});

test("started timers auto-dismiss visible toasts and report the timeout reason", () => {
  const store = createToastStore({ duration: 1000 });
  const reasons: ToastDismissReason[] = [];
  const autoClosed: string[] = [];
  store.start();
  const id = store.toast({
    title: "Copied",
    onAutoClose: (toast) => autoClosed.push(toast.id),
    onDismiss: (_toast, reason) => reasons.push(reason),
  });

  vi.advanceTimersByTime(999);
  assert.equal(store.get(id)?.open, true);
  vi.advanceTimersByTime(1);
  assert.equal(store.get(id)?.state, "closed");
  assert.equal(store.get(id)?.dismissReason, "timeout");
  assert.deepEqual(autoClosed, [id]);
  assert.deepEqual(reasons, ["timeout"]);
  assert.equal(store.remove(id), true);
  assert.equal(store.toasts.value.length, 0);
});

test("limit queues extra toasts and high priority jumps ahead of normal and low", () => {
  const store = createToastStore({ limit: 1 });
  const first = store.toast("First");
  const low = store.toast({ title: "Low", priority: "low" });
  const normal = store.toast("Normal");
  const high = store.error({ title: "High", priority: "high" });

  assert.deepEqual(
    store.visibleToasts.value.map((toast) => toast.id),
    [first],
  );
  assert.deepEqual(
    store.queuedToasts.value.map((toast) => toast.id),
    [high, normal, low],
  );

  store.dismiss(first);
  assert.deepEqual(
    store.visibleToasts.value.filter((toast) => toast.open).map((toast) => toast.id),
    [high],
  );
  store.remove(first);
  store.dismiss(high);
  assert.equal(store.queuedToasts.value[0]?.id, low);
});

test("queued toasts keep their full duration until shown and dismissing a queued toast removes it", () => {
  const store = createToastStore({ duration: 1000, limit: 1 });
  store.start();
  const first = store.toast("First");
  const second = store.toast("Second");
  const third = store.toast("Third");

  vi.advanceTimersByTime(900);
  assert.equal(store.dismiss(third), true);
  assert.equal(store.get(third), undefined);
  vi.advanceTimersByTime(100);
  assert.equal(store.get(first)?.open, false);
  store.remove(first);
  vi.advanceTimersByTime(999);
  assert.equal(store.get(second)?.open, true);
  vi.advanceTimersByTime(1);
  assert.equal(store.get(second)?.open, false);
});

test("pause reasons overlap and resume keeps the remaining time", () => {
  const store = createToastStore({ duration: 1000 });
  store.start();
  const id = store.toast("Paused");

  vi.advanceTimersByTime(600);
  store.pause("hover");
  store.pause("focus");
  assert.equal(store.paused.value, true);
  vi.advanceTimersByTime(5000);
  store.resume("hover");
  vi.advanceTimersByTime(5000);
  assert.equal(store.get(id)?.open, true);
  store.resume("focus");
  assert.equal(store.paused.value, false);
  vi.advanceTimersByTime(399);
  assert.equal(store.get(id)?.open, true);
  vi.advanceTimersByTime(1);
  assert.equal(store.get(id)?.open, false);

  store.toast("Stopped");
  store.stop();
  vi.advanceTimersByTime(10_000);
  assert.equal(store.get("toast-2")?.open, true);
});

test("an explicit id replaces the existing toast and restarts its timer", () => {
  const store = createToastStore({ duration: 1000 });
  store.start();
  store.toast({ id: "sync", title: "Syncing", type: "loading" });
  assert.equal(store.get("sync")?.duration, Number.POSITIVE_INFINITY);
  vi.advanceTimersByTime(10_000);
  assert.equal(store.get("sync")?.open, true);

  store.toast({ id: "sync", title: "Synced", type: "success" });
  assert.equal(store.toasts.value.length, 1);
  assert.equal(store.get("sync")?.revision, 1);
  assert.equal(store.get("sync")?.duration, 1000);
  assert.equal(store.update("sync", { description: "All caught up" }), true);
  assert.equal(store.get("sync")?.title, "Synced");
  assert.equal(store.get("sync")?.description, "All caught up");
  assert.equal(store.update("missing", { title: "No" }), false);
  vi.advanceTimersByTime(1000);
  assert.equal(store.get("sync")?.open, false);
});

test("promise toasts move from loading to success or error and return the original promise", async () => {
  const store = createToastStore<{ readonly retry: boolean }>();
  const resolved = Promise.resolve(42);
  const returned = store.promise(resolved, {
    id: "save",
    loading: "Saving",
    success: (value) => ({ title: `Saved ${value}`, data: { retry: false } }),
    error: "Failed",
  });

  assert.equal(returned, resolved);
  assert.equal(store.get("save")?.type, "loading");
  assert.equal(await returned, 42);
  await Promise.resolve();
  assert.equal(store.get("save")?.type, "success");
  assert.equal(store.get("save")?.title, "Saved 42");
  assert.deepEqual(store.get("save")?.data, { retry: false });

  const failure = new Error("offline");
  const rejected = Promise.reject(failure);
  void store.promise(rejected, {
    id: "load",
    loading: { title: "Loading" },
    success: "Loaded",
    error: (reason) => (reason instanceof Error ? reason.message : "unknown"),
  });
  await assert.rejects(rejected, failure);
  await Promise.resolve();
  assert.equal(store.get("load")?.type, "error");
  assert.equal(store.get("load")?.title, "offline");
});

test("dismiss without an id closes every toast and clear empties the store", () => {
  const store = createToastStore();
  store.toast("One");
  store.toast("Two");
  const closed: ToastRecord[] = [];
  store.toast({ title: "Three", onDismiss: (toast) => closed.push(toast) });

  assert.equal(store.dismiss(), true);
  assert.equal(
    store.toasts.value.every((toast) => !toast.open),
    true,
  );
  assert.equal(closed.length, 1);
  assert.equal(store.dismiss(), false);
  store.clear();
  assert.equal(store.toasts.value.length, 0);
});

test("configure and options validate their inputs", () => {
  assert.throws(() => createToastStore({ limit: 0 }), /VIZE_UI_TOAST_OPTION/);
  assert.throws(() => createToastStore({ duration: -1 }), /VIZE_UI_TOAST_OPTION/);
  const store = createToastStore();
  assert.throws(
    () => store.toast({ action: { label: "Undo", altText: " " } }),
    /VIZE_UI_TOAST_OPTION/,
  );
  store.configure({ limit: 5, duration: 200 });
  assert.equal(store.limit.value, 5);
  assert.equal(store.duration.value, 200);
});

test("hotkey helpers format labels and match modifiers plus code or key", () => {
  assert.equal(formatToastHotkey(["altKey", "KeyT"]), "Alt+T");
  assert.equal(formatToastHotkey(["F8"]), "F8");
  assert.equal(
    matchesToastHotkey(new KeyboardEvent("keydown", { altKey: true, code: "KeyT" }), [
      "altKey",
      "KeyT",
    ]),
    true,
  );
  assert.equal(
    matchesToastHotkey(new KeyboardEvent("keydown", { code: "KeyT" }), ["altKey", "KeyT"]),
    false,
  );
  assert.equal(matchesToastHotkey(new KeyboardEvent("keydown", { key: "F8" }), []), false);
});
