import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";
import { useClipboard } from "./use-clipboard.ts";
import type { ClipboardHost, ClipboardItemLike, ClipboardLike } from "./use-clipboard.ts";
import type { PermissionDescriptorLike, PermissionStatusLike } from "./use-permission.ts";

class FakeClipboard implements ClipboardLike {
  content = "";
  items: readonly ClipboardItemLike[] = [];
  rejectWith: unknown;

  readText(): Promise<string> {
    return this.rejectWith === undefined
      ? Promise.resolve(this.content)
      : Promise.reject(this.rejectWith);
  }

  writeText(text: string): Promise<void> {
    if (this.rejectWith !== undefined) return Promise.reject(this.rejectWith);
    this.content = text;
    return Promise.resolve();
  }

  read(): Promise<readonly ClipboardItemLike[]> {
    return Promise.resolve(this.items);
  }

  write(items: readonly ClipboardItemLike[]): Promise<void> {
    this.items = items;
    return Promise.resolve();
  }
}

class ManualScheduler implements TimeoutScheduler {
  readonly timers = new Map<number, () => void>();
  next = 0;

  setTimeout(callback: () => void): unknown {
    this.next += 1;
    this.timers.set(this.next, callback);
    return this.next;
  }

  clearTimeout(handle: unknown): void {
    if (typeof handle === "number") this.timers.delete(handle);
  }

  runAll(): void {
    const callbacks = [...this.timers.values()];
    this.timers.clear();
    for (const callback of callbacks) callback();
  }
}

function notAllowed(): Error {
  const error = new Error("denied");
  error.name = "NotAllowedError";
  return error;
}

void test("copies through the async API and resets the copied flag", async () => {
  const clipboard = new FakeClipboard();
  const scheduler = new ManualScheduler();
  const controls = useClipboard({ host: { clipboard }, scheduler });

  assert.equal(controls.supported.value, true);
  const result = await controls.copy("hello");
  assert.deepEqual(result, { status: "success", value: "hello", method: "clipboard" });
  assert.equal(clipboard.content, "hello");
  assert.equal(controls.text.value, "hello");
  assert.equal(controls.copied.value, true);

  scheduler.runAll();
  assert.equal(controls.copied.value, false);
});

void test("falls back to the legacy copy when the async API refuses", async () => {
  const clipboard = new FakeClipboard();
  clipboard.rejectWith = notAllowed();
  const copiedLegacy: string[] = [];
  const host: ClipboardHost = {
    clipboard,
    legacyCopy: (text) => {
      copiedLegacy.push(text);
      return true;
    },
  };
  const controls = useClipboard({ host, scheduler: new ManualScheduler() });

  const result = await controls.copy("x");
  assert.deepEqual(result, { status: "success", value: "x", method: "legacy" });
  assert.deepEqual(copiedLegacy, ["x"]);
});

void test("reports permission denial when the legacy fallback is disabled", async () => {
  const clipboard = new FakeClipboard();
  const denial = notAllowed();
  clipboard.rejectWith = denial;
  const controls = useClipboard({ host: { clipboard }, legacy: false });

  const result = await controls.copy("x");
  assert.deepEqual(result, { status: "permission-denied", error: denial });
  assert.equal(controls.error.value?.status, "permission-denied");
  assert.equal(controls.copied.value, false);
});

void test("uses only the legacy copy when the async API is missing", async () => {
  const controls = useClipboard({
    host: { legacyCopy: () => false },
    scheduler: new ManualScheduler(),
  });
  assert.equal(controls.supported.value, true);
  assert.deepEqual(await controls.copy("x"), { status: "failed", error: undefined });

  const none = useClipboard({ host: {} });
  assert.equal(none.supported.value, false);
  assert.equal((await none.copy("x")).status, "unsupported");
  assert.equal((await none.readText()).status, "unsupported");
});

void test("reads text and rich items", async () => {
  const clipboard = new FakeClipboard();
  clipboard.content = "from clipboard";
  const controls = useClipboard({ host: { clipboard } });

  assert.deepEqual(await controls.readText(), {
    status: "success",
    value: "from clipboard",
    method: "clipboard",
  });
  assert.equal(controls.text.value, "from clipboard");

  const item: ClipboardItemLike = {
    types: ["text/plain"],
    getType: () => Promise.resolve(new Blob(["x"])),
  };
  assert.equal((await controls.writeItems([item])).status, "success");
  const read = await controls.readItems();
  assert.equal(read.status, "success");
  assert.deepEqual(read.status === "success" ? read.value : [], [item]);
});

void test("exposes clipboard permission states", async () => {
  const statuses: Record<string, string> = {
    "clipboard-read": "prompt",
    "clipboard-write": "granted",
  };
  const permissions = {
    query: (descriptor: PermissionDescriptorLike): Promise<PermissionStatusLike> =>
      Promise.resolve(
        Object.assign(new EventTarget(), { state: statuses[descriptor.name] ?? "denied" }),
      ),
  };
  const controls = useClipboard({ host: { clipboard: new FakeClipboard(), permissions } });
  for (let index = 0; index < 3; index += 1) await Promise.resolve();

  assert.equal(controls.readPermission.value, "prompt");
  assert.equal(controls.writePermission.value, "granted");
});

void test("follows copy events and releases listeners with the scope", async () => {
  const clipboard = new FakeClipboard();
  const events = new EventTarget();
  const scope = effectScope();
  const controls = scope.run(() => useClipboard({ host: { clipboard, events }, listen: true }));
  assert.ok(controls);

  clipboard.content = "copied elsewhere";
  events.dispatchEvent(new Event("copy"));
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(controls.text.value, "copied elsewhere");

  scope.stop();
  clipboard.content = "ignored";
  events.dispatchEvent(new Event("cut"));
  await Promise.resolve();
  assert.equal(controls.text.value, "copied elsewhere");
});

void test("rejects invalid feedback durations", () => {
  assert.throws(
    () => useClipboard({ copiedDuringMs: -1 }),
    /VIZE_COMPOSE_CLIPBOARD_INVALID_DURATION/,
  );
});

void test("server rendering reads nothing and queries no permission", async () => {
  const state = await renderComposableOnServer(() => {
    const clipboard = useClipboard({ listen: true });
    return {
      supported: clipboard.supported,
      text: clipboard.text,
      copied: clipboard.copied,
      read: clipboard.readPermission,
    };
  });
  assert.equal(state, '{"supported":false,"text":"","copied":false,"read":"unknown"}');
});
