import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { execFileSync, spawn } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import type { FileIdentity, ImageIdentity, ObservationConfig } from "./types.ts";

type JsonRecord = Record<string, unknown>;
type OriginalTask = "test:performance:lsp-incremental" | "test:performance:lsp-churn";
type Exit = { code: number | null; signal: string | null; controllerError: string | null };
export const source = "68eb9e29cb2b3df14ca5a9cfe7fd3f92095cabbb";
const launcherSource = "tools/support/compat/davinci/lib/lsp-profile-launch.ts";
const snapshotFields = ["read_before_ns", "read_after_ns", "alloc_calls", "alloc_zeroed_calls", "alloc_failures", "alloc_zeroed_failures", "alloc_bytes", "alloc_zeroed_bytes", "dealloc_calls", "dealloc_bytes", "realloc_calls", "realloc_failures", "realloc_old_bytes", "realloc_new_bytes"];
const regions = ["worker", "descriptor", "script", "style", "contract"];
type ProfileLauncher = { create: (server: string, workspace: string, profile: string) => unknown; sourceSha256: string };
const sha = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
function record(value: unknown): JsonRecord {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return Object.fromEntries(Object.entries(value));
}
function text(value: JsonRecord, key: string): string {
  const result = value[key]; assert(typeof result === "string", key); return result;
}
function read(file: string): JsonRecord {
  const parsed: unknown = JSON.parse(fs.readFileSync(file, "utf8")); return record(parsed);
}
function env(key: string): string {
  const value = process.env[key]; assert(typeof value === "string" && value.length > 0, key); return value;
}
function write(file: string, value: unknown): void {
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`, { flag: "wx" });
}
function git(...args: string[]): string {
  return execFileSync("git", args, { encoding: "utf8" }).trim();
}
function metadata(file: string): FileIdentity {
  const value = fs.statSync(file, { bigint: true });
  assert(value.isFile()); const bytes = Number(value.size); assert(Number.isSafeInteger(bytes) && bytes > 0);
  return { dev: value.dev.toString(), ino: value.ino.toString(), bytes,
    mtimeNs: value.mtimeNs.toString(), ctimeNs: value.ctimeNs.toString() };
}
function unchangedImage(value: FileIdentity & { realpath: string }): void {
  assert.deepEqual(metadata(value.realpath), { dev: value.dev, ino: value.ino, bytes: value.bytes,
    mtimeNs: value.mtimeNs, ctimeNs: value.ctimeNs }, "image metadata changed during original workload");
}
function identity(value: JsonRecord): FileIdentity & { realpath: string; sha256: string } {
  const file = text(value, "realpath"); assert(path.isAbsolute(file));
  const expected = text(value, "sha256"); assert.match(expected, /^[0-9a-f]{64}$/);
  const before = metadata(file); const actual = sha(fs.readFileSync(file)); const after = metadata(file);
  assert.deepEqual(before, after, "image metadata must stay stable while hashing");
  assert.equal(actual, expected, "preflight must hash the actual reviewed image");
  assert.equal(value["bytes"], before.bytes);
  return { ...before, realpath: file, sha256: actual };
}
export function originalArguments(task: OriginalTask): string[] {
  return ["run", "--no-cache", "--filter", "./tests", task];
}
export function validateChurn(value: unknown): JsonRecord {
  const packet = record(value); assert.equal(packet["commit"], source);
  const budget = record(packet["budget"]);
  assert.equal(budget["scale"], 1); assert.equal(budget["maxPeakRssMiB"], 136);
  assert.equal(budget["cyclesPerPhase"], 20);
  assert(packet["status"] === "passed" || packet["status"] === "failed");
  const timings = record(packet["timingsMs"]);
  assert(Array.isArray(timings["cycle"]) && timings["cycle"].length === 40);
  assert(Array.isArray(timings["fileLifecycle"]) && timings["fileLifecycle"].length === 20);
  assert.equal(record(packet["cycleStats"])["cycles"], 40);
  const rss = packet["rssSamples"]; assert(Array.isArray(rss));
  assert.deepEqual(rss.map((sample: unknown) => record(sample)["label"]), ["coldOpen",
    ...Array.from({ length: 20 }, (_, index: number) => `A${index}`),
    ...Array.from({ length: 20 }, (_, index: number) => `B${index}`), "cancellation", "closed",
    ...Array.from({ length: 20 }, (_, index: number) => `file-lifecycle-${index}`)]);
  return { complete: true, originalStatus: packet["status"], peakRssKiB: packet["peakRssKiB"],
    peakProcessTreeRssKiB: packet["peakProcessTreeRssKiB"], qualificationCredit: 0 };
}
export function prepareCommandDirectory(directory: string, observed: boolean): void {
  fs.mkdirSync(directory, { mode: 0o700 });
  if (observed) for (const name of ["proc", "profile"]) fs.mkdirSync(path.join(directory, name), { mode: 0o700 });
}
function unsigned(value: unknown): number {
  assert(typeof value === "number" && Number.isSafeInteger(value) && value >= 0, "expected nonnegative safe integer"); return value;
}
function values(value: unknown): unknown[] { assert(Array.isArray(value)); return value; }
function sum(...items: number[]): number { return unsigned(items.reduce((total, item) => total + item, 0)); }
export function validateProfile(value: unknown): JsonRecord {
  const report = record(value); assert.equal(report["schema_version"], 1); assert.equal(report["command"], "lsp");
  assert.deepEqual(report["truncation"], { dropped_spans: 0, dropped_counters: 0 });
  const expectedCounters = [1, 2].flatMap((scan) => ["before", "after"].flatMap((edge) => snapshotFields.map((field) => `diagnostic.workspace_symbols.request${scan}.${edge}.${field}`)));
  const snapshots = new Map<string, number>();
  for (const row of values(report["counters"]).map(record)) {
    const key = text(row, "key"); if (!key.startsWith("diagnostic.workspace_symbols.")) continue;
    assert(expectedCounters.includes(key) && !snapshots.has(key), "foreign or duplicate diagnostic counter");
    assert.equal(row["samples"], 1, "each diagnostic counter must have one sample");
    const total = unsigned(row["total"]); assert.equal(row["min"], total); assert.equal(row["max"], total); snapshots.set(key, total);
  }
  assert.equal(snapshots.size, 56, "all four fourteen-field snapshots are required");
  const get = (scan: number, edge: string, field: string): number => {
    const result = snapshots.get(`diagnostic.workspace_symbols.request${scan}.${edge}.${field}`); assert(result !== undefined); return result;
  };
  let previousEnd = 0; const previous = new Map<string, number>();
  for (const scan of [1, 2]) for (const edge of ["before", "after"]) {
    const start = get(scan, edge, "read_before_ns"); const end = get(scan, edge, "read_after_ns");
    assert(start >= previousEnd && end >= start, "snapshot read intervals must be ordered"); previousEnd = end;
    for (const field of snapshotFields.slice(2)) {
      const current = get(scan, edge, field); assert(current >= (previous.get(field) ?? 0), "global operation counter regressed"); previous.set(field, current);
    }
  }
  const differences = [1, 2].map((scan) => {
    const delta = Object.fromEntries(snapshotFields.slice(2).map((field) => [field, get(scan, "after", field) - get(scan, "before", field)]));
    const number = (field: string): number => unsigned(delta[field]);
    const requestedBytes = sum(number("alloc_bytes"), number("alloc_zeroed_bytes"), number("realloc_new_bytes"));
    const releasedBytes = sum(number("dealloc_bytes"), number("realloc_old_bytes"));
    return { instrumentedServiceInvocation: scan, beforeReadNs: [get(scan, "before", "read_before_ns"), get(scan, "before", "read_after_ns")],
      afterReadNs: [get(scan, "after", "read_before_ns"), get(scan, "after", "read_after_ns")], operationsDelta: delta,
      requestedAllocLikeCalls: sum(number("alloc_calls"), number("alloc_zeroed_calls"), number("realloc_calls")),
      requestedBytes, releasedBytes, netRequestedMinusReleasedBytes: requestedBytes - releasedBytes };
  });
  const expectedSpans = [1, 2].flatMap((scan) => regions.map((region) => `diagnostic.workspace_symbols.scan${scan}.${region}`));
  const spans: JsonRecord[] = []; const seen = new Set<string>();
  for (const row of values(report["spans"]).map(record)) {
    const key = text(row, "key"); if (!key.startsWith("diagnostic.workspace_symbols.")) continue;
    assert(expectedSpans.includes(key) && !seen.has(key), "foreign or duplicate diagnostic span"); seen.add(key);
    const count = unsigned(row["count"]); assert(count > 0, "diagnostic span must execute");
    const allocation = record(row["alloc"]); const wall = record(row["wall_ns"]);
    for (const field of ["calls", "bytes", "self_calls", "self_bytes"]) unsigned(allocation[field]);
    for (const field of ["total", "self", "min", "max", "p50", "p95", "p99"]) unsigned(wall[field]);
    assert(unsigned(allocation["self_calls"]) <= unsigned(allocation["calls"]) && unsigned(allocation["self_bytes"]) <= unsigned(allocation["bytes"]));
    assert(unsigned(wall["self"]) <= unsigned(wall["total"]) && unsigned(wall["min"]) <= unsigned(wall["max"]));
    if (key.endsWith(".worker")) {
      assert.equal(count, 1, "each original workspace request must have exactly one instrumented worker pass");
      assert(unsigned(allocation["calls"]) > 0 && unsigned(allocation["bytes"]) > 0, "worker allocation instrumentation is required");
    }
    spans.push({ key, count, wall_ns: wall, alloc: allocation });
  }
  assert.equal(spans.length, 10, "both complete five-region span sets are required");
  return { complete: true, qualificationCredit: 0, snapshots: Object.fromEntries(snapshots), differences, spans,
    limits: "Process-wide relaxed snapshots include concurrent Rust work and are non-atomic. Net requested-minus-released is arithmetic, not live heap or RSS. Rust read times use a private Instant origin, not Node hrtime. Spans count requested allocation-like volume on the current worker thread only; script/style are Vue-block regions, direct TS/JS remains worker-exclusive. After snapshots retain lexical result/open/path/filter owners. Source-verified single-service routing plus one-worker-pass admission maps these first two invocations to this original workload's first two workspace RPCs, not exact cross-clock alignment." };
}
export function profileLaunchMetadata(value: unknown): JsonRecord {
  const launch = record(value); const directory = text(launch, "directory"); const server = text(launch, "server");
  const wrapper = text(launch, "wrapper"); const profile = text(launch, "profile");
  for (const file of [directory, server, wrapper, profile]) assert(path.isAbsolute(file) && !file.includes("\0"));
  assert.equal(wrapper, path.join(directory, ".vize-profile-lsp.sh")); assert.equal(profile, path.join(directory, "profile", "lsp.json"));
  assert.notEqual(server, wrapper); assert.notEqual(server, profile);
  for (const key of ["helperSha256", "wrapperSha256"]) assert.match(text(launch, key), /^[0-9a-f]{64}$/);
  assert.equal(launch["helperSource"], launcherSource);
  assert.deepEqual(launch["addedArguments"], ["--profile-json", profile]);
  return { ...launch, measuredServerImage: server, execPreservesServerPid: "existing reviewed launcher contract; proc image custody checks actual process", profilingInstrumentation: true, qualificationCredit: 0 };
}
async function execute(directory: string, task: OriginalTask, observed: boolean, config: string, server: string, launcher: ProfileLauncher): Promise<Exit> {
  prepareCommandDirectory(directory, observed);
  const beforeNs = process.hrtime.bigint(); const baseOptions = process.env["NODE_OPTIONS"] ?? "";
  const options = observed ? `${baseOptions} --import=${pathToFileURL(path.join(path.dirname(config), "preload.ts")).href}` : baseOptions;
  const childEnvironment: NodeJS.ProcessEnv = { ...process.env, NODE_OPTIONS: options };
  delete childEnvironment["VIZE_LSP_CURRENT_BASELINE"];
  if (observed) {
    childEnvironment["VIZE_LSP_LIFECYCLE_OBSERVATION"] = "diagnostic-only-v1";
    childEnvironment["VIZE_LSP_LIFECYCLE_CONFIG"] = config;
  } else {
    delete childEnvironment["VIZE_LSP_LIFECYCLE_OBSERVATION"];
    delete childEnvironment["VIZE_LSP_LIFECYCLE_CONFIG"];
  }
  let profileLaunch: JsonRecord | null = null;
  if (observed) {
    const profile = path.join(directory, "profile", "lsp.json"); assert(!fs.existsSync(profile));
    const wrapper: unknown = launcher.create(server, directory, profile); assert(typeof wrapper === "string");
    assert(fs.lstatSync(wrapper).isFile() && !fs.lstatSync(wrapper).isSymbolicLink());
    profileLaunch = profileLaunchMetadata({ directory, server, wrapper, profile, helperSource: launcherSource,
      helperSha256: launcher.sourceSha256, wrapperSha256: sha(fs.readFileSync(wrapper)), addedArguments: ["--profile-json", profile] });
    childEnvironment["VIZE_LSP_BIN"] = wrapper;
  }
  write(path.join(directory, "invocation.json"), { executable: "vp", args: originalArguments(task), observed,
    source, existingWireCapture: false, plannedExtraLspRequests: 0, originalBudgetPolicy: "unchanged source assertion and scale1",
    originalNodeOptionsSha256: sha(Buffer.from(baseOptions)), nodeOptionsSha256: sha(Buffer.from(options)),
    profileLaunch, qualificationCredit: 0, startedNs: beforeNs.toString() });
  const child = spawn("vp", originalArguments(task), { cwd: process.cwd(), env: childEnvironment,
    detached: true, stdio: ["ignore", "pipe", "pipe"] });
  let controllerError: string | null = null; let bytes = 0;
  const terminate = (): void => {
    if (child.pid !== undefined) { try { process.kill(-child.pid, "SIGKILL"); } catch { /* Already exited. */ } }
  };
  const deadline = setTimeout(() => { controllerError = "original process cleanup deadline350s"; terminate(); }, 350_000);
  for (const stream of ["stdout", "stderr"] as const) {
    const file = path.join(directory, `${stream}.txt`); fs.writeFileSync(file, "", { flag: "wx" });
    child[stream].on("data", (chunk: Buffer) => {
      bytes += chunk.length;
      if (bytes > 16 * 1024 * 1024) { controllerError ??= "combined output16MiB cap"; terminate(); }
      else {
        try { fs.appendFileSync(file, chunk); }
        catch (error: unknown) { controllerError ??= error instanceof Error ? error.message : String(error); terminate(); }
      }
    });
  }
  child.on("error", (error: Error) => { controllerError = error.message; });
  const result = await new Promise<{ code: number | null; signal: string | null }>((resolve) =>
    child.once("close", (code, signal) => resolve({ code, signal })));
  clearTimeout(deadline);
  const exit: Exit = { ...result, controllerError };
  write(path.join(directory, "exit.json"), { ...exit, endedNs: process.hrtime.bigint().toString(),
    wallMs: Number(process.hrtime.bigint() - beforeNs) / 1e6, outputBytes: bytes });
  for (const [relative, name] of [["target/vize-tests/metrics", "metrics"]]) {
    if (relative !== undefined && name !== undefined && fs.existsSync(relative)) fs.renameSync(relative, path.join(directory, name));
  }
  return exit;
}
async function main(): Promise<void> {
  assert.equal(process.platform, "linux"); assert.equal(process.arch, "x64"); assert.equal(process.version, "v24.14.0");
  assert.equal(env("SOURCE_SHA"), source); assert.equal(git("rev-parse", "HEAD"), source);
  assert.equal(env("VIZE_PERF_BUDGET_SCALE"), "1"); assert.equal(env("RAYON_NUM_THREADS"), "4");
  for (const key of ["VIZE_LSP_CURRENT_BASELINE", "VIZE_LSP_LIFECYCLE_OBSERVATION", "VIZE_LSP_LIFECYCLE_CONFIG", "VIZE_LSP_REQUIRE_SOURCE_BUILD", "VIZE_LSP_BUILD_RECIPE"]) assert(!process.env[key], key);
  const output = fs.realpathSync(env("EVIDENCE")); assert(path.isAbsolute(output));
  const custody = read(path.join(output, "before.json")); assert.equal(custody["source"], source);
  const server: ImageIdentity = { ...identity(record(custody["binary"])), role: "server" };
  const native: ImageIdentity = { ...identity(record(custody["nativeImage"])), role: "native" };
  const node = identity(record(record(custody["vp"])["selectedNode"]));
  const launcherFile = path.join(process.cwd(), launcherSource); const launcherSha256 = sha(fs.readFileSync(launcherFile));
  const launcherEntries = values(custody["harness"]).map(record).filter((entry) => entry["path"] === launcherSource);
  assert.equal(launcherEntries.length, 1); assert.equal(launcherEntries[0]?.["sha256"], launcherSha256);
  const imported: unknown = await import(pathToFileURL(launcherFile).href); const create: unknown = record(imported)["profiledLauncher"];
  assert(typeof create === "function");
  const launcher: ProfileLauncher = { create: (server, directory, profile): unknown => Reflect.apply(create, undefined, [server, directory, profile]), sourceSha256: launcherSha256 };
  const configFile = path.join(output, "observation-config.json");
  const config: ObservationConfig = { schema: "vize.lsp.lifecycle-observation", version: 1,
    repoRoot: fs.realpathSync(process.cwd()), sourceRevision: source, workflowRevision: env("WORKFLOW_SHA"),
    outputRoot: path.join(output, "observed", "proc"), images: [server, native], nodeImage: node,
    sessionSourceSha256: sha(fs.readFileSync("tests/tooling/support/lsp/session.ts")) };
  write(configFile, config);
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  assert(!fs.existsSync("target/vize-tests/metrics"), "fresh measurement outputs must be absent");
  assert(!fs.existsSync("target/differential/lsp-sessions"), "fresh wire outputs must be absent");
  const results: Array<{ label: string; exit: Exit; rawChurn: JsonRecord | null; profile: JsonRecord | null; errors: string[] }> = [];
  for (const [label, task, observed] of [
    ["incremental", "test:performance:lsp-incremental", false],
    ["observed", "test:performance:lsp-churn", true],
  ] satisfies Array<[string, OriginalTask, boolean]>) {
    const directory = path.join(output, label); const exit = await execute(directory, task, observed, configFile, server.realpath, launcher);
    const errors: string[] = []; let rawChurn: JsonRecord | null = null; let profile: JsonRecord | null = null;
    if (task === "test:performance:lsp-churn") {
      try { rawChurn = validateChurn(read(path.join(directory, "metrics/misskey-lsp-churn/metrics.json"))); }
      catch (error: unknown) { errors.push(error instanceof Error ? error.message : String(error)); }
      const profileFile = path.join(directory, "profile", "lsp.json");
      profile = { complete: false, qualificationCredit: 0, path: profileFile };
      try {
        const file = profileFile; const info = fs.lstatSync(file);
        assert(info.isFile() && !info.isSymbolicLink() && info.size > 0 && info.size <= 16 * 1024 * 1024);
        const bytes = fs.readFileSync(file); profile = { ...profile, bytes: bytes.length, sha256: sha(bytes) };
        const value: unknown = JSON.parse(bytes.toString("utf8")); profile = { ...profile, ...validateProfile(value) };
      } catch (error: unknown) {
        const message = error instanceof Error ? error.message : String(error); profile = { ...profile, error: message }; errors.push(`profile: ${message}`);
      }
    }
    results.push({ label, exit, rawChurn, profile, errors });
    assert.equal(git("rev-parse", "HEAD"), source);
    assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
    unchangedImage(server); unchangedImage(native); unchangedImage(node);
  }
  const observations: Array<{ file: string; bytes: number; sha256: string; summary: JsonRecord | null }> = [];
  if (fs.existsSync(config.outputRoot)) {
    for (const file of fs.readdirSync(config.outputRoot).sort()) {
      assert.match(file, /^[A-Za-z0-9_.-]+$/);
      const full = path.join(config.outputRoot, file); const info = fs.lstatSync(full);
      assert(info.isFile() && !info.isSymbolicLink() && info.size <= 8 * 1024 * 1024);
      const bytes = fs.readFileSync(full);
      const summary = file.endsWith(".summary.json") ? record(JSON.parse(bytes.toString("utf8"))) : null;
      observations.push({ file, bytes: bytes.length, sha256: sha(bytes), summary });
    }
  }
  const summaries = observations.flatMap((item) => item.summary === null ? [] : [item.summary]);
  const coverage = summaries.length === 1 && summaries.every((summary) =>
    summary["schema"] === "vize.lsp.lifecycle-observation.summary" && summary["version"] === 1
    && summary["sourceRevision"] === source && summary["workflowRevision"] === env("WORKFLOW_SHA")
    && summary["qualificationCredit"] === 0 && summary["complete"] === true
    && summary["invalid"] === false && summary["inFlight"] === false && summary["truncated"] === false);
  // Observed output and all original failures are retained. A successful driver
  // is never a source qualification or a repaired136MiB budget assertion.
  const originalAllPassed = results.every((result) => result.exit.code === 0 && result.exit.signal === null && result.exit.controllerError === null
    && (result.label !== "observed" || result.rawChurn?.["originalStatus"] === "passed"));
  const profileComplete = results.filter((result) => result.label === "observed").every((result) => result.profile?.["complete"] === true);
  write(path.join(output, "campaign.json"), { schema: "vize.lsp.lifecycle-observation-campaign", version: 1,
    source, workflow: env("WORKFLOW_SHA"), qualificationCredit: 0, results,
    observationComplete: coverage, originalAllPassed, profileComplete, observations,
    limits: "Profiling instrumentation and client/proc observation perturb timing and allocation/residency.50ms snapshots can miss short-lived peaks/threads. No wire-RPC/Rust-clock alignment, per-thread RSS, net-live-heap or resource admission claim." });
  assert(coverage, "diagnostic observer coverage incomplete; original results remain retained");
  assert(originalAllPassed, "original command gate failed; diagnostic telemetry remains retained with zero qualification credit");
  assert(profileComplete && results.every((result) => result.errors.length === 0), "diagnostic profile or original metrics incomplete; raw failures remain retained");
}
if (process.argv[1] !== undefined && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
