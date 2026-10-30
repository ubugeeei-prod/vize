import assert from "node:assert/strict";
import { setImmediate } from "node:timers/promises";
import test from "node:test";

import { instrumentSessionPrototype } from "../observer.ts";
import type { ObservationSink, PhaseEvent } from "../types.ts";

type Deferred = {
  promise: Promise<unknown>;
  resolve(value: unknown): void;
  reject(reason: unknown): void;
};

function deferred(): Deferred {
  let resolve!: (value: unknown) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<unknown>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}

type Message = { id?: number; result?: unknown; error?: unknown };
type Call = { kind: "request" | "notify"; args: unknown[]; receiver: FakeSession };

class FakeSession {
  readonly processId = 900;
  readonly notificationObservers: Array<(method: string, params: unknown) => void> = [];
  readonly responseObservers: Array<(message: Message) => void> = [];
  readonly calls: Call[] = [];
  readonly queued: Deferred[] = [];
  nextResult: unknown = undefined;
  requestError: Error | null = null;
  notifyError: Error | null = null;

  request(...args: unknown[]): Promise<unknown> {
    this.calls.push({ kind: "request", args, receiver: this });
    if (this.requestError) throw this.requestError;
    const following = args[3];
    if (typeof following === "function") Reflect.apply(following, undefined, [4321]);
    return this.queued.shift()?.promise ?? Promise.resolve(this.nextResult);
  }

  notify(...args: unknown[]): void {
    this.calls.push({ kind: "notify", args, receiver: this });
    if (this.notifyError) throw this.notifyError;
  }

  publish(method: string, params: unknown): void {
    for (const observer of this.notificationObservers) observer(method, params);
  }

  respond(message: Message): void {
    for (const observer of this.responseObservers) observer(message);
  }
}

class Sink implements ObservationSink {
  readonly events: PhaseEvent[] = [];
  readonly refusals: string[] = [];
  readonly starts: number[] = [];
  readonly stops: string[] = [];
  throwEvents = false;
  eventCalls = 0;
  event(event: PhaseEvent): void {
    this.eventCalls += 1;
    if (this.throwEvents) throw new Error("synthetic observer sink failure");
    this.events.push(event);
  }
  refuse(reason: string): void {
    this.refusals.push(reason);
  }
  start(pid: number): void {
    this.starts.push(pid);
  }
  stop(reason: string): void {
    this.stops.push(reason);
  }
}

const root = "file:///synthetic/misskey/packages/frontend";
const main = `${root}/src/components/MkDivider.vue`;
const dependency = `${root}/src/components/MkCodeInline.vue`;
const ephemeral = `${root}/src/__vize_ephemeral_lifecycle.vue`;
const source = `<script setup lang="ts">
const createdLifecycleProbe: number = 'broken';
void createdLifecycleProbe;
</script>
`;

async function initialize(session: FakeSession): Promise<void> {
  await session.request("initialize", {
    rootUri: root,
    workspaceFolders: [{ uri: root, name: "frontend" }],
  });
}

function open(session: FakeSession, uri: string, version = 1, text = "synthetic source"): void {
  session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId: "vue", version, text },
  });
}

async function armChurn(session: FakeSession): Promise<void> {
  await initialize(session);
  open(session, dependency);
  open(session, main);
  for (let index = 0; index < 80; index += 1) {
    for (const uri of [main, dependency]) {
      session.notify("textDocument/didChange", {
        textDocument: { uri, version: index + 2 },
        contentChanges: [{ text: "synthetic edit" }],
      });
    }
  }
  for (let index = 0; index < 6; index += 1) {
    session.notify("textDocument/didChange", {
      textDocument: { uri: main, version: index + 82 },
      contentChanges: [{ text: "synthetic fence or cancellation" }],
    });
  }
  for (const uri of [main, dependency]) {
    session.notify("textDocument/didClose", { textDocument: { uri } });
    session.publish("textDocument/publishDiagnostics", { uri, diagnostics: [] });
  }
}

