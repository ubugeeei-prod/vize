import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { useDeviceMotion } from "./device-motion.ts";
import type { DeviceMotionHost } from "./device-motion.ts";
import type { MotionPermissionConstructor } from "./device-orientation.ts";
import { eventWith } from "./testing/fake-dom.ts";

class MotionWindow extends EventTarget implements DeviceMotionHost {
  DeviceMotionEvent: MotionPermissionConstructor = {
    prototype: {},
    requestPermission: async () => "denied",
  };
}

void test("copies readings into snapshots and stops with the scope", async () => {
  const host = new MotionWindow();
  const scope = effectScope();
  const motion = scope.run(() => useDeviceMotion({ host }));
  assert.ok(motion);
  assert.deepEqual([motion.isSupported.value, motion.acceleration.value], [true, null]);

  const source = { x: 1, y: 2, z: 3 };
  host.dispatchEvent(
    eventWith("devicemotion", {
      acceleration: source,
      accelerationIncludingGravity: { x: 1, y: 2, z: 12 },
      rotationRate: { alpha: 4, beta: 5, gamma: 6 },
      interval: 16,
    }),
  );
  assert.deepEqual(motion.acceleration.value, source);
  assert.notEqual(motion.acceleration.value, source);
  assert.deepEqual(motion.rotationRate.value, { alpha: 4, beta: 5, gamma: 6 });
  assert.equal(motion.interval.value, 16);
  assert.equal(await motion.requestPermission(), "denied");

  scope.stop();
  host.dispatchEvent(
    eventWith("devicemotion", {
      acceleration: null,
      accelerationIncludingGravity: null,
      rotationRate: null,
      interval: 32,
    }),
  );
  assert.equal(motion.interval.value, 16);
});

void test("server renders expose null readings", () => {
  const motion = useDeviceMotion({ host: () => null });
  assert.deepEqual([motion.isSupported.value, motion.rotationRate.value], [false, null]);
});
