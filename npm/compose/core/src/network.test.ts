import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { useNetwork, useOnline } from "./network.ts";
import type { NetworkHost, NetworkInformationLike } from "./network.ts";

class FakeConnection extends EventTarget implements NetworkInformationLike {
  downlink = 10;
  effectiveType = "4g";
  rtt = 50;
  saveData = false;
  type = "wifi";
}

class FakeNetworkWindow extends EventTarget implements NetworkHost {
  readonly navigator: { onLine: boolean; connection?: FakeConnection } = { onLine: true };
}

void test("server renders use the configured connectivity", () => {
  const network = useNetwork({ host: () => undefined, ssrOnline: false });
  assert.equal(network.isOnline.value, false);
  assert.equal(network.isSupported.value, false);
  assert.equal(network.effectiveType.value, null);
  assert.equal(useOnline({ host: () => undefined }).value, true);
});

void test("tracks online state and connection quality with closed unions", () => {
  const host = new FakeNetworkWindow();
  const connection = new FakeConnection();
  host.navigator.connection = connection;
  const scope = effectScope();
  const network = scope.run(() => useNetwork({ host }));
  assert.ok(network);
  assert.deepEqual(
    [network.isSupported.value, network.effectiveType.value, network.type.value, network.rtt.value],
    [true, "4g", "wifi", 50],
  );

  host.navigator.onLine = false;
  host.dispatchEvent(new Event("offline"));
  assert.equal(network.isOnline.value, false);
  assert.equal(typeof network.offlineAt.value, "number");

  connection.effectiveType = "6g";
  connection.type = "satellite";
  connection.saveData = true;
  connection.dispatchEvent(new Event("change"));
  assert.deepEqual(
    [network.effectiveType.value, network.type.value, network.saveData.value],
    [null, null, true],
  );

  scope.stop();
  host.navigator.onLine = true;
  host.dispatchEvent(new Event("online"));
  assert.equal(network.isOnline.value, false);
});

void test("online-only hosts are unsupported for quality but still track connectivity", () => {
  const host = new FakeNetworkWindow();
  const online = useOnline({ host });
  host.navigator.onLine = false;
  host.dispatchEvent(new Event("offline"));
  assert.equal(online.value, false);
});