async function completeFirstCycle(session: FakeSession, alreadyOpened = false): Promise<void> {
  if (!alreadyOpened) open(session, ephemeral, 1, source);
  session.publish("textDocument/publishDiagnostics", {
    uri: ephemeral,
    version: 1,
    diagnostics: [{ code: 2322, message: "Type 'string' is not assignable to type 'number'." }],
  });
  session.nextResult = [{ name: "createdLifecycleProbe", location: { uri: ephemeral } }];
  await session.request("workspace/symbol", { query: "createdLifecycleProbe" });
  session.notify("textDocument/didClose", { textDocument: { uri: ephemeral } });
  session.publish("textDocument/publishDiagnostics", { uri: ephemeral, diagnostics: [] });
  session.nextResult = null;
  await session.request("workspace/symbol", { query: "createdLifecycleProbe" });
  await session.request("textDocument/documentSymbol", { textDocument: { uri: ephemeral } });
  open(session, ephemeral, 2, source);
}

test("installer returns exact request promise and forwards this, argument count, object and following identity once", async () => {
  const sink = new Sink();
  const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => sink);
  try {
    const session = new FakeSession();
    const pending = deferred();
    session.queued.push(pending);
    const params = Object.freeze({ rootUri: root, workspaceFolders: [{ uri: root }] });
    let followingCalls = 0;
    const following = (id: number) => {
      assert.equal(id, 4321);
      followingCalls += 1;
      return [];
    };
    const result = session.request("initialize", params, undefined, following);
    assert.equal(result, pending.promise);
    assert.equal(session.calls.length, 1);
    const call = session.calls[0]!;
    assert.equal(call.receiver, session);
    assert.equal(call.args.length, 4);
    assert.equal(call.args[1], params);
    assert.equal(call.args[2], undefined);
    assert.equal(call.args[3], following);
    assert.equal(followingCalls, 1);
    const payload = Object.freeze({ capabilities: {} });
    pending.resolve(payload);
    assert.equal(await result, payload);
  } finally {
    uninstall();
  }
});

test("rejection identity and synchronous request/notify throws remain the caller's original errors", async () => {
  const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => new Sink());
  try {
    const session = new FakeSession();
    const pending = deferred();
    session.queued.push(pending);
    const result = session.request("initialize", { rootUri: root });
    const error = new Error("synthetic original rejection");
    pending.reject(error);
    await assert.rejects(result, (actual: unknown) => actual === error);
    session.requestError = error;
    assert.throws(() => session.request("synthetic/request", {}), (actual: unknown) => actual === error);
    session.notifyError = error;
    assert.throws(() => session.notify("synthetic/notify", {}), (actual: unknown) => actual === error);
    assert.equal(session.calls.length, 3);
    await setImmediate();
  } finally {
    uninstall();
  }
});

test("notify retains void return and does not clone or mutate the caller's packet", () => {
  const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => new Sink());
  try {
    const session = new FakeSession();
    const packet = Object.freeze({ textDocument: Object.freeze({ uri: main }) });
    assert.equal(session.notify("textDocument/didClose", packet), undefined);
    assert.equal(session.calls.length, 1);
    assert.equal(session.calls[0]!.args[1], packet);
    assert.deepEqual(packet, { textDocument: { uri: main } });
  } finally {
    uninstall();
  }
});

test("sink factory failure cannot replace an original result or insert a protocol call", async () => {
  let factoryCalls = 0;
  const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => {
    factoryCalls += 1;
    throw new Error("synthetic factory refusal");
  });
  try {
    const session = new FakeSession();
    const value = Object.freeze({ original: true });
    session.nextResult = value;
    assert.equal(await session.request("initialize", { rootUri: root }), value);
    assert.equal(session.calls.length, 1);
    assert.equal(factoryCalls, 1);
  } finally {
    uninstall();
  }
});

test("full synthetic churn keeps every original packet and call order, arms before ephemeral open, then stops", async () => {
  const plain = new FakeSession();
  await armChurn(plain);
  await completeFirstCycle(plain);
  const expected = plain.calls.map(({ kind, args }) => ({ kind, args }));
  const sink = new Sink();
  const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => sink);
  try {
    const observed = new FakeSession();
    await armChurn(observed);
    assert.deepEqual(sink.starts, [900], "sampling must arm at existing close, before ephemeral didOpen");
    assert.equal(observed.calls.some((call) => JSON.stringify(call.args).includes(ephemeral)), false);
    await completeFirstCycle(observed);
    assert.deepEqual(observed.calls.map(({ kind, args }) => ({ kind, args })), expected);
    assert.equal(observed.calls.filter(({ kind }) => kind === "request").length, 4);
    assert.ok(sink.stops.length > 0, "next cycle must close the bounded observation window");
    assert.deepEqual(sink.refusals, []);
  } finally {
    uninstall();
  }
});

