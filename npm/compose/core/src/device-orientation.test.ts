import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { requestMotionPermission, useDeviceOrientation } from "./device-orientation.ts";
import type { DeviceOrientationHost, MotionPermissionConstructor } from "./device-orientation.ts";
import { eventWith } from "./testing/fake-dom.ts";

class OrientationWindow extends EventTarget implements DeviceOrientationHost {
  DeviceOrientationEvent: MotionPermissionConstructor = { prototype: {} };
}

void test("server renders expose null angles", () => {
  const orientation = useDeviceOrientation({ host: () => undefined });
  assert.deepEqual(
    [orientation.isSupported.value, orientation.alpha.value, orientation.permission.value],
    [false, null, "prompt"],
  );
});

void test("reads angles from the selected event and stops with the scope", () => {
  const host = new OrientationWindow();
  const scope = effectScope();
  const orientation = scope.run(() => useDeviceOrientation({ host, absolute: true }));
  assert.ok(orientation);
  assert.equal(orientation.permission.value, "granted");

  host.dispatchEvent(
    eventWith("deviceorientation", { alpha: 9, beta: 9, gamma: 9, absolute: false }),
  );
  assert.equal(orientation.alpha.value, null);
  host.dispatchEvent(
    eventWith("deviceorientationabsolute", { alpha: 10, beta: 20, gamma: 30, absolute: true }),
  );
  assert.deepEqual(
    [
      orientation.alpha.value,
      orientation.beta.value,
      orientation.gamma.value,
      orientation.isAbsolute.value,
    ],
    [10, 20, 30, true],
  );
  scope.stop();
  host.dispatchEvent(
    eventWith("deviceorientationabsolute", { alpha: 1, beta: 1, gamma: 1, absolute: true }),
  );
  assert.equal(orientation.alpha.value, 10);
});

void test("normalizes permission prompts", async () => {
  const host = new OrientationWindow();
  host.DeviceOrientationEvent = { prototype: {}, requestPermission: async () => "granted" };
  const orientation = useDeviceOrientation({ host });
  assert.equal(orientation.permission.value, "prompt");
  assert.equal(await orientation.requestPermission(), "granted");
  assert.equal(orientation.permission.value, "granted");

  assert.equal(await requestMotionPermission(undefined), "denied");
  assert.equal(
    await requestMotionPermission({ prototype: {}, requestPermission: async () => "maybe" }),
    "prompt",
  );
  assert.equal(
    await requestMotionPermission({
      prototype: {},
      requestPermission: () => Promise.reject(new Error("gesture")),
    }),
    "denied",
  );
});
