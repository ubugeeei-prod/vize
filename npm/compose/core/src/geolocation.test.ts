import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick } from "vue";

import { useGeolocation } from "./geolocation.ts";
import type { GeolocationControls, GeolocationHost } from "./geolocation.ts";

function reasonOf(geo: GeolocationControls): string {
  const result = geo.capability.value;
  return result.available ? "available" : result.reason;
}

class FakeGeolocation {
  readonly watches = new Map<
    number,
    {
      success: PositionCallback;
      error: PositionErrorCallback | null | undefined;
      options: PositionOptions | undefined;
    }
  >();
  private next = 1;

  watchPosition(
    success: PositionCallback,
    error?: PositionErrorCallback | null,
    options?: PositionOptions,
  ): number {
    const id = this.next++;
    this.watches.set(id, { success, error, options });
    return id;
  }

  clearWatch(id: number): void {
    this.watches.delete(id);
  }

  emit(latitude: number, timestamp: number): void {
    for (const watcher of this.watches.values()) {
      watcher.success({
        timestamp,
        coords: {
          latitude,
          longitude: 2,
          accuracy: 3,
          altitude: null,
          altitudeAccuracy: null,
          heading: null,
          speed: null,
          toJSON: () => ({}),
        },
        toJSON: () => ({}),
      });
    }
  }

  fail(code: number): void {
    for (const watcher of this.watches.values()) {
      watcher.error?.({
        code,
        message: "failed",
        PERMISSION_DENIED: 1,
        POSITION_UNAVAILABLE: 2,
        TIMEOUT: 3,
      });
    }
  }
}

void test("reports unsupported without a host", () => {
  const geo = useGeolocation({ host: () => undefined });
  assert.equal(geo.isSupported.value, false);
  assert.equal(geo.coords.value, null);
  assert.deepEqual(geo.capability.value, {
    status: "unavailable",
    available: false,
    reason: "unsupported",
    details: undefined,
  });
});

void test("watches positions, maps errors to capability reasons, and clears with the scope", () => {
  const geolocation = new FakeGeolocation();
  const host: GeolocationHost = { geolocation };
  const scope = effectScope();
  const geo = scope.run(() => useGeolocation({ host, enableHighAccuracy: true }));
  assert.ok(geo);
  assert.equal(geolocation.watches.size, 1);
  assert.deepEqual([...geolocation.watches.values()][0]?.options, {
    enableHighAccuracy: true,
    maximumAge: 30_000,
    timeout: 27_000,
  });
  assert.equal(reasonOf(geo), "not-ready");

  geolocation.emit(1, 100);
  assert.equal(geo.coords.value?.latitude, 1);
  assert.equal(geo.locatedAt.value, 100);
  assert.equal(reasonOf(geo), "available");

  geolocation.fail(1);
  assert.equal(reasonOf(geo), "permission-denied");
  geolocation.fail(2);
  assert.equal(reasonOf(geo), "unavailable");
  geolocation.emit(5, 200);
  assert.equal(geo.error.value, null);

  scope.stop();
  assert.equal(geolocation.watches.size, 0);
});

void test("pause and resume control the watch", async () => {
  const geolocation = new FakeGeolocation();
  const scope = effectScope();
  const geo = scope.run(() => useGeolocation({ host: { geolocation }, immediate: false }));
  assert.equal(geolocation.watches.size, 0);
  geo?.resume();
  await nextTick();
  assert.equal(geolocation.watches.size, 1);
  geo?.pause();
  await nextTick();
  assert.equal(geolocation.watches.size, 0);
  scope.stop();
});
