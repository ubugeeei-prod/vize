export const VBEN_LANES = [
  "initialize", "coldOpen", "coldOpenSecondApp", "completion", "hover", "warmNoop",
  "leafBroken", "leafRepaired", "sharedBroken", "sharedBrokenSecondApp", "sharedRepaired",
  "sharedRepairedSecondApp", "warmNoopSecondApp", "cancellationConverge",
] as const;
export const VBEN_REQUIRED_PHASES = VBEN_LANES.flatMap((lane) => [`vben-${lane}-before`, `vben-${lane}-settled`]);
export type VbenNativeTarget = Readonly<{ pid: number; birthTicks: string; parentPid: number; nativeImageSha: string }>;
export type VbenRoot = Readonly<{ rootPid: number; rootBirthTicks: string; serverImageSha: string; nativeImageSha: string; startNs: string }>;
export type VbenPhase = VbenRoot & {
  phase: string; lane: string; stage: "before" | "settled"; outcome: "pending" | "fulfilled" | "rejected" | "threw";
  atNs: string; target: VbenNativeTarget | null; sourceSnapshot: unknown;
};
export interface VbenSink {
  phase(event: VbenPhase): void;
  refuse(reason: string): void;
  stop(reason: string): void;
}
type Member = { pid: number; parentPid: number; role: "root" | "descendant"; rssKiB: number; executable: string;
  birth: string; hash: string; kind: string; runtimeRole: string; argvCount: number; argvSha: string };
type State = { root: VbenRoot; sink: VbenSink; next: number; active: boolean; invalid: boolean; stopped: boolean;
  baseline: Set<number> | undefined; target: VbenNativeTarget | null; history: Map<number, string> };
type Options = { serverImageSha: string; nativeImageSha: string; sinkFactory(root: VbenRoot): VbenSink; now?: () => bigint };
type Callable = (this: unknown, ...args: unknown[]) => unknown;

function object(value: unknown): object {
  if (value === null || (typeof value !== "object" && typeof value !== "function")) throw new Error("metrics object absent");
  return value;
}
function field(value: unknown, key: string): unknown { return Reflect.get(object(value), key); }
function integer(value: unknown, minimum = 0): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < minimum) throw new Error("invalid process integer");
  return value;
}
function string(value: unknown): string { if (typeof value !== "string") throw new Error("invalid process string"); return value; }
function digest(value: unknown): string { const result = string(value); if (!/^[a-f0-9]{64}$/.test(result)) throw new Error("invalid image digest"); return result; }
function birth(value: unknown): string { const result = string(value); if (!/^\d+$/.test(result)) throw new Error("invalid process birth"); return result; }
function member(value: unknown): Member {
  const identity = field(value, "identity"), role = field(value, "role");
  if (role !== "root" && role !== "descendant") throw new Error("invalid member role");
  const result: Member = { pid: integer(field(value, "pid"), 1), parentPid: integer(field(value, "parentPid")), role,
    rssKiB: integer(field(value, "rssKiB")), executable: string(field(value, "executable")),
    birth: birth(field(identity, "birthTicks")), hash: digest(field(identity, "executableSha256")),
    kind: string(field(identity, "executableKind")), runtimeRole: string(field(identity, "runtimeRole")),
    argvCount: integer(field(identity, "argvCount"), 1), argvSha: digest(field(identity, "argvSha256")) };
  if (!["vize", "node", "other"].includes(result.kind) || result.executable !== result.kind
    || !["native-lsp", "native-api", "typescript-lsp", "typescript-api", "other"].includes(result.runtimeRole)) throw new Error("member kind/role refused");
  return result;
}
function native(value: Member): boolean { return value.runtimeRole === "typescript-lsp" || value.runtimeRole === "typescript-api"; }

