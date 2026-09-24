import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useHID } from "./use-hid.ts";
import type { HIDDeviceFilter, HIDDeviceLike, HIDHost, HIDInputReport } from "./use-hid.ts";

class FakeDevice extends EventTarget implements HIDDeviceLike {
  opened = false;
  readonly vendorId = 0x054c;
  readonly productId = 0x0ce6;
  readonly productName = "Controller";
  sent: { kind: string; reportId: number; bytes: number[] }[] = [];
  fail: unknown = undefined;
  listeners = 0;

  private check(): Promise<void> {
    if (this.fail === undefined) return Promise.resolve();
    const failure = this.fail;
    this.fail = undefined;
    return Promise.reject(failure);
  }
  async open(): Promise<void> {
    await this.check();
    this.opened = true;
  }
  close(): Promise<void> {
    this.opened = false;
    return Promise.resolve();
  }
  async sendReport(reportId: number, data: BufferSource): Promise<void> {
    await this.check();
    this.sent.push({ kind: "output", reportId, bytes: [...toBytes(data)] });
  }
  async sendFeatureReport(reportId: number, data: BufferSource): Promise<void> {
    await this.check();
    this.sent.push({ kind: "feature", reportId, bytes: [...toBytes(data)] });
  }
  async receiveFeatureReport(reportId: number): Promise<DataView> {
    await this.check();
    return new DataView(Uint8Array.of(reportId, 42).buffer);
  }
  override addEventListener(type: string, listener: (event: Event) => void): void {
    this.listeners += 1;
    super.addEventListener(type, listener);
  }
  override removeEventListener(type: string, listener: (event: Event) => void): void {
    this.listeners -= 1;
    super.removeEventListener(type, listener);
  }
}

function toBytes(data: BufferSource): Uint8Array {
  return ArrayBuffer.isView(data)
    ? new Uint8Array(data.buffer, data.byteOffset, data.byteLength)
    : new Uint8Array(data);
}

class InputReportEvent extends Event {
  readonly reportId: number;
  readonly data: DataView;
  constructor(reportId: number, bytes: number[]) {
    super("inputreport");
    this.reportId = reportId;
    this.data = new DataView(Uint8Array.from(bytes).buffer);
  }
}

class FakeHID extends EventTarget implements HIDHost {
  granted: HIDDeviceLike[] = [];
  requests: HIDDeviceFilter[][] = [];
  deny: unknown = undefined;

  getDevices(): Promise<HIDDeviceLike[]> {
    return Promise.resolve([...this.granted]);
  }
  requestDevice(options: { readonly filters: HIDDeviceFilter[] }): Promise<HIDDeviceLike[]> {
    this.requests.push(options.filters);
    if (this.deny !== undefined) return Promise.reject(this.deny);
    const device = new FakeDevice();
    this.granted.push(device);
    return Promise.resolve([device]);
  }
}

async function flush(): Promise<void> {
  for (let index = 0; index < 10; index += 1) await Promise.resolve();
}

void test("requests devices and follows connection events", async () => {
  const host = new FakeHID();
  const hid = useHID({ host });
  assert.equal(hid.supported.value, true);
  const [device] = await hid.requestDevice([{ vendorId: 0x054c }]);
  assert.ok(device);
  assert.deepEqual(host.requests, [[{ vendorId: 0x054c }]]);
  assert.deepEqual(hid.devices.value, [
    { device, vendorId: 0x054c, productId: 0x0ce6, productName: "Controller", opened: false },
  ]);

  host.granted = [];
  host.dispatchEvent(new Event("disconnect"));
  await flush();
  assert.equal(hid.devices.value.length, 0);
  host.granted = [new FakeDevice()];
  host.dispatchEvent(new Event("connect"));
  await flush();
  assert.equal(hid.devices.value.length, 1);
});

void test("opens a device and forwards input reports", async () => {
  const host = new FakeHID();
  const device = new FakeDevice();
  host.granted = [device];
  const reports: HIDInputReport[] = [];
  const hid = useHID({ host, onInputReport: (report) => reports.push(report) });
  await flush();

  assert.equal(await hid.open(device), true);
  assert.equal(hid.devices.value[0]?.opened, true);
  device.dispatchEvent(new InputReportEvent(1, [5, 6]));
  device.dispatchEvent(new Event("inputreport"));
  assert.equal(reports.length, 1);
  assert.equal(reports[0]?.reportId, 1);
  assert.equal(reports[0]?.data.getUint8(1), 6);

  assert.equal(await hid.close(device), true);
  assert.equal(hid.devices.value[0]?.opened, false);
  device.dispatchEvent(new InputReportEvent(1, [7]));
  assert.equal(reports.length, 1);
});

void test("sends and receives reports", async () => {
  const hid = useHID({ host: new FakeHID() });
  const device = new FakeDevice();
  await hid.open(device);
  assert.equal(await hid.sendReport(device, 2, Uint8Array.of(1, 2)), true);
  assert.equal(await hid.sendFeatureReport(device, 3, Uint8Array.of(9).buffer), true);
  assert.deepEqual(device.sent, [
    { kind: "output", reportId: 2, bytes: [1, 2] },
    { kind: "feature", reportId: 3, bytes: [9] },
  ]);
  const feature = await hid.receiveFeatureReport(device, 4);
  assert.equal(feature?.getUint8(1), 42);
});

void test("records failures without throwing", async () => {
  const host = new FakeHID();
  const hid = useHID({ host });
  const denied = new Error("NotAllowedError");
  host.deny = denied;
  assert.deepEqual(await hid.requestDevice(), []);
  assert.equal(hid.error.value, denied);

  const device = new FakeDevice();
  const busy = new Error("NotAllowedError: open");
  device.fail = busy;
  assert.equal(await hid.open(device), false);
  assert.equal(hid.error.value, busy);
  device.fail = new Error("send");
  assert.equal(await hid.sendReport(device, 1, Uint8Array.of(0)), false);
  device.fail = new Error("receive");
  assert.equal(await hid.receiveFeatureReport(device, 1), undefined);

  const missing = useHID({ host: null });
  assert.equal(missing.supported.value, false);
  assert.deepEqual(await missing.requestDevice(), []);
});

void test("closes owned devices with the scope", async () => {
  const host = new FakeHID();
  const device = new FakeDevice();
  const scope = effectScope();
  const hid = scope.run(() => useHID({ host, onInputReport: () => undefined }));
  assert.ok(hid);
  await hid.open(device);
  assert.equal(device.listeners, 1);

  scope.stop();
  await flush();
  assert.equal(device.listeners, 0);
  assert.equal(device.opened, false);
});

void test("server rendering lists nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const hid = useHID();
    return { supported: hid.supported, devices: hid.devices };
  });
  assert.equal(state, '{"supported":false,"devices":[]}');
});