test("passive callbacks preserve pre-existing observer order, payload identity and restoration", async () => {
  const sink = new Sink();
  const originalRequest = FakeSession.prototype.request;
  const originalNotify = FakeSession.prototype.notify;
  const session = new FakeSession();
  const seen: string[] = [];
  const notification = { uri: main, diagnostics: [] };
  const response = { id: 999, result: Object.freeze({ original: true }) };
  const firstNotification = (_method: string, value: unknown) => {
    assert.equal(value, notification);
    seen.push("notification-first");
  };
  const firstResponse = (value: Message) => {
    assert.equal(value, response);
    seen.push("response-first");
  };
  session.notificationObservers.push(firstNotification);
  session.responseObservers.push(firstResponse);
  const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => sink);
  try {
    await initialize(session);
    session.notificationObservers.push(() => seen.push("notification-last"));
    session.responseObservers.push(() => seen.push("response-last"));
    session.publish("textDocument/publishDiagnostics", notification);
    session.respond(response);
    assert.deepEqual(seen, ["notification-first", "notification-last", "response-first", "response-last"]);
  } finally {
    uninstall();
  }
  assert.equal(FakeSession.prototype.request, originalRequest);
  assert.equal(FakeSession.prototype.notify, originalNotify);
  assert.equal(session.notificationObservers[0], firstNotification);
  assert.equal(session.responseObservers[0], firstResponse);
  assert.equal(session.notificationObservers.length, 2, "uninstall removes only its own callback");
  assert.equal(session.responseObservers.length, 2, "uninstall removes only its own callback");
});

test("observer event errors do not escape public methods or asynchronous callbacks", async () => {
  const sink = new Sink();
  sink.throwEvents = true;
  const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => sink);
  try {
    const session = new FakeSession();
    await armChurn(session);
    await completeFirstCycle(session);
    assert.ok(sink.eventCalls > 0, "negative control must actually throw inside the observer");
    assert.equal(session.calls.filter(({ kind }) => kind === "request").length, 4);
    await setImmediate();
  } finally {
    uninstall();
  }
});

test("incremental-like and foreign-source close traffic cannot arm the first-cycle sampler", async () => {
  const sinks: Sink[] = [];
  const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => {
    const sink = new Sink();
    sinks.push(sink);
    return sink;
  });
  try {
    const incremental = new FakeSession();
    await initialize(incremental);
    open(incremental, main);
    incremental.notify("textDocument/didClose", { textDocument: { uri: main } });
    const foreign = new FakeSession();
    await initialize(foreign);
    for (let version = 2; version < 168; version += 1) {
      foreign.notify("textDocument/didChange", {
        textDocument: { uri: "file:///foreign/MkDivider.vue", version },
        contentChanges: [{ text: "foreign" }],
      });
    }
    foreign.notify("textDocument/didClose", { textDocument: { uri: main } });
    assert.equal(sinks.length, 2);
    assert.ok(sinks.every((sink) => sink.starts.length === 0));
  } finally {
    uninstall();
  }
});

test("unmatched incoming responses stay uncorrelated until the original workspace promise settles", async () => {
  const sink = new Sink();
  const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => sink);
  try {
    const session = new FakeSession();
    await armChurn(session);
    open(session, ephemeral, 1, source);
    const pending = deferred();
    session.queued.push(pending);
    const result = session.request("workspace/symbol", { query: "createdLifecycleProbe" });
    assert.equal(result, pending.promise);
    const foreignResponse = Object.freeze({ id: 999_999, result: null });
    session.respond(foreignResponse);
    await setImmediate();
    assert.equal(sink.events.filter((event) => event.phase === "workspace-open-scan-settled").length, 0);
    assert.equal(sink.events.filter((event) => event.phase === "uncorrelated-response").length, 1);
    const value = [{ name: "createdLifecycleProbe", location: { uri: ephemeral } }];
    pending.resolve(value);
    assert.equal(await result, value);
    assert.equal(sink.events.filter((event) => event.phase === "workspace-open-scan-settled").length, 1);
    assert.deepEqual(foreignResponse, { id: 999_999, result: null });
    assert.equal(session.calls.filter(({ kind }) => kind === "request").length, 2);
  } finally {
    uninstall();
  }
});

