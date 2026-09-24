import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useWebMIDI } from "./use-web-midi.ts";
import type {
  MIDIAccessLike,
  MIDIHost,
  MIDIInputMessage,
  MIDIOutputLike,
  MIDIPortLike,
  MIDIRequestFlags,
} from "./use-web-midi.ts";

class FakePort extends EventTarget implements MIDIPortLike {
  state = "connected";
  connection = "closed";
  listeners = 0;
  readonly id: string;
  readonly name: string | null;
  readonly manufacturer: string | null;
  constructor(
    id: string,
    name: string | null = `port ${id}`,
    manufacturer: string | null = "vize",
  ) {
    super();
    this.id = id;
    this.name = name;
    this.manufacturer = manufacturer;
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

class FakeOutput extends FakePort implements MIDIOutputLike {
  sent: { data: number[]; timestamp: number | undefined }[] = [];
  fail = false;
  send(data: number[], timestamp?: number): void {
    if (this.fail) throw new TypeError("invalid status byte");
    this.sent.push({ data, timestamp });
  }
}

class MessageEvent extends Event {
  readonly data: Uint8Array | null;
  constructor(data: Uint8Array | null) {
    super("midimessage");
    this.data = data;
  }
}

class FakeAccess extends EventTarget implements MIDIAccessLike {
  readonly inputs = new Map<string, FakePort>();
  readonly outputs = new Map<string, FakeOutput>();
}

function createHost(): {
  host: MIDIHost;
  access: FakeAccess;
  flags: (MIDIRequestFlags | undefined)[];
  fail: { next: unknown };
} {
  const access = new FakeAccess();
  access.inputs.set("in-1", new FakePort("in-1"));
  access.outputs.set("out-1", new FakeOutput("out-1", null, null));
  const flags: (MIDIRequestFlags | undefined)[] = [];
  const fail: { next: unknown } = { next: undefined };
  const host: MIDIHost = {
    requestMIDIAccess: (options) => {
      flags.push(options);
      if (fail.next !== undefined) return Promise.reject(fail.next);
      return Promise.resolve(access);
    },
  };
  return { host, access, flags, fail };
}

void test("requests access on demand and lists ports", async () => {
  const { host, flags } = createHost();
  const midi = useWebMIDI({ host, sysex: true });
  assert.equal(midi.supported.value, true);
  assert.equal(midi.status.value, "idle");
  assert.equal(flags.length, 0);

  assert.equal(await midi.request(), true);
  assert.deepEqual(flags, [{ sysex: true, software: false }]);
  assert.equal(midi.status.value, "granted");
  assert.deepEqual(midi.inputs.value, [
    {
      id: "in-1",
      name: "port in-1",
      manufacturer: "vize",
      state: "connected",
      connection: "closed",
    },
  ]);
  assert.deepEqual(midi.outputs.value[0], {
    id: "out-1",
    name: "",
    manufacturer: "",
    state: "connected",
    connection: "closed",
  });
});

void test("refreshes on statechange and forwards input messages", async () => {
  const { host, access } = createHost();
  const messages: MIDIInputMessage[] = [];
  const midi = useWebMIDI({ host, onMessage: (message) => messages.push(message) });
  await midi.request();

  const second = new FakePort("in-2");
  access.inputs.set("in-2", second);
  access.dispatchEvent(new Event("statechange"));
  assert.equal(midi.inputs.value.length, 2);

  second.dispatchEvent(new MessageEvent(Uint8Array.of(0x90, 60, 100)));
  second.dispatchEvent(new MessageEvent(null));
  assert.equal(messages.length, 1);
  assert.equal(messages[0]?.inputId, "in-2");
  assert.deepEqual([...(messages[0]?.data ?? [])], [0x90, 60, 100]);
});

void test("sends to outputs and reports failures", async () => {
  const { host, access } = createHost();
  const midi = useWebMIDI({ host });
  assert.equal(midi.send("out-1", [0x90]), false, "nothing before access");
  await midi.request();
  const output = access.outputs.get("out-1");
  assert.ok(output);

  assert.equal(midi.send("out-1", Uint8Array.of(0x80, 60, 0), 12), true);
  assert.equal(midi.send("out-1", [0xf8]), true);
  assert.deepEqual(output.sent, [
    { data: [0x80, 60, 0], timestamp: 12 },
    { data: [0xf8], timestamp: undefined },
  ]);
  assert.equal(midi.send("missing", [0xf8]), false);
  output.fail = true;
  assert.equal(midi.send("out-1", [0x00]), false);
  assert.ok(midi.error.value instanceof TypeError);
});

void test("exposes denial without throwing", async () => {
  const { host, fail } = createHost();
  const denial = new Error("SecurityError");
  fail.next = denial;
  const midi = useWebMIDI({ host });
  assert.equal(await midi.request(), false);
  assert.equal(midi.status.value, "denied");
  assert.equal(midi.error.value, denial);
  assert.equal(useWebMIDI({ host: null }).supported.value, false);
  assert.equal(await useWebMIDI({ host: null }).request(), false);
});

void test("removes listeners with the scope", async () => {
  const { host, access } = createHost();
  const scope = effectScope();
  const midi = scope.run(() => useWebMIDI({ host, onMessage: () => undefined }));
  assert.ok(midi);
  await midi.request();
  const input = access.inputs.get("in-1");
  assert.equal(input?.listeners, 1);

  scope.stop();
  assert.equal(input?.listeners, 0);
  assert.equal(midi.inputs.value.length, 0);
  access.inputs.set("in-3", new FakePort("in-3"));
  access.dispatchEvent(new Event("statechange"));
  assert.equal(midi.inputs.value.length, 0);
});

void test("server rendering requests nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const midi = useWebMIDI();
    return { supported: midi.supported, status: midi.status, inputs: midi.inputs };
  });
  assert.equal(state, '{"supported":false,"status":"idle","inputs":[]}');
});
