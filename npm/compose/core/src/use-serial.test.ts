import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useSerial } from "./use-serial.ts";
import type {
  SerialHost,
  SerialOpenOptions,
  SerialPortFilter,
  SerialPortLike,
} from "./use-serial.ts";

class FakePort extends EventTarget implements SerialPortLike {
  readable: ReadableStream<Uint8Array> | null = null;
  writable: WritableStream<Uint8Array> | null = null;
  controller: ReadableStreamDefaultController<Uint8Array> | undefined;
  written: Uint8Array[] = [];
  opened: SerialOpenOptions[] = [];
  closed = 0;
  failOpen: unknown = undefined;
  failCancel: unknown = undefined;

  open(options: SerialOpenOptions): Promise<void> {
    if (this.failOpen !== undefined) return Promise.reject(this.failOpen);
    this.opened.push(options);
    this.readable = new ReadableStream<Uint8Array>({
      start: (controller) => {
        this.controller = controller;
      },
      cancel: () => {
        if (this.failCancel !== undefined) return Promise.reject(this.failCancel);
      },
    });
    this.writable = new WritableStream<Uint8Array>({
      write: (chunk) => {
        this.written.push(chunk);
      },
    });
    return Promise.resolve();
  }

  close(): Promise<void> {
    this.closed += 1;
    this.readable = null;
    this.writable = null;
    return Promise.resolve();
  }
}

class FakeSerial extends EventTarget implements SerialHost {
  granted: SerialPortLike[] = [];
  requests: ({ readonly filters?: readonly SerialPortFilter[] } | undefined)[] = [];
  deny: unknown = undefined;
  listeners = 0;

  getPorts(): Promise<SerialPortLike[]> {
    return Promise.resolve([...this.granted]);
  }

  requestPort(options?: {
    readonly filters?: readonly SerialPortFilter[];
  }): Promise<SerialPortLike> {
    this.requests.push(options);
    if (this.deny !== undefined) return Promise.reject(this.deny);
    const port = new FakePort();
    this.granted.push(port);
    return Promise.resolve(port);
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

async function flush(): Promise<void> {
  for (let index = 0; index < 10; index += 1) await Promise.resolve();
}

void test("requests a port and tracks granted ports", async () => {
  const host = new FakeSerial();
  const serial = useSerial({ host });
  assert.equal(serial.supported.value, true);
  const port = await serial.requestPort([{ usbVendorId: 0x2341 }]);
  assert.ok(port);
  assert.deepEqual(host.requests, [{ filters: [{ usbVendorId: 0x2341 }] }]);
  assert.deepEqual(serial.ports.value, [port]);

  host.granted = [];
  host.dispatchEvent(new Event("disconnect"));
  await flush();
  assert.equal(serial.ports.value.length, 0);
  host.granted = [new FakePort()];
  host.dispatchEvent(new Event("connect"));
  await flush();
  assert.equal(serial.ports.value.length, 1);
});

void test("closes a port even when reader cancellation fails", async () => {
  const serial = useSerial({ host: new FakeSerial() });
  const port = new FakePort();
  const failure = new Error("cancel failed");
  port.failCancel = failure;
  await serial.open(port, { baudRate: 9600 });
  await serial.close();
  assert.equal(port.closed, 1);
  assert.equal(serial.error.value, failure);
  assert.equal(serial.connected.value, false);
});

void test("reads decoded text chunks and writes text or bytes", async () => {
  const host = new FakeSerial();
  const chunks: string[] = [];
  const serial = useSerial({ host, decode: "text", onData: (chunk) => chunks.push(chunk) });
  const port = new FakePort();
  assert.equal(await serial.open(port, { baudRate: 9600 }), true);
  assert.equal(serial.connected.value, true);
  assert.equal(serial.port.value, port);
  assert.deepEqual(port.opened, [{ baudRate: 9600 }]);

  const euro = new TextEncoder().encode("€");
  port.controller?.enqueue(Uint8Array.of(0x68, 0x69, euro[0] ?? 0));
  port.controller?.enqueue(euro.subarray(1));
  await flush();
  assert.deepEqual(chunks, ["hi", "€"]);

  assert.equal(await serial.write("ok"), true);
  assert.equal(await serial.write(Uint8Array.of(1, 2)), true);
  assert.deepEqual(
    port.written.map((chunk) => [...chunk]),
    [
      [0x6f, 0x6b],
      [1, 2],
    ],
  );

  await serial.close();
  assert.equal(serial.connected.value, false);
  assert.equal(port.closed, 1);
  assert.equal(await serial.write("late"), false);
});

void test("delivers raw bytes by default and records read errors", async () => {
  const host = new FakeSerial();
  const chunks: Uint8Array[] = [];
  const serial = useSerial({ host, onData: (chunk) => chunks.push(chunk) });
  const port = new FakePort();
  await serial.open(port, { baudRate: 115200 });
  port.controller?.enqueue(Uint8Array.of(7));
  await flush();
  assert.deepEqual(
    chunks.map((chunk) => [...chunk]),
    [[7]],
  );
  const failure = new Error("BreakError");
  port.controller?.error(failure);
  await flush();
  assert.equal(serial.error.value, failure);
});

void test("closes the session when the port disconnects", async () => {
  const host = new FakeSerial();
  const serial = useSerial({ host });
  const port = new FakePort();
  await serial.open(port, { baudRate: 9600 });
  port.dispatchEvent(new Event("disconnect"));
  await flush();
  assert.equal(serial.connected.value, false);
  assert.equal(port.closed, 1);
});

void test("exposes request and open failures without throwing", async () => {
  const host = new FakeSerial();
  const serial = useSerial({ host });
  const cancelled = new Error("NotFoundError");
  host.deny = cancelled;
  assert.equal(await serial.requestPort(), undefined);
  assert.equal(serial.error.value, cancelled);
  assert.deepEqual(host.requests, [{}]);

  const port = new FakePort();
  const busy = new Error("InvalidStateError");
  port.failOpen = busy;
  assert.equal(await serial.open(port, { baudRate: 9600 }), false);
  assert.equal(serial.error.value, busy);
  assert.equal(serial.connected.value, false);

  const missing = useSerial({ host: null });
  assert.equal(missing.supported.value, false);
  assert.equal(await missing.requestPort(), undefined);
  assert.deepEqual(await missing.getPorts(), []);
  assert.throws(() => useSerial({ decode: "text", encoding: "nope" }), /SERIAL_INVALID_ENCODING/);
});

void test("closes the port and listeners with the scope", async () => {
  const host = new FakeSerial();
  const port = new FakePort();
  const scope = effectScope();
  const serial = scope.run(() => useSerial({ host }));
  assert.ok(serial);
  await serial.open(port, { baudRate: 9600 });
  assert.equal(host.listeners, 2);

  scope.stop();
  await flush();
  assert.equal(host.listeners, 0);
  assert.equal(port.closed, 1);
  assert.equal(serial.connected.value, false);
});

void test("server rendering opens nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const serial = useSerial();
    return { supported: serial.supported, connected: serial.connected, ports: serial.ports };
  });
  assert.equal(state, '{"supported":false,"connected":false,"ports":[]}');
});
