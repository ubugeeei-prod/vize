import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useUSB } from "./use-usb.ts";
import type {
  USBControlTransferSetup,
  USBDeviceFilter,
  USBDeviceLike,
  USBHost,
  USBInTransferResultLike,
  USBOutTransferResultLike,
} from "./use-usb.ts";

class FakeDevice implements USBDeviceLike {
  readonly vendorId = 0x2341;
  readonly productId = 0x0043;
  readonly productName: string | undefined = "Board";
  readonly serialNumber: string | undefined = undefined;
  opened = false;
  calls: string[] = [];
  fail: unknown = undefined;

  private async step(name: string): Promise<void> {
    await Promise.resolve();
    if (this.fail !== undefined) {
      const failure = this.fail;
      this.fail = undefined;
      throw failure;
    }
    this.calls.push(name);
  }
  async open(): Promise<void> {
    await this.step("open");
    this.opened = true;
  }
  async close(): Promise<void> {
    await this.step("close");
    this.opened = false;
  }
  selectConfiguration(value: number): Promise<void> {
    return this.step(`config ${value}`);
  }
  claimInterface(number: number): Promise<void> {
    return this.step(`claim ${number}`);
  }
  releaseInterface(number: number): Promise<void> {
    return this.step(`release ${number}`);
  }
  async transferIn(endpoint: number, length: number): Promise<USBInTransferResultLike> {
    await this.step(`in ${endpoint} ${length}`);
    return { status: "ok", data: new DataView(Uint8Array.of(1, 2).buffer) };
  }
  async transferOut(endpoint: number, data: BufferSource): Promise<USBOutTransferResultLike> {
    await this.step(`out ${endpoint}`);
    return { status: "ok", bytesWritten: data.byteLength };
  }
  async controlTransferIn(
    setup: USBControlTransferSetup,
    length: number,
  ): Promise<USBInTransferResultLike> {
    await this.step(`control-in ${setup.request} ${length}`);
    return { status: "stall" };
  }
  async controlTransferOut(
    setup: USBControlTransferSetup,
    data?: BufferSource,
  ): Promise<USBOutTransferResultLike> {
    await this.step(`control-out ${setup.request}`);
    return { status: "ok", bytesWritten: data?.byteLength ?? 0 };
  }
}

class FakeUSB extends EventTarget implements USBHost {
  granted: USBDeviceLike[] = [];
  requests: USBDeviceFilter[][] = [];
  deny: unknown = undefined;
  listeners = 0;

  getDevices(): Promise<USBDeviceLike[]> {
    return Promise.resolve([...this.granted]);
  }
  requestDevice(options: { readonly filters: USBDeviceFilter[] }): Promise<USBDeviceLike> {
    this.requests.push(options.filters);
    if (this.deny !== undefined) return Promise.reject(this.deny);
    const device = new FakeDevice();
    this.granted.push(device);
    return Promise.resolve(device);
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

const setup: USBControlTransferSetup = {
  requestType: "vendor",
  recipient: "device",
  request: 7,
  value: 0,
  index: 0,
};

async function flush(): Promise<void> {
  for (let index = 0; index < 10; index += 1) await Promise.resolve();
}

void test("requests devices and follows connection events", async () => {
  const host = new FakeUSB();
  const usb = useUSB({ host });
  assert.equal(usb.supported.value, true);
  const device = await usb.requestDevice([{ vendorId: 0x2341 }]);
  assert.ok(device);
  assert.deepEqual(host.requests, [[{ vendorId: 0x2341 }]]);
  assert.deepEqual(usb.devices.value, [
    {
      device,
      vendorId: 0x2341,
      productId: 0x0043,
      productName: "Board",
      serialNumber: "",
      opened: false,
    },
  ]);

  host.granted = [];
  host.dispatchEvent(new Event("disconnect"));
  await flush();
  assert.equal(usb.devices.value.length, 0);
  host.granted = [new FakeDevice()];
  host.dispatchEvent(new Event("connect"));
  await flush();
  assert.equal(usb.devices.value.length, 1);
});

void test("wraps the device session and transfers", async () => {
  const host = new FakeUSB();
  const device = new FakeDevice();
  host.granted = [device];
  const usb = useUSB({ host });
  await flush();

  assert.equal(await usb.open(device), true);
  assert.equal(usb.devices.value[0]?.opened, true);
  assert.equal(await usb.selectConfiguration(device, 1), true);
  assert.equal(await usb.claimInterface(device, 0), true);
  const input = await usb.transferIn(device, 1, 64);
  assert.equal(input?.data?.getUint8(1), 2);
  assert.deepEqual(await usb.transferOut(device, 2, Uint8Array.of(1, 2, 3)), {
    status: "ok",
    bytesWritten: 3,
  });
  assert.deepEqual(await usb.controlTransferIn(device, setup, 8), { status: "stall" });
  assert.deepEqual(await usb.controlTransferOut(device, setup), { status: "ok", bytesWritten: 0 });
  assert.equal(await usb.releaseInterface(device, 0), true);
  assert.equal(await usb.close(device), true);
  assert.equal(usb.devices.value[0]?.opened, false);
  assert.deepEqual(device.calls, [
    "open",
    "config 1",
    "claim 0",
    "in 1 64",
    "out 2",
    "control-in 7 8",
    "control-out 7",
    "release 0",
    "close",
  ]);
});

void test("records failures without throwing", async () => {
  const host = new FakeUSB();
  const usb = useUSB({ host });
  const cancelled = new Error("NotFoundError");
  host.deny = cancelled;
  assert.equal(await usb.requestDevice(), undefined);
  assert.equal(usb.error.value, cancelled);

  const device = new FakeDevice();
  const stall = new Error("NetworkError");
  device.fail = stall;
  assert.equal(await usb.claimInterface(device, 0), false);
  assert.equal(usb.error.value, stall);
  device.fail = new Error("transfer");
  assert.equal(await usb.transferIn(device, 1, 8), undefined);
  assert.equal(await usb.transferIn(device, 1, 8).then((result) => result?.status), "ok");
  assert.equal(usb.error.value, undefined);

  const missing = useUSB({ host: null });
  assert.equal(missing.supported.value, false);
  assert.equal(await missing.requestDevice(), undefined);
});

void test("closes owned sessions and listeners with the scope", async () => {
  const host = new FakeUSB();
  const device = new FakeDevice();
  const scope = effectScope();
  const usb = scope.run(() => useUSB({ host }));
  assert.ok(usb);
  await usb.open(device);
  assert.equal(host.listeners, 2);

  scope.stop();
  await flush();
  assert.equal(host.listeners, 0);
  assert.equal(device.opened, false);
});

void test("server rendering lists nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const usb = useUSB();
    return { supported: usb.supported, devices: usb.devices };
  });
  assert.equal(state, '{"supported":false,"devices":[]}');
});
