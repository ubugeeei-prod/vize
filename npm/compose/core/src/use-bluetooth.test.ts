import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, shallowRef } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useBluetooth } from "./use-bluetooth.ts";
import type {
  BluetoothCharacteristicLike,
  BluetoothDeviceLike,
  BluetoothGATTServerLike,
  BluetoothHost,
  BluetoothRequestOptions,
  BluetoothServiceLike,
  BluetoothUUIDLike,
} from "./use-bluetooth.ts";

class FakeCharacteristic extends EventTarget implements BluetoothCharacteristicLike {
  value: DataView | null = null;
  writes: { bytes: number[]; response: boolean }[] = [];
  notifying = false;
  listeners = 0;

  readValue(): Promise<DataView> {
    this.value = new DataView(Uint8Array.of(0x55).buffer);
    return Promise.resolve(this.value);
  }
  writeValueWithResponse(value: BufferSource): Promise<void> {
    this.writes.push({ bytes: bytes(value), response: true });
    return Promise.resolve();
  }
  writeValueWithoutResponse(value: BufferSource): Promise<void> {
    this.writes.push({ bytes: bytes(value), response: false });
    return Promise.resolve();
  }
  startNotifications(): Promise<unknown> {
    this.notifying = true;
    return Promise.resolve(this);
  }
  stopNotifications(): Promise<unknown> {
    this.notifying = false;
    return Promise.resolve(this);
  }
  emit(value: number): void {
    this.value = new DataView(Uint8Array.of(value).buffer);
    this.dispatchEvent(new Event("characteristicvaluechanged"));
  }
  override addEventListener(type: string, listener: () => void): void {
    this.listeners += 1;
    super.addEventListener(type, listener);
  }
  override removeEventListener(type: string, listener: () => void): void {
    this.listeners -= 1;
    super.removeEventListener(type, listener);
  }
}

function bytes(value: BufferSource): number[] {
  return [
    ...(ArrayBuffer.isView(value)
      ? new Uint8Array(value.buffer, value.byteOffset, value.byteLength)
      : new Uint8Array(value)),
  ];
}

class FakeServer implements BluetoothGATTServerLike {
  connected = false;
  failConnect: unknown = undefined;
  readonly characteristic = new FakeCharacteristic();
  lookups: string[] = [];
  device: FakeDevice | undefined;

  connect(): Promise<unknown> {
    if (this.failConnect !== undefined) return Promise.reject(this.failConnect);
    this.connected = true;
    return Promise.resolve(this);
  }
  disconnect(): void {
    if (!this.connected) return;
    this.connected = false;
    this.device?.dispatchEvent(new Event("gattserverdisconnected"));
  }
  getPrimaryService(service: BluetoothUUIDLike): Promise<BluetoothServiceLike> {
    return Promise.resolve({
      getCharacteristic: (characteristic: BluetoothUUIDLike) => {
        this.lookups.push(`${service}/${characteristic}`);
        return characteristic === "missing"
          ? Promise.reject(new Error("NotFoundError"))
          : Promise.resolve(this.characteristic);
      },
    });
  }
}

class FakeDevice extends EventTarget implements BluetoothDeviceLike {
  readonly id = "device-1";
  readonly name = "Heart Rate";
  readonly gatt = new FakeServer();
  constructor() {
    super();
    this.gatt.device = this;
  }
}

class AvailabilityEvent extends Event {
  readonly value: boolean;
  constructor(value: boolean) {
    super("availabilitychanged");
    this.value = value;
  }
}

class FakeBluetooth extends EventTarget implements BluetoothHost {
  availability = true;
  requests: BluetoothRequestOptions[] = [];
  deny: unknown = undefined;
  readonly device = new FakeDevice();

  getAvailability(): Promise<boolean> {
    return Promise.resolve(this.availability);
  }
  requestDevice(options: BluetoothRequestOptions): Promise<BluetoothDeviceLike> {
    this.requests.push(options);
    if (this.deny !== undefined) return Promise.reject(this.deny);
    return Promise.resolve(this.device);
  }
}

async function flush(): Promise<void> {
  for (let index = 0; index < 10; index += 1) await Promise.resolve();
}

void test("tracks adapter availability", async () => {
  const host = new FakeBluetooth();
  const current = shallowRef<BluetoothHost | undefined>(host);
  const ble = useBluetooth({ host: current });
  assert.equal(ble.supported.value, true);
  await flush();
  assert.equal(ble.available.value, true);
  host.dispatchEvent(new AvailabilityEvent(false));
  assert.equal(ble.available.value, false);
  current.value = undefined;
  assert.equal(ble.supported.value, false);
  host.dispatchEvent(new AvailabilityEvent(true));
  assert.equal(ble.available.value, false);
});

