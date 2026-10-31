import { createHash } from "node:crypto";
import type { ObservationSink, PhaseEvent } from "./types.ts";

function object(value: unknown): object {
  if (value === null || (typeof value !== "object" && typeof value !== "function")) throw new Error("expected object");
  return value;
}
function field(value: unknown, key: string): unknown { return Reflect.get(object(value), key); }
function array(value: unknown): unknown[] { if (!Array.isArray(value)) throw new Error("expected array"); return value; }
type Callable = (this: object, ...args: unknown[]) => unknown;
function callable(value: unknown): value is Callable { return typeof value === "function"; }
function descriptor(prototype: object, name: string): PropertyDescriptor {
  const value = Object.getOwnPropertyDescriptor(prototype, name);
  if (value === undefined || (!value.configurable && !value.writable)) throw new Error("public seam is not replaceable");
  return value;
}
function promise(value: unknown): asserts value is Promise<unknown> { if (!(value instanceof Promise)) throw new Error("request returned non-Promise"); }
function fingerprint(value: unknown): { bytes: number; sha256: string } {
  const encoded = JSON.stringify(value); if (typeof encoded !== "string") throw new Error("not JSON");
  return { bytes: Buffer.byteLength(encoded), sha256: createHash("sha256").update(encoded).digest("hex") };
}
const ephemeralText = "<script setup lang=\"ts\">\nconst createdLifecycleProbe: number = 'broken';\nvoid createdLifecycleProbe;\n</script>\n";
export const EPHEMERAL_SOURCE_SHA256 = createHash("sha256").update(ephemeralText).digest("hex");

