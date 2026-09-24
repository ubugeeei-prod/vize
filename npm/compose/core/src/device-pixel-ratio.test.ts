import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { useDevicePixelRatio } from "./device-pixel-ratio.ts";
import type { DevicePixelRatioHost } from "./device-pixel-ratio.ts";

class RatioWindow implements DevicePixelRatioHost {
  devicePixelRatio = 1;
  readonly lists: EventTarget[] = [];
  readonly queries: string[] = [];

  matchMedia(query: string): MediaQueryList {
    const list = new EventTarget();
    this.lists.push(list);
    this.queries.push(query);
    return list as unknown as MediaQueryList;
  }

  change(ratio: number): void {
    this.devicePixelRatio = ratio;
    this.lists.at(-1)?.dispatchEvent(new Event("change"));
  }
}

void test("server renders use the configured ratio", () => {
  const ratio = useDevicePixelRatio({ host: () => undefined, ssrPixelRatio: 2 });
  assert.deepEqual([ratio.pixelRatio.value, ratio.isSupported.value], [2, false]);
});

void test("re-subscribes with the new ratio after each change", () => {
  const host = new RatioWindow();
  const scope = effectScope();
  const ratio = scope.run(() => useDevicePixelRatio({ host }));
  assert.equal(ratio?.pixelRatio.value, 1);
  host.change(2);
  assert.equal(ratio?.pixelRatio.value, 2);
  host.change(1.5);
  assert.equal(ratio?.pixelRatio.value, 1.5);
  assert.deepEqual(host.queries, [
    "(resolution: 1dppx)",
    "(resolution: 2dppx)",
    "(resolution: 1.5dppx)",
  ]);

  scope.stop();
  host.change(3);
  assert.equal(ratio?.pixelRatio.value, 1.5);
});
