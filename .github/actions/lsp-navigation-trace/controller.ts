import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { ChildProcess, spawn, execFileSync } from "node:child_process";
import { Readable } from "node:stream";
import { fileURLToPath, pathToFileURL } from "node:url";
import { joinRaw, RAW_LIMIT, type RawOwner } from "./log-join.ts";

type ObjectValue = Record<string, unknown>;
export type Image = { realpath: string; sha256: string; dev: string; ino: string; bytes: number; mtimeNs: string; ctimeNs: string };
export type Config = { schema: "vize.navigation-trace-config"; version: 1; repoRoot: string; outputRoot: string;
  sourceRevision: string; workflowRevision: string; runId: string; runAttempt: string; serverImage: Image;
  nodeImage: Image; wireSourceSha256: string; benchmarkSourceSha256: string };
type Callable = (this: unknown, ...args: unknown[]) => unknown;
export type CaptureHandle = { dispose(): void; seal(): void };
const sha = (value: Uint8Array): string => createHash("sha256").update(value).digest("hex");
function object(value: unknown): ObjectValue { assert(value !== null && typeof value === "object" && !Array.isArray(value)); return Object.fromEntries(Object.entries(value)); }
function property(value: unknown, key: string): unknown { assert(value !== null && (typeof value === "object" || typeof value === "function")); return Reflect.get(value, key); }
function string(value: unknown): string { assert(typeof value === "string"); return value; }
function env(key: string): string { const value = process.env[key]; assert(value !== undefined && value.length > 0, key); return value; }
function readJson(file: string): unknown { const info = fs.lstatSync(file); assert(info.isFile() && !info.isSymbolicLink() && info.size <= 4 * 1024 * 1024); const value: unknown = JSON.parse(fs.readFileSync(file, "utf8")); return value; }
function write(file: string, value: unknown, cap = 4 * 1024 * 1024): void { const bytes = Buffer.from(JSON.stringify(value, null, 2) + "\n"); assert(bytes.length <= cap); fs.writeFileSync(file, bytes, { flag: "wx", mode: 0o600 }); }
function metadata(file: string): Omit<Image, "realpath" | "sha256"> { const s = fs.statSync(file, { bigint: true }); assert(s.isFile()); return { dev: String(s.dev), ino: String(s.ino), bytes: Number(s.size), mtimeNs: String(s.mtimeNs), ctimeNs: String(s.ctimeNs) }; }
function checkImage(image: Image): void { const { realpath, sha256: _hash, ...expected } = image; assert.equal(fs.realpathSync(realpath), realpath); assert.deepEqual(metadata(realpath), expected); }
function image(value: unknown): Image {
  const row = object(value), realpath = string(row["realpath"]), hash = string(row["sha256"]); assert(path.isAbsolute(realpath)); assert.match(hash, /^[a-f0-9]{64}$/);
  const bytes = row["bytes"]; assert(typeof bytes === "number" && Number.isSafeInteger(bytes) && bytes > 0);
  return { realpath, sha256: hash, bytes, dev: string(row["dev"]), ino: string(row["ino"]), mtimeNs: string(row["mtimeNs"]), ctimeNs: string(row["ctimeNs"]) };
}
export function parseConfig(value: unknown): Config {
  const row = object(value); assert.equal(row["schema"], "vize.navigation-trace-config"); assert.equal(row["version"], 1);
  const result: Config = { schema: "vize.navigation-trace-config", version: 1, repoRoot: string(row["repoRoot"]), outputRoot: string(row["outputRoot"]),
    sourceRevision: string(row["sourceRevision"]), workflowRevision: string(row["workflowRevision"]), runId: string(row["runId"]), runAttempt: string(row["runAttempt"]),
    serverImage: image(row["serverImage"]), nodeImage: image(row["nodeImage"]), wireSourceSha256: string(row["wireSourceSha256"]), benchmarkSourceSha256: string(row["benchmarkSourceSha256"]) };
  for (const value of [result.sourceRevision, result.workflowRevision]) assert.match(value, /^[a-f0-9]{40}$/);
  for (const value of [result.wireSourceSha256, result.benchmarkSourceSha256]) assert.match(value, /^[a-f0-9]{64}$/);
  for (const value of [result.runId, result.runAttempt]) assert.match(value, /^[1-9][0-9]*$/);
  for (const directory of [result.repoRoot, result.outputRoot]) { assert(path.isAbsolute(directory)); assert.equal(fs.realpathSync(directory), directory); }
  assert.notEqual(result.repoRoot, result.outputRoot); return result;
}
/** A nonasync wrapper: exact receiver, arguments, original Promise/errors, and one dispatch. */
export function instrumentWirePrototype(prototype: object, factory: (wire: unknown) => CaptureHandle, refuse: (reason: string) => void): () => void {
  const descriptor = Object.getOwnPropertyDescriptor(prototype, "initialize"); assert(descriptor !== undefined && typeof descriptor.value === "function");
  const original: Callable = descriptor.value as Callable, handles: CaptureHandle[] = [];
  function failure(reason: string): void { try { refuse(reason); } catch { /* Never change the original dispatch. */ } }
  function wrapped(this: unknown, ...args: unknown[]): unknown {
    try { if (property(property(this, "launch"), "name") === "vize") handles.push(factory(this)); }
    catch (error: unknown) { failure(error instanceof Error ? error.message : "raw attachment failed"); }
    return Reflect.apply(original, this, args);
  }
  Object.defineProperty(prototype, "initialize", { ...descriptor, value: wrapped });
  return () => {
    if (Object.getOwnPropertyDescriptor(prototype, "initialize")?.value === wrapped) Object.defineProperty(prototype, "initialize", descriptor);
    for (const handle of handles) { try { handle.seal(); handle.dispose(); } catch { failure("raw finalization failed"); } }
  };
}
function anchor(pid: number, expected: Image): { birthTicks: string; parentPid: string; executable: string; namespaces: ObjectValue; bootIdSha256: string } {
  const small = (file: string): string => { const bytes = fs.readFileSync(file); assert(bytes.length <= 8192); return bytes.toString("utf8"); };
  const stat = (): string[] => { const text = small(`/proc/${pid}/stat`), end = text.lastIndexOf(")"); assert(end > 0 && text.startsWith(`${pid} (`)); return text.slice(end + 2).trim().split(/\s+/); };
  const before = stat(), executable = fs.readlinkSync(`/proc/${pid}/exe`); assert.equal(executable, expected.realpath, "actual child executable path differs");
  const { realpath: _path, sha256: _hash, ...expectedMetadata } = expected;
  const descriptor = fs.openSync(`/proc/${pid}/exe`, "r");
  try {
    const current = fs.fstatSync(descriptor, { bigint: true });
    assert.deepEqual({ dev: String(current.dev), ino: String(current.ino), bytes: Number(current.size), mtimeNs: String(current.mtimeNs), ctimeNs: String(current.ctimeNs) }, expectedMetadata);
  } finally { fs.closeSync(descriptor); }
  assert.deepEqual(metadata(`/proc/${pid}/exe`), expectedMetadata); checkImage(expected);
  const readNamespaces = (): ObjectValue => {
    const result: ObjectValue = {};
    for (const name of ["pid", "mnt", "user", "time"]) { const file = `/proc/${pid}/ns/${name}`, s = fs.statSync(file, { bigint: true }); result[name] = { link: fs.readlinkSync(file), dev: String(s.dev), ino: String(s.ino) }; }
    return result;
  };
  const namespaces = readNamespaces(), after = stat(); assert.equal(before[1], after[1]); assert.equal(before[19], after[19]);
  assert.equal(fs.readlinkSync(`/proc/${pid}/exe`), executable); assert.deepEqual(metadata(`/proc/${pid}/exe`), expectedMetadata); assert.deepEqual(readNamespaces(), namespaces);
  const birthTicks = before[19], parentPid = before[1];
  assert(birthTicks !== undefined && parentPid !== undefined && /^[1-9][0-9]*$/.test(birthTicks));
  return { birthTicks, parentPid, executable, namespaces, bootIdSha256: sha(Buffer.from(small("/proc/sys/kernel/random/boot_id"))) };
}
export class RawCapture implements CaptureHandle {
  private readonly owner: Omit<RawOwner, "rawSha256" | "rawBytes" | "eof" | "closed" | "truncated" | "errors">;
  private readonly stream: Readable;
  private readonly child: ChildProcess;
  private readonly evidence: unknown;
  private readonly rawFd: number;
  private readonly hash = createHash("sha256");
  private saved = 0; private received = 0; private truncated = false; private eof = false; private closed = false;
  private sealed = false; private writeFailed = false; private code: number | null = null; private signal: NodeJS.Signals | null = null;
  private readonly errors: string[] = []; private callbackNs = 0n; private readonly startedNs = process.hrtime.bigint();
  private readonly onData: (chunk: unknown) => void;
  private readonly onEnd: () => void;
  private readonly onError: (error: Error) => void;
  private readonly onClose: (code: number | null, signal: NodeJS.Signals | null) => void;
  constructor(privateOutput: string, owner: Omit<RawOwner, "rawSha256" | "rawBytes" | "eof" | "closed" | "truncated" | "errors">,
    stream: Readable, child: ChildProcess, evidence: unknown, earlyTail: string) {
    this.owner = owner; this.stream = stream; this.child = child; this.evidence = evidence;
    this.rawFd = fs.openSync(privateOutput + ".raw", "wx", 0o600);
    this.manifest = privateOutput + ".json";
    if (earlyTail !== "") this.errors.push("stderr tail was nonempty at attachment; lost prefix cannot be repaired");
    this.onData = chunk => { const start = process.hrtime.bigint(); try {
      if (!Buffer.isBuffer(chunk)) throw new Error("non-Buffer raw chunk"); this.received += chunk.length;
      const kept = chunk.subarray(0, Math.max(0, RAW_LIMIT - this.saved)); if (kept.length < chunk.length) this.truncated = true;
      if (!this.sealed && !this.writeFailed) { let offset = 0; while (offset < kept.length) { const count = fs.writeSync(this.rawFd, kept, offset); if (count <= 0) throw new Error("raw write made no progress"); this.hash.update(kept.subarray(offset, offset + count)); this.saved += count; offset += count; } }
    } catch (error: unknown) { this.writeFailed = true; this.failure(error instanceof Error ? error.message : "raw write failed"); } finally { this.callbackNs += process.hrtime.bigint() - start; } };
    this.onEnd = () => { this.eof = true; };
    this.onError = error => { this.failure(`stderr/child:${error.message}`); };
    this.onClose = (code, signal) => { this.closed = true; this.code = code; this.signal = signal; this.seal(); this.dispose(); };
    stream.on("data", this.onData); stream.on("end", this.onEnd); stream.on("error", this.onError);
    child.on("close", this.onClose); child.on("error", this.onError);
  }
  private readonly manifest: string;
  private failure(reason: string): void { if (this.errors.length < 8) this.errors.push(reason.slice(0, 256)); }
  dispose(): void { this.stream.off("data", this.onData); this.stream.off("end", this.onEnd); this.stream.off("error", this.onError); this.child.off("close", this.onClose); this.child.off("error", this.onError); }
  seal(): void {
    if (this.sealed) return; this.sealed = true;
    try { fs.closeSync(this.rawFd); } catch { this.failure("raw close failed"); }
    const owner: RawOwner = { ...this.owner, rawSha256: this.hash.digest("hex"), rawBytes: this.saved, eof: this.eof, closed: this.closed, truncated: this.truncated, errors: [...this.errors] };
    try { write(this.manifest, { schema: "vize.navigation-raw", version: 1, qualificationCredit: 0, owner, evidence: this.evidence,
      receivedBytes: this.received, startedNs: this.startedNs.toString(), endedNs: process.hrtime.bigint().toString(), callbackNs: this.callbackNs.toString(), code: this.code, signal: this.signal,
      semantics: "One owned child stderr pipe; capture overhead included. No RPC, pause, spawn substitution, drain or retry. close is not a successful native operation." }, 8192); }
    catch { process.stderr.write("navigation raw manifest incomplete\n"); }
  }
}
async function preload(config: Config): Promise<void> {
  const benchmark = path.join(config.repoRoot, "tests/performance/lsp-dual-server-benchmark.ts");
  if (process.argv[1] === undefined || path.resolve(process.argv[1]) !== benchmark) return;
  const modulePath = path.join(config.repoRoot, "tests/performance/support/dual-lsp-wire.ts");
  assert.equal(sha(fs.readFileSync(modulePath)), config.wireSourceSha256); checkImage(config.serverImage); checkImage(config.nodeImage);
  assert.equal(fs.realpathSync(process.execPath), config.nodeImage.realpath); assert.equal(process.version, "v24.14.0");
  const module: unknown = await import(pathToFileURL(modulePath).href), constructor = property(module, "DualLspWire"), prototype = property(constructor, "prototype"); assert(prototype !== null && typeof prototype === "object");
  let serial = 0; const errors: string[] = [], seen = new WeakSet<object>();
  const uninstall = instrumentWirePrototype(prototype, wire => {
    assert(wire !== null && typeof wire === "object" && !seen.has(wire)); seen.add(wire); serial += 1; assert(serial <= 2, "two original Vize process sessions only");
    const launch = object(property(wire, "launch")); assert.equal(fs.realpathSync(string(launch["command"])), config.serverImage.realpath);
    const args = launch["args"]; assert(Array.isArray(args) && args[0] === "lsp");
    assert(args.length === 1 || (args.length === 3 && args[1] === "--profile-json" && typeof args[2] === "string"));
    const child = property(wire, "child"); assert(child instanceof ChildProcess && child.stderr instanceof Readable && child.pid !== undefined);
    const pid = child.pid, proof = anchor(pid, config.serverImage), basename = `session-${serial}-pid-${pid}-birth-${proof.birthTicks}`;
    return new RawCapture(path.join(config.outputRoot, basename), { instance: `${config.runId}:${config.runAttempt}:${serial}:${pid}:${proof.birthTicks}`, pid, birthTicks: proof.birthTicks, sourceRevision: config.sourceRevision, imageSha256: config.serverImage.sha256 }, child.stderr, child,
      { workflowRevision: config.workflowRevision, launchCommand: config.serverImage.realpath, argumentKinds: args.length === 1 ? ["lsp"] : ["lsp", "--profile-json", "owned-output-path"], anchor: proof,
        helperSha256: sha(fs.readFileSync(fileURLToPath(import.meta.url))), wireSourceSha256: config.wireSourceSha256 }, string(property(wire, "stderrText")));
  }, reason => { if (errors.length < 8) errors.push(reason.slice(0, 256)); });
  let done = false; const finish = (): void => {
    if (done) return; done = true;
    try { uninstall(); } catch { errors.push("raw uninstall incomplete"); }
    try { write(path.join(config.outputRoot, `preload-${process.pid}.json`), { schema: "vize.navigation-trace-preload", version: 1,
      sourceRevision: config.sourceRevision, workflowRevision: config.workflowRevision, sessionsAttempted: serial, errors, qualificationCredit: 0 }, 8192); }
    catch { /* Missing terminal manifest refuses diagnostic coverage; original result is untouched. */ }
  };
  process.once("beforeExit", finish); process.once("exit", finish);
}
export function originalArguments(output: string): string[] { return ["exec", "node", "tests/performance/lsp-dual-server-benchmark.ts", "--fresh", "1", "--warm", "5", "--out", output]; }
export function validatePreloadManifest(value: unknown, expected: { sourceRevision: string; workflowRevision: string }): void {
  const row = object(value); assert.equal(row["schema"], "vize.navigation-trace-preload"); assert.equal(row["version"], 1);
  assert.equal(row["sourceRevision"], expected.sourceRevision); assert.equal(row["workflowRevision"], expected.workflowRevision);
  assert.equal(row["sessionsAttempted"], 2); assert.equal(row["qualificationCredit"], 0);
  assert.deepEqual(row["errors"], [], "all attachment/finalization failures remain diagnostic failures");
}
export function validatePreloadTerminals(values: readonly unknown[], expected: { sourceRevision: string; workflowRevision: string }): void {
  assert.equal(values.length, 1, "one process-owned preload terminal required"); validatePreloadManifest(values[0], expected);
}
export function writeAll(fd: number, bytes: Buffer, writer: (fd: number, bytes: Buffer, offset: number, length: number) => number = (fd, bytes, offset, length) => fs.writeSync(fd, bytes, offset, length)): void {
  let offset = 0;
  while (offset < bytes.length) { const count = writer(fd, bytes, offset, bytes.length - offset); assert(Number.isSafeInteger(count) && count > 0 && count <= bytes.length - offset, "output write made no valid progress"); offset += count; }
}
export function expectedNavigationCounts(value: unknown): Record<string, number> {
  const report = object(value); assert.equal(report["schemaVersion"], 3); assert.equal(report["status"], "passed");
  const config = object(report["configuration"]); assert.equal(config["freshSamplesPerProduct"], 1); assert.equal(config["warmSamplesPerProduct"], 5);
  const receipts = object(object(report["wire"])["primaryOperationReceipts"]), result: Record<string, number> = { references: 0, prepare_rename: 0, rename: 0 };
  for (const [kind, packets] of [["fresh", 1], ["warm", 5]] as const) {
    const rows = receipts[kind]; assert(Array.isArray(rows) && rows.length === 1); const products = object(object(rows[0])["products"]), vize = object(products["vize"]);
    assert.deepEqual(vize, products["vue-language-tools"]); assert.equal(vize["queryPackets"], packets); const counts = object(vize["methodCounts"]);
    for (const [name, method] of [["references", "references"], ["prepare_rename", "prepareRename"], ["rename", "rename"]]) {
      assert(name !== undefined && method !== undefined); assert.equal(counts[`request textDocument/${method}`], packets); result[name] = (result[name] ?? 0) + packets;
    }
  }
  return result;
}
async function main(): Promise<void> {
  assert.equal(process.platform, "linux"); assert.equal(process.arch, "x64"); assert.equal(process.version, "v24.14.0");
  const configPath = env("VIZE_NAV_TRACE_CONFIG"), config = parseConfig(readJson(configPath));
  assert.equal(execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(), config.sourceRevision);
  for (const current of [config.serverImage, config.nodeImage]) { checkImage(current); assert.equal(sha(fs.readFileSync(current.realpath)), current.sha256); checkImage(current); }
  assert.equal(sha(fs.readFileSync(path.join(config.repoRoot, "tests/performance/lsp-dual-server-benchmark.ts"))), config.benchmarkSourceSha256);
  assert.equal(fs.readdirSync(config.outputRoot).length, 0); const output = path.join(path.dirname(config.outputRoot), "report"); assert(!fs.existsSync(output));
  const arguments_ = originalArguments(output), options = `${process.env["NODE_OPTIONS"] ?? ""} --import=${pathToFileURL(fileURLToPath(import.meta.url)).href}`;
  write(path.join(path.dirname(config.outputRoot), "invocation.json"), { executable: "vp", args: arguments_, source: config.sourceRevision, workflow: config.workflowRevision, traceFlag: true, preloadSha256: sha(fs.readFileSync(fileURLToPath(import.meta.url))), extraRpc: 0, qualificationCredit: 0 });
  const child = spawn("vp", arguments_, { cwd: config.repoRoot, env: { ...process.env, NODE_OPTIONS: options, VIZE_NAV_TRACE_PRELOAD: "diagnostic-only-v1", VIZE_NAV_TRACE_CONFIG: configPath, VIZE_TRACE_EDITOR_PREPARATION: "1", LSP_DUAL_ALLOW_SMOKE: "1" }, stdio: ["ignore", "pipe", "pipe"] });
  const stdout = fs.openSync(path.join(path.dirname(config.outputRoot), "original.stdout"), "wx", 0o600), stderr = fs.openSync(path.join(path.dirname(config.outputRoot), "original.stderr"), "wx", 0o600); let ioError: string | null = null;
  const sizes = { stdout: 0, stderr: 0 }; for (const [name, stream, fd] of [["stdout", child.stdout, stdout], ["stderr", child.stderr, stderr]] as const) {
    stream.on("data", (chunk: Buffer) => { try { sizes[name] += chunk.length; if (sizes[name] > 16 * 1024 * 1024) { ioError = "original output cap"; return; } writeAll(fd, chunk); } catch { ioError = "original output writer failed"; } });
  }
  const exit = await new Promise<{ code: number | null; signal: NodeJS.Signals | null; error: string | null }>(resolve => {
    child.once("error", error => resolve({ code: null, signal: null, error: error.message })); child.once("close", (code, signal) => resolve({ code, signal, error: ioError }));
  }); fs.closeSync(stdout); fs.closeSync(stderr);
  const joins = []; const rawErrors: string[] = []; let expected: Record<string, number> | null = null;
  try { const report = readJson(path.join(output, "report.json")); assert.equal(object(report)["commit"], config.sourceRevision); expected = expectedNavigationCounts(report); }
  catch (error: unknown) { rawErrors.push(error instanceof Error ? error.message : "original report incomplete"); }
  const names = fs.readdirSync(config.outputRoot); assert(names.length <= 5, "two raw/manifest pairs and one preload manifest");
  try { const terminal = names.filter(name => /^preload-[0-9]+\.json$/.test(name)); validatePreloadTerminals(terminal.map(name => readJson(path.join(config.outputRoot, name))), config); }
  catch (error: unknown) { rawErrors.push(error instanceof Error ? error.message : "preload terminal authority incomplete"); }
  for (const name of names.filter(name => /^session-.*\.json$/.test(name))) {
    try { const manifest = object(readJson(path.join(config.outputRoot, name))); assert.equal(manifest["schema"], "vize.navigation-raw"); assert.equal(manifest["version"], 1); assert.equal(object(manifest["evidence"])["workflowRevision"], config.workflowRevision);
      const owner = object(manifest["owner"]), rawPath = path.join(config.outputRoot, name.replace(/\.json$/, ".raw")), info = fs.lstatSync(rawPath); assert(info.isFile() && !info.isSymbolicLink() && info.size <= RAW_LIMIT); const raw = fs.readFileSync(rawPath);
      const typed: RawOwner = { instance: string(owner["instance"]), pid: Number(owner["pid"]), birthTicks: string(owner["birthTicks"]), sourceRevision: string(owner["sourceRevision"]), imageSha256: string(owner["imageSha256"]), rawSha256: string(owner["rawSha256"]), rawBytes: Number(owner["rawBytes"]), eof: owner["eof"] === true, closed: owner["closed"] === true, truncated: owner["truncated"] === true, errors: Array.isArray(owner["errors"]) ? owner["errors"].map(string) : ["invalid error list"] };
      assert.equal(typed.sourceRevision, config.sourceRevision); assert.equal(typed.imageSha256, config.serverImage.sha256); const join = joinRaw(raw, typed); joins.push(join); write(path.join(path.dirname(config.outputRoot), name.replace(/\.json$/, ".join.json")), join);
    } catch (error: unknown) { rawErrors.push(error instanceof Error ? error.message : "raw manifest incomplete"); }
  }
  const counts: Record<string, number> = {}; for (const join of joins) for (const request of join.requests) if (request.complete) counts[request.method] = (counts[request.method] ?? 0) + 1;
  try { assert.equal(joins.length, 2); assert.deepEqual(counts, expected); assert(joins.every(join => join.captureComplete && join.requests.every(request => request.complete))); }
  catch { rawErrors.push("owned raw/request coverage incomplete; all raw and unbound rows retained"); }
  for (const current of [config.serverImage, config.nodeImage]) { checkImage(current); assert.equal(sha(fs.readFileSync(current.realpath)), current.sha256); }
  write(path.join(path.dirname(config.outputRoot), "campaign.json"), { source: config.sourceRevision, workflow: config.workflowRevision, exit, expected, observed: counts, rawErrors, sessions: joins.map(join => ({ owner: join.owner, requests: join.requests.length, unboundRows: join.unboundRows, attributionComplete: join.attributionComplete })), qualificationCredit: 0, semantics: "Diagnostic profiling/tracing only; original whole oracle/exit retained. Inclusive elapsed phases are not exclusive CPU, allocation or ordinary performance qualification." });
  assert.equal(exit.code, 0, "original same-wire command failed; raw retained"); assert.equal(exit.signal, null); assert.equal(exit.error, null); assert.equal(rawErrors.length, 0, "diagnostic coverage failed; raw retained");
}
if (process.env["VIZE_NAV_TRACE_PRELOAD"] === "diagnostic-only-v1") {
  try { await preload(parseConfig(readJson(env("VIZE_NAV_TRACE_CONFIG")))); } catch { process.stderr.write("navigation trace preload incomplete\n"); }
} else if (process.argv[1] !== undefined && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