/** Pure observer: only reads retained metrics snapshots; never performs live process I/O. */
export function instrumentIncrementalMetricsPrototype(prototype: object, options: Options): () => void {
  digest(options.serverImageSha); digest(options.nativeImageSha);
  const descriptor = Object.getOwnPropertyDescriptor(prototype, "measure");
  if (descriptor === undefined || typeof descriptor.value !== "function" || (!descriptor.configurable && !descriptor.writable)) throw new Error("measure seam absent");
  const original: Callable = descriptor.value as Callable;
  const now = options.now ?? (() => process.hrtime.bigint());
  const states = new WeakMap<object, State>();
  const bindings: State[] = [];
  function refuse(state: State, reason: string): void {
    state.invalid = true;
    try { state.sink.refuse(reason.slice(0, 160)); } catch { /* Observer callbacks cannot change the caller. */ }
  }
  function guard(state: State, work: () => void): void {
    try { work(); } catch (error: unknown) { refuse(state, error instanceof Error ? error.message : "observer failure"); }
  }
  function stop(state: State, reason: string): void {
    if (state.stopped) return; state.stopped = true;
    try { state.sink.stop(reason); } catch { refuse(state, "observer stop failed"); }
  }
  function get(receiver: unknown): State | undefined {
    try {
      const key = object(receiver), existing = states.get(key); if (existing !== undefined) return existing;
      if (field(field(receiver, "suite"), "id") !== "vben-lsp-incremental") return undefined;
      const root: VbenRoot = Object.freeze({ rootPid: integer(field(receiver, "processId"), 1),
        rootBirthTicks: birth(field(receiver, "expectedRootBirthTicks")), serverImageSha: options.serverImageSha,
        nativeImageSha: options.nativeImageSha, startNs: now().toString() });
      const sink = options.sinkFactory(root);
      const state: State = { root, sink, next: 0, active: false, invalid: false, stopped: false,
        baseline: undefined, target: null, history: new Map() };
      states.set(key, state); bindings.push(state); return state;
    } catch { return undefined; /* Metadata/setup failure cannot replace original dispatch. */ }
  }
  function phase(state: State, lane: string, stage: VbenPhase["stage"], outcome: VbenPhase["outcome"], snapshot: unknown): void {
    state.sink.phase({ ...state.root, phase: `vben-${lane}-${stage}`, lane, stage, outcome,
      atNs: now().toString(), target: state.invalid ? null : state.target, sourceSnapshot: snapshot });
  }
  function observeSnapshot(state: State, lane: string, snapshot: unknown): void {
    const values = field(snapshot, "members"); if (!Array.isArray(values) || values.length === 0 || values.length > 4) throw new Error("process cohort size refused");
    const members = values.map((value: unknown) => member(value));
    const byPid = new Map(members.map((value) => [value.pid, value]));
    if (byPid.size !== members.length || integer(field(snapshot, "processes"), 1) !== members.length
      || integer(field(snapshot, "totalKiB")) !== members.reduce((sum, value) => sum + value.rssKiB, 0)) throw new Error("process cohort census refused");
    const roots = members.filter((value) => value.role === "root"); const root = roots[0];
    if (roots.length !== 1 || root === undefined || root.pid !== state.root.rootPid || root.birth !== state.root.rootBirthTicks
      || root.kind !== "vize" || root.runtimeRole !== "native-lsp" || root.hash !== state.root.serverImageSha) throw new Error("Vize root identity refused");
    for (const value of members) {
      if (value.pid !== root.pid && (value.role !== "descendant" || !byPid.has(value.parentPid))) throw new Error("descendant parent refused");
      const seen = new Set<number>(); let cursor = value;
      while (cursor.pid !== root.pid) {
        if (seen.has(cursor.pid)) throw new Error("cyclic process cohort"); seen.add(cursor.pid);
        const parent = byPid.get(cursor.parentPid); if (parent === undefined) throw new Error("foreign process cohort"); cursor = parent;
      }
      if (native(value) && (value.kind !== "other" || value.parentPid !== root.pid || value.hash !== state.root.nativeImageSha)) throw new Error("native parent/image refused");
      const identity = JSON.stringify([value.parentPid, value.role, value.birth, value.hash, value.kind, value.runtimeRole, value.argvCount, value.argvSha]);
      const previous = state.history.get(value.pid); if (previous !== undefined && previous !== identity) throw new Error("PID/image/role identity changed");
      state.history.set(value.pid, identity);
    }
    if (lane === "coldOpen") state.baseline = new Set(members.map((value) => value.pid));
    if (lane === "coldOpenSecondApp" || state.target !== null) {
      if (state.baseline === undefined) throw new Error("coldOpen cohort absent");
      const candidates = members.filter((value) => native(value) && !state.baseline?.has(value.pid));
      if (candidates.length !== 1) throw new Error("new native cohort is empty or ambiguous");
      const chosen = candidates[0]; if (chosen === undefined) throw new Error("native selection absent");
      const target = Object.freeze({ pid: chosen.pid, birthTicks: chosen.birth, parentPid: chosen.parentPid, nativeImageSha: chosen.hash });
      if (state.target !== null && JSON.stringify(target) !== JSON.stringify(state.target)) throw new Error("selected native owner changed");
      state.target = target;
    }
  }
  function settled(state: State, receiver: unknown, lane: string, outcome: VbenPhase["outcome"]): void {
    let snapshot: unknown = null;
    guard(state, () => {
      snapshot = field(field(receiver, "processTreeSamples"), lane);
      if (!state.invalid) observeSnapshot(state, lane, snapshot);
    });
    if (outcome !== "fulfilled") refuse(state, "original measure did not fulfill");
    guard(state, () => phase(state, lane, "settled", outcome, snapshot)); state.active = false;
    if (outcome !== "fulfilled" || state.next === VBEN_LANES.length) stop(state, outcome === "fulfilled" ? "vben-all-lanes-settled" : "vben-measure-failed");
  }
  function wrapped(this: unknown, ...args: unknown[]): unknown {
    const state = get(this); const value = args[0];
    const lane = typeof value === "string" && (VBEN_LANES as readonly string[]).includes(value) ? value : undefined;
    if (state !== undefined) {
      guard(state, () => {
        if (lane === undefined || state.active || state.stopped || VBEN_LANES[state.next] !== lane) throw new Error("unexpected Vben lane/order");
        state.active = true; state.next += 1; phase(state, lane, "before", "pending", null);
      });
    }
    let result: unknown;
    try { result = Reflect.apply(original, this, args); }
    catch (error: unknown) { if (state !== undefined && lane !== undefined) settled(state, this, lane, "threw"); throw error; }
    if (state !== undefined && lane !== undefined) {
      if (result instanceof Promise) {
        try { Reflect.apply(Promise.prototype.then, result, [() => settled(state, this, lane, "fulfilled"), () => settled(state, this, lane, "rejected")]); }
        catch { refuse(state, "measure promise observation failed"); }
      } else { refuse(state, "measure returned non-Promise"); stop(state, "vben-non-promise"); }
    }
    return result;
  }
  Object.defineProperty(prototype, "measure", { ...descriptor, value: wrapped });
  return () => {
    if (Object.getOwnPropertyDescriptor(prototype, "measure")?.value === wrapped) Object.defineProperty(prototype, "measure", descriptor);
    for (const state of bindings) {
      if (state.next !== VBEN_LANES.length || state.active) refuse(state, "Vben lanes incomplete at uninstall");
      stop(state, "vben-uninstall");
    }
  };
}
