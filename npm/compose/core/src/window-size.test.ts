import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { useWindowSize } from "./window-size.ts";
import type { WindowSizeHost } from "./window-size.ts";
import { FakeWindow } from "./testing/fake-dom.ts";

void test("exposes deterministic server values without a window", () => {
  const size = useWindowSize({ host: () => undefined, initialWidth: 320, initialHeight: 480 });
  assert.equal(size.isSupported.value, false);
  assert.deepEqual([size.width.value, size.height.value], [320, 480]);
});

void test("tracks inner size, orientation changes, and cleans up with the scope", () => {
  const host = new FakeWindow();
  const scope = effectScope();
  const size = scope.run(() => useWindowSize({ host }));
  assert.ok(size);
  assert.deepEqual([size.width.value, size.height.value], [1024, 768]);

  host.innerWidth = 500;
  host.dispatchEvent(new Event("resize"));
  assert.equal(size.width.value, 500);
  host.innerHeight = 300;
  host.dispatchEvent(new Event("orientationchange"));
  assert.equal(size.height.value, 300);

  scope.stop();
  host.innerWidth = 1;
  host.dispatchEvent(new Event("resize"));
  assert.equal(size.width.value, 500);
});

void test("supports outer and scrollbar-free measurements and host swaps", async () => {
  const host = new FakeWindow();
  host.document.documentElement.clientWidth = 1000;
  host.document.documentElement.clientHeight = 700;
  const current = ref<WindowSizeHost | null>(host);
  const scope = effectScope();
  const inner = scope.run(() => useWindowSize({ host: current, includeScrollbar: false }));
  const outer = scope.run(() => useWindowSize({ host, type: "outer", listenOrientation: false }));
  assert.deepEqual([inner?.width.value, inner?.height.value], [1000, 700]);
  assert.deepEqual([outer?.width.value, outer?.height.value], [1040, 800]);

  current.value = null;
  await nextTick();
  assert.equal(inner?.isSupported.value, false);
  assert.equal(inner?.width.value, 0);
  scope.stop();
});