void test("requests, connects, and follows gattserverdisconnected", async () => {
  const host = new FakeBluetooth();
  const ble = useBluetooth({ host, requestOptions: { filters: [{ services: ["heart_rate"] }] } });
  const device = await ble.requestDevice();
  assert.equal(device, host.device);
  assert.equal(ble.device.value, host.device);
  assert.deepEqual(host.requests, [{ filters: [{ services: ["heart_rate"] }] }]);
  await ble.requestDevice({ acceptAllDevices: true, optionalServices: [0x180d] });
  assert.deepEqual(host.requests[1], { acceptAllDevices: true, optionalServices: [0x180d] });

  assert.equal(await ble.connect(), true);
  assert.equal(ble.connected.value, true);
  host.device.gatt.disconnect();
  assert.equal(ble.connected.value, false);
});

void test("reads, writes, and subscribes to characteristics", async () => {
  const host = new FakeBluetooth();
  const ble = useBluetooth({ host });
  await ble.requestDevice();
  await ble.connect();
  const characteristic = host.device.gatt.characteristic;

  const value = await ble.read("heart_rate", "body_sensor_location");
  assert.equal(value?.getUint8(0), 0x55);
  assert.equal(await ble.write("svc", "chr", Uint8Array.of(1)), true);
  assert.equal(await ble.write("svc", "chr", Uint8Array.of(2), { withoutResponse: true }), true);
  assert.deepEqual(characteristic.writes, [
    { bytes: [1], response: true },
    { bytes: [2], response: false },
  ]);

  const seen: number[] = [];
  const stop = await ble.notify("heart_rate", "heart_rate_measurement", (next) =>
    seen.push(next.getUint8(0)),
  );
  assert.ok(stop);
  assert.equal(characteristic.notifying, true);
  characteristic.emit(72);
  await stop();
  characteristic.emit(80);
  assert.deepEqual(seen, [72]);
  assert.equal(characteristic.notifying, false);
  assert.equal(characteristic.listeners, 0);
});

void test("removes characteristic listeners when the device disconnects", async () => {
  const host = new FakeBluetooth();
  const ble = useBluetooth({ host });
  await ble.requestDevice();
  await ble.connect();
  const characteristic = host.device.gatt.characteristic;
  const seen: number[] = [];
  await ble.notify("svc", "chr", (value) => seen.push(value.getUint8(0)));
  assert.equal(characteristic.listeners, 1);

  host.device.gatt.disconnect();
  await flush();
  characteristic.emit(72);
  assert.equal(characteristic.listeners, 0);
  assert.deepEqual(seen, []);
});

void test("records failures without throwing", async () => {
  const host = new FakeBluetooth();
  const ble = useBluetooth({ host });
  assert.equal(await ble.connect(), false, "no device yet");
  const cancelled = new Error("NotFoundError");
  host.deny = cancelled;
  assert.equal(await ble.requestDevice(), undefined);
  assert.equal(ble.error.value, cancelled);
  host.deny = undefined;

  await ble.requestDevice();
  assert.equal(await ble.read("svc", "chr"), undefined);
  assert.match(String(ble.error.value), /BLUETOOTH_NOT_CONNECTED/);

  const refused = new Error("NetworkError");
  host.device.gatt.failConnect = refused;
  assert.equal(await ble.connect(), false);
  assert.equal(ble.error.value, refused);
  host.device.gatt.failConnect = undefined;
  await ble.connect();
  assert.equal(await ble.write("svc", "missing", Uint8Array.of(0)), false);
  assert.equal(await ble.notify("svc", "missing", () => undefined), undefined);

  const missing = useBluetooth({ host: null });
  assert.equal(missing.supported.value, false);
  assert.equal(await missing.requestDevice(), undefined);
});

void test("stops notifications and disconnects with the scope", async () => {
  const host = new FakeBluetooth();
  const scope = effectScope();
  const ble = scope.run(() => useBluetooth({ host }));
  assert.ok(ble);
  await ble.requestDevice();
  await ble.connect();
  await ble.notify("svc", "chr", () => undefined);
  const characteristic = host.device.gatt.characteristic;
  assert.equal(characteristic.notifying, true);

  scope.stop();
  await flush();
  assert.equal(characteristic.notifying, false);
  assert.equal(characteristic.listeners, 0);
  assert.equal(host.device.gatt.connected, false);
  assert.equal(ble.connected.value, false);
});

void test("server rendering queries nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const ble = useBluetooth();
    return { supported: ble.supported, available: ble.available, connected: ble.connected };
  });
  assert.equal(state, '{"supported":false,"available":false,"connected":false}');
});