test("first-epoch URI, version and full source identity cannot be substituted", async () => {
  for (const [uri, version, text] of [
    ["file:///foreign/src/__vize_ephemeral_lifecycle.vue", 1, source],
    [ephemeral, 2, source],
    [ephemeral, 1, source.replace("'broken'", "'changed'")],
  ] as const) {
    const sink = new Sink();
    const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => sink);
    try {
      const session = new FakeSession();
      await armChurn(session);
      open(session, uri, version, text);
      await session.request("workspace/symbol", { query: "createdLifecycleProbe" });
      assert.equal(sink.events.filter((event) => event.phase === "ephemeral-did-open-before-write").length, 0);
      assert.equal(sink.events.filter((event) => event.phase === "workspace-open-scan-request-before-call").length, 0);
      assert.equal(session.calls.filter(({ kind }) => kind === "request").length, 2);
    } finally {
      uninstall();
    }
  }
});

test("versionless nonempty diagnostics are never labelled the close-clear fence", async () => {
  const sink = new Sink();
  const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => sink);
  try {
    const session = new FakeSession();
    await armChurn(session);
    open(session, ephemeral, 1, source);
    session.nextResult = [{ name: "createdLifecycleProbe", location: { uri: ephemeral } }];
    await session.request("workspace/symbol", { query: "createdLifecycleProbe" });
    session.notify("textDocument/didClose", { textDocument: { uri: ephemeral } });
    const notClear = Object.freeze({ uri: ephemeral, diagnostics: [{ message: "still present" }] });
    session.publish("textDocument/publishDiagnostics", notClear);
    assert.equal(sink.events.filter((event) => event.phase === "ephemeral-close-clear").length, 0);
    session.publish("textDocument/publishDiagnostics", { uri: ephemeral, diagnostics: [] });
    assert.equal(sink.events.filter((event) => event.phase === "ephemeral-close-clear").length, 1);
    assert.equal(notClear.diagnostics.length, 1);
  } finally {
    uninstall();
  }
});

test("wrong query does not consume either original workspace-scan slot", async () => {
  const sink = new Sink();
  const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => sink);
  try {
    const session = new FakeSession();
    await armChurn(session);
    open(session, ephemeral, 1, source);
    await session.request("workspace/symbol", { query: "foreignProbe" });
    assert.equal(sink.events.filter((event) => event.phase === "workspace-open-scan-request-before-call").length, 0);
    await completeFirstCycle(session, true);
    assert.equal(sink.events.filter((event) => event.phase === "workspace-open-scan-request-before-call").length, 1);
    assert.equal(sink.events.filter((event) => event.phase === "workspace-deleted-scan-request-before-call").length, 1);
    assert.equal(session.calls.filter(({ kind }) => kind === "request").length, 5);
  } finally {
    uninstall();
  }
});

test("empty and unrelated version1 publications stay generic until the original workspace scan starts", async () => {
  const sink = new Sink();
  const uninstall = instrumentSessionPrototype(FakeSession.prototype, () => sink);
  try {
    const session = new FakeSession();
    await armChurn(session);
    open(session, ephemeral, 1, source);
    for (const diagnostics of [
      [],
      [{ code: 2307, message: "Cannot find module 'other'." }],
      [{ code: 2322, message: "Type 'number' is not assignable to type 'string'." }],
    ]) {
      session.publish("textDocument/publishDiagnostics", { uri: ephemeral, version: 1, diagnostics });
      assert.equal(sink.events.filter((event) => event.phase === "ephemeral-open-publication").length, 0);
      assert.equal(sink.events.filter((event) => event.phase === "workspace-open-scan-request-before-call").length, 0);
    }
    session.publish("textDocument/publishDiagnostics", {
      uri: ephemeral,
      version: 1,
      diagnostics: [{ code: "TS2322", message: "Type 'string' is not assignable to type 'number'." }],
    });
    assert.equal(sink.events.filter((event) => event.phase === "ephemeral-open-publication").length, 0);
    assert.equal(sink.events.filter((event) => event.phase === "ephemeral-version1-publication").length, 4);
    session.nextResult = [{ name: "createdLifecycleProbe", location: { uri: ephemeral } }];
    await session.request("workspace/symbol", { query: "createdLifecycleProbe" });
    assert.equal(sink.events.filter((event) => event.phase === "workspace-open-scan-request-before-call").length, 1);
  } finally {
    uninstall();
  }
});