type Tracker = {
  pid: number; sink: ObservationSink; workspace: string | undefined; main: string | undefined;
  dependency: string | undefined; ephemeral: string | undefined; changes: number;
  opened: Set<string>; armed: boolean; active: boolean; completed: boolean;
  workspaceCalls: number; closed: boolean; hookNs: bigint;
  clearPublished: boolean;
};
/** Pure/injectable seam: never starts proc I/O itself and never awaits a caller. */
export function instrumentSessionPrototype(
  prototype: object,
  sinkFactory: (pid: number) => ObservationSink,
  now: () => bigint = () => process.hrtime.bigint(),
): () => void {
  const requestDescriptor = descriptor(prototype, "request");
  const notifyDescriptor = descriptor(prototype, "notify");
  const requestValue: unknown = requestDescriptor.value;
  const notifyValue: unknown = notifyDescriptor.value;
  if (!callable(requestValue) || !callable(notifyValue)) throw new Error("public seams absent");
  const originalRequest = requestValue;
  const originalNotify = notifyValue;
  const tracked = new WeakMap<object, Tracker>();
  const bindings: Array<{ session: WeakRef<object>; state: Tracker;
    notification: (method: string, params: unknown) => void; response: (message: unknown) => void }> = [];
  function refuse(state: Tracker, reason: string): void {
    try { state.sink.refuse(reason); } catch { /* Observation failure never changes dispatch. */ }
  }
  function guard(state: Tracker, work: () => void): void {
    let start: bigint | undefined;
    try { start = now(); work(); } catch (error: unknown) { refuse(state, error instanceof Error ? error.message : String(error)); }
    finally {
      try { if (start !== undefined) state.hookNs += now() - start; }
      catch { refuse(state, "observation clock failed"); }
    }
  }
  function event(state: Tracker, phase: string, extra: Omit<PhaseEvent, "phase" | "atNs"> = {}): void {
    state.sink.event({ phase, atNs: now().toString(), ...extra });
  }
  function get(session: object): Tracker | undefined {
    const existing = tracked.get(session); if (existing !== undefined) return existing;
    try {
      const pid: unknown = Reflect.get(session, "processId");
      if (typeof pid !== "number" || !Number.isSafeInteger(pid) || pid <= 0) return undefined;
      const sink = sinkFactory(pid);
      const state: Tracker = { pid, sink, workspace: undefined, main: undefined, dependency: undefined,
        ephemeral: undefined, changes: 0, opened: new Set(), armed: false, active: false,
        completed: false, workspaceCalls: 0, closed: false, hookNs: 0n, clearPublished: false };
      const notifications = array(Reflect.get(session, "notificationObservers"));
      const responses = array(Reflect.get(session, "responseObservers"));
      const notification = (method: string, params: unknown): void => guard(state, () => {
        if (!state.active || method !== "textDocument/publishDiagnostics" || field(params, "uri") !== state.ephemeral) return;
        const version = field(params, "version");
        if (version === 1 && !state.closed) {
          const diagnostics = array(field(params, "diagnostics"));
          // Informational only: the unchanged caller's original exact wait and
          // assertions precede the first workspace request. Do not duplicate
          // that semantic oracle or claim a publication closes the native work.
          event(state, "ephemeral-version1-publication", { method, detail: { diagnostics: diagnostics.length } });
        }
        else if (version == null && state.closed) {
          const diagnostics = field(params, "diagnostics");
          if (Array.isArray(diagnostics) && diagnostics.length === 0 && !state.clearPublished) {
            state.clearPublished = true; event(state, "ephemeral-close-clear", { method });
          }
          else if (Array.isArray(diagnostics) && diagnostics.length === 0) event(state, "uncorrelated-publication", { method });
          else refuse(state, "versionless publication was not an empty clear");
        }
      });
      const response = (message: unknown): void => guard(state, () => {
        if (!state.active) return;
        const id = field(message, "id"); const digest = fingerprint(message);
        event(state, "uncorrelated-response", { detail: { id: typeof id === "number" || typeof id === "string" ? id : null, ...digest } });
      });
      notifications.push(notification); responses.push(response);
      bindings.push({ session: new WeakRef(session), state, notification, response });
      tracked.set(session, state);
      return state;
    } catch { return undefined; }
  }
  function beforeNotify(state: Tracker, method: unknown, params: unknown): void {
    if (typeof method !== "string" || state.workspace === undefined) return;
    if (method !== "textDocument/didOpen" && method !== "textDocument/didChange" && method !== "textDocument/didClose") return;
    const document = field(params, "textDocument"); const uri = field(document, "uri");
    if (typeof uri !== "string") return;
    const owned = uri === state.main || uri === state.dependency;
    if (method === "textDocument/didOpen" && owned) state.opened.add(uri);
    if (method === "textDocument/didChange" && owned && state.opened.has(uri)) state.changes += 1;
    if (method === "textDocument/didClose" && owned && state.opened.size === 2 && state.changes >= 160 && !state.armed) {
      state.armed = true; state.sink.start(state.pid); event(state, "pre-lifecycle-source-close", { method, detail: { changes: state.changes } });
    }
    if (uri !== state.ephemeral || method !== "textDocument/didOpen") {
      if (state.active && uri === state.ephemeral && method === "textDocument/didClose") {
        state.closed = true; event(state, "ephemeral-did-close-before-write", { method });
      }
      return;
    }
    const version = field(document, "version");
    if (version === 2 && state.completed) {
      event(state, "next-ephemeral-v2-before-write", { detail: { hookNs: state.hookNs.toString() } });
      state.active = false; state.sink.stop("rss-fence0-bracket-closed"); return;
    }
    if (version !== 1 || !state.armed || state.active) return;
    if (field(document, "languageId") !== "vue" || fingerprint(field(document, "text")).sha256 !== fingerprint(ephemeralText).sha256) {
      refuse(state, "foreign ephemeral source/version"); return;
    }
    state.active = true; event(state, "ephemeral-did-open-before-write", { method, detail: { sourceSha256: EPHEMERAL_SOURCE_SHA256 } });
  }
  function beforeRequest(state: Tracker, method: unknown, params: unknown): string | undefined {
    if (method === "initialize") {
      const root = field(params, "rootUri");
      if (typeof root === "string" && root.startsWith("file://")) {
        state.workspace = root.endsWith("/") ? root : `${root}/`;
        state.main = new URL("src/components/MkDivider.vue", state.workspace).href;
        state.dependency = new URL("src/components/MkCodeInline.vue", state.workspace).href;
        state.ephemeral = new URL("src/__vize_ephemeral_lifecycle.vue", state.workspace).href;
      }
    }
    if (!state.active || state.completed) return undefined;
    if (method === "workspace/symbol" && field(params, "query") === "createdLifecycleProbe") {
      state.workspaceCalls += 1;
      if (state.workspaceCalls > 2) { refuse(state, "extra workspace query in first lifecycle"); return undefined; }
      const phase = state.workspaceCalls === 1 ? "workspace-open-scan" : "workspace-deleted-scan";
      if ((state.workspaceCalls === 1 && state.closed) || (state.workspaceCalls === 2 && !state.closed)) refuse(state, "workspace phase order");
      if (state.workspaceCalls === 2 && !state.clearPublished) refuse(state, "workspace before empty close clear");
      event(state, `${phase}-request-before-call`, { method }); return phase;
    }
    if (method === "textDocument/documentSymbol" && field(field(params, "textDocument"), "uri") === state.ephemeral) {
      if (state.workspaceCalls !== 2 || !state.closed) refuse(state, "document symbol phase order");
      event(state, "final-document-symbol-request-before-call", { method }); return "final-document-symbol";
    }
    return undefined;
  }
  function wrappedRequest(this: object, ...args: unknown[]): unknown {
    const state = get(this); let phase: string | undefined;
    if (state !== undefined) guard(state, () => { phase = beforeRequest(state, args[0], args[1]); });
    const result: unknown = Reflect.apply(originalRequest, this, args);
    if (state !== undefined && phase !== undefined) {
      const selectedPhase = phase;
      guard(state, () => {
        promise(result);
        void result.then((value: unknown) => guard(state, () => {
          const digest = fingerprint(value); event(state, `${selectedPhase}-settled`, { outcome: "fulfilled", detail: digest });
          if (selectedPhase === "final-document-symbol") { state.completed = true; state.sink.stop("first-cycle-methods-complete"); }
        }), (_error: unknown) => guard(state, () => {
          event(state, `${selectedPhase}-settled`, { outcome: "rejected" }); refuse(state, "original scoped request rejected");
          state.sink.stop("original-request-rejected");
        }));
      });
    }
    return result;
  }
  function wrappedNotify(this: object, ...args: unknown[]): unknown {
    const state = get(this);
    if (state !== undefined) guard(state, () => beforeNotify(state, args[0], args[1]));
    return Reflect.apply(originalNotify, this, args);
  }
  Object.defineProperty(prototype, "request", { ...requestDescriptor, value: wrappedRequest });
  Object.defineProperty(prototype, "notify", { ...notifyDescriptor, value: wrappedNotify });
  return () => {
    for (const binding of bindings) {
      const session = binding.session.deref();
      if (session === undefined) continue;
      guard(binding.state, () => {
        for (const [name, callback] of [["notificationObservers", binding.notification], ["responseObservers", binding.response]]) {
          if (typeof name !== "string") throw new Error("observer binding name");
          const observers = array(Reflect.get(session, name)); const index = observers.indexOf(callback);
          if (index >= 0) observers.splice(index, 1);
        }
        binding.state.sink.stop("instrumentation-uninstalled");
      });
    }
    bindings.length = 0;
    if (Reflect.get(prototype, "request") === wrappedRequest) Object.defineProperty(prototype, "request", requestDescriptor);
    if (Reflect.get(prototype, "notify") === wrappedNotify) Object.defineProperty(prototype, "notify", notifyDescriptor);
  };
}
// Runtime attachment is deliberately a separate explicit import, wired only by
// the diagnostic workflow after custody preflight; importing this module is pure.
