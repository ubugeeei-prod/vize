import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { useBattery } from "./battery.ts";
import type { BatteryHost, BatteryManagerLike } from "./battery.ts";

class FakeBattery extends EventTarget implements BatteryManagerLike {
  charging = false;
  chargingTime = Number.POSITIVE_INFINITY;
  dischargingTime = 3600;
  level = 0.5;
}

const settle = async (): Promise<void> => {
  await Promise.resolve();
  await nextTick();
};

void test("exposes a full battery on the server", () => {
  const battery = useBattery({ host: () => undefined });
  assert.deepEqual(
    [battery.isSupported.value, battery.level.value, battery.charging.value, battery.isReady.value],
    [false, 1, true, false],
  );
});

void test("reads the manager, follows events, and detaches with the scope", async () => {
  const manager = new FakeBattery();
  const scope = effectScope();
  const battery = scope.run(() => useBattery({ host: { getBattery: async () => manager } }));
  assert.ok(battery);
  assert.equal(battery.isSupported.value, true);
  await settle();
  assert.deepEqual(
    [battery.isReady.value, battery.level.value, battery.charging.value],
    [true, 0.5, false],
  );

  manager.level = 0.4;
  manager.dispatchEvent(new Event("levelchange"));
  assert.equal(battery.level.value, 0.4);

  scope.stop();
  manager.level = 0.1;
  manager.dispatchEvent(new Event("levelchange"));
  assert.equal(battery.level.value, 0.4);
});

void test("ignores late resolutions from replaced hosts and records rejections", async () => {
  let resolveFirst: (value: BatteryManagerLike) => void = () => undefined;
  const first: BatteryHost = {
    getBattery: () => new Promise((resolve) => (resolveFirst = resolve)),
  };
  const failing: BatteryHost = { getBattery: () => Promise.reject(new Error("policy")) };
  const host = ref<BatteryHost>(first);
  const scope = effectScope();
  const battery = scope.run(() => useBattery({ host }));
  host.value = failing;
  await nextTick();
  const late = new FakeBattery();
  late.level = 0.01;
  resolveFirst(late);
  await settle();
  assert.equal(battery?.level.value, 1);
  assert.ok(battery?.error.value instanceof Error);
  scope.stop();
});
