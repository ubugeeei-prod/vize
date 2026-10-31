import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { execFileSync, spawn } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import type { FileIdentity, ImageIdentity, ObservationConfig } from "./types.ts";
import { SMAPS_LIMITS } from "./smaps-snapshots.ts";

type JsonRecord = Record<string, unknown>;
type OriginalTask = "test:performance:lsp-incremental";
type Exit = { code: number | null; signal: string | null; controllerError: string | null };
const source = "c34d95586eb9459560c9be4990ff9ebdba3bde61";
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
export const VBEN_LANES = ["initialize", "coldOpen", "coldOpenSecondApp", "completion", "hover", "warmNoop", "leafBroken", "leafRepaired", "sharedBroken", "sharedBrokenSecondApp", "sharedRepaired", "sharedRepairedSecondApp", "warmNoopSecondApp", "cancellationConverge"];
export const MISSKEY_LANES = ["initialize", "coldOpen", "completion", "hover", "warmNoop", "leafBroken", "leafRepaired", "sharedBroken", "sharedRepaired"];
const laneBudgets = { initialize: 1000, coldOpen: 8000, coldOpenSecondApp: 1500, completion: 1000, hover: 1000, warmNoop: 1500, leafBroken: 1500, leafRepaired: 1500, sharedBroken: 3000, sharedBrokenSecondApp: 3000, sharedRepaired: 3000, sharedRepairedSecondApp: 3000, warmNoopSecondApp: 1500, cancellationConverge: 6000 };
export function validateIncremental(value: unknown, suite: "misskey" | "vben"): JsonRecord {
  const packet = record(value); assert.equal(packet["commit"], source);
  assert.equal(packet["schemaVersion"], 3);
  const lanes = suite === "vben" ? VBEN_LANES : MISSKEY_LANES;
  assert.equal(packet["fixture"], suite === "vben" ? "vue-vben-admin/apps+packages+playground" : "misskey/packages/frontend");
  assert.equal(packet["fixtureRevision"], suite === "vben" ? "748b182e440863711dc9624924e92440f5ddc4d3" : "810faa8e5db4bbf9d6d408a61df148559340ee99");
  assert.deepEqual(packet["budget"], { scale: 1, suite: `${suite}-lsp-incremental`, laneHardTimeoutMs: 60000,
    maxPeakRssMiB: 256, maxPeakProcessTreeRssMiB: suite === "vben" ? 512 : 384,
    maxProcessTreeSize: suite === "vben" ? 4 : 3,
    laneBudgetsMs: Object.fromEntries(lanes.map((lane) => [lane, Reflect.get(laneBudgets, lane)])) });
  assert(packet["status"] === "passed" || packet["status"] === "failed");
  const timings = record(packet["timingsMs"]);
  const rss = record(packet["rssSamplesKiB"]), trees = record(packet["processTreeSamples"]);
  assert.deepEqual(Object.keys(timings), lanes); assert.deepEqual(Object.keys(rss), lanes); assert.deepEqual(Object.keys(trees), lanes);
  let peakServer = 0, peakTree = 0, peakCount = 0; let peakLabel = "";
  for (const lane of lanes) {
    assert(typeof timings[lane] === "number" && Number.isFinite(timings[lane]) && timings[lane] >= 0);
    const rootRss = rss[lane]; assert(typeof rootRss === "number" && Number.isSafeInteger(rootRss) && rootRss > 0);
    peakServer = Math.max(peakServer, rootRss);
    const tree = record(trees[lane]); const members = tree["members"]; assert(Array.isArray(members) && members.length > 0);
    const membersRss = members.map((member: unknown) => { const n = record(member)["rssKiB"]; assert(typeof n === "number" && Number.isSafeInteger(n) && n >= 0); return n; });
    const sum = membersRss.reduce((a: number, b: number) => a + b, 0);
    assert.equal(tree["totalKiB"], sum); assert.equal(tree["processes"], members.length);
    peakCount = Math.max(peakCount, members.length);
    if (sum > peakTree) { peakTree = sum; peakLabel = lane; }
  }
  assert.equal(packet["sampledPeakRssKiB"], peakServer); assert.equal(packet["sampledPeakProcessTreeRssKiB"], peakTree);
  assert.equal(packet["sampledPeakProcessTreeSize"], peakCount);
  assert.deepEqual(packet["sampledPeakProcessTree"], { label: peakLabel, ...record(trees[peakLabel]), memberRssSumKiB: peakTree });
  return { complete: true, originalStatus: packet["status"], originalFailure: packet["failure"], lanes,
    sampledPeakRssKiB: peakServer, sampledPeakProcessTreeRssKiB: peakTree, sampledPeakProcessTreeSize: peakCount, qualificationCredit: 0,
    settlement: "Original assertion order is authoritative; a failed tree gate short-circuits later assertions. Complete lane data does not imply later gates executed." };
}
export function prepareCommandDirectory(directory: string, observed: boolean): void {
  fs.mkdirSync(directory, { mode: 0o700 });
  if (observed) fs.mkdirSync(path.join(directory, "proc"), { mode: 0o700 });
}
export function validateSmapsSummary(value: unknown, rootPid: unknown, rootBirth: unknown, workflow: string,
  selectedTarget: unknown, nativeImage: ImageIdentity, serverImage: ImageIdentity): JsonRecord {
  const summary = record(value);
  assert.equal(summary["schema"], "vize.lsp.smaps-snapshots"); assert.equal(summary["version"], 1);
  assert.equal(summary["diagnosticOnly"], true); assert.equal(summary["qualificationCredit"], 0);
  assert.equal(summary["sourceRevision"], source); assert.equal(summary["workflowRevision"], workflow);
  assert.equal(summary["mode"], "vben-native"); assert.equal(summary["rootPid"], rootPid);
  if (summary["rootBirthTicks"] === null) assert.equal(summary["incomplete"], true);
  else assert.equal(summary["rootBirthTicks"], rootBirth);
  const selected = record(selectedTarget), target = record(summary["samplingTarget"]), owner = record(summary["ownerRoot"]);
  assert.equal(target["pid"], selected["pid"]); assert.equal(target["selectedBirthTicks"], selected["birthTicks"]);
  if (target["birthTicks"] === null) assert.equal(summary["incomplete"], true);
  else assert.equal(target["birthTicks"], selected["birthTicks"]);
  assert.equal(selected["parentPid"], rootPid); assert.equal(selected["executableSha256"], nativeImage.sha256);
  assert.deepEqual(target["image"], nativeImage); assert.equal(nativeImage.role, "native");
  assert.equal(owner["pid"], rootPid); assert.equal(owner["birthTicks"], rootBirth);
  assert.deepEqual(owner["image"], serverImage); assert.equal(serverImage.role, "server");
  assert.deepEqual(summary["limits"], SMAPS_LIMITS);
  const slots = summary["slots"]; assert(Array.isArray(slots) && slots.length === 4);
  assert.deepEqual(slots.map((slot: unknown) => record(slot)["label"]),
    ["coldOpenSecondApp-settlement-trigger", "sharedBrokenSecondApp-settlement-trigger", "sharedRepaired-settlement-trigger", "sharedRepairedSecondApp-settlement-trigger"]);
  assert.deepEqual(slots.map((slot: unknown) => record(slot)["triggerPhase"]),
    ["vben-coldOpenSecondApp-settled", "vben-sharedBrokenSecondApp-settled", "vben-sharedRepaired-settled", "vben-sharedRepairedSecondApp-settled"]);
  const states = slots.map((slot: unknown) => record(slot)["state"]);
  assert(states.every((state: unknown) => typeof state === "string" && ["missing", "busy-gap", "refused", "complete"].includes(state)));
  assert.equal(summary["snapshotsValid"], states.filter((state) => state === "complete").length);
  assert.equal(summary["allSlotsAccounted"], states.every((state) => state !== "missing"));
  assert.equal(summary["allReadsComplete"], states.every((state) => state === "complete"));
  const contained = slots.filter((slot: unknown) => record(slot)["state"] === "complete" && record(slot)["phaseRelation"] === "contained").length;
  assert.equal(summary["phaseContained"], contained);
  for (const field of ["incomplete", "inFlight"]) assert.equal(typeof summary[field], "boolean");
  if (summary["inFlight"] === true || summary["allReadsComplete"] === false) assert.equal(summary["incomplete"], true);
  for (const [field, maximum] of [["bytesRead", SMAPS_LIMITS.readBytes], ["outputBytes", SMAPS_LIMITS.outputBytes], ["totalVmas", SMAPS_LIMITS.totalVmas]] as const) {
    const amount = summary[field]; assert(typeof amount === "number" && Number.isSafeInteger(amount) && amount >= 0 && amount <= maximum);
  }
  return summary;
}
async function execute(directory: string, task: OriginalTask, observed: boolean, config: string): Promise<Exit> {
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
  write(path.join(directory, "invocation.json"), { executable: "vp", args: originalArguments(task), observed,
    source, existingWireCapture: false, plannedExtraLspRequests: 0, originalBudgetPolicy: "unchanged source assertion and scale1",
    originalNodeOptionsSha256: sha(Buffer.from(baseOptions)), nodeOptionsSha256: sha(Buffer.from(options)),
    qualificationCredit: 0, startedNs: beforeNs.toString() });
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
  const configFile = path.join(output, "observation-config.json");
  const config: ObservationConfig = { schema: "vize.lsp.lifecycle-observation", version: 1,
    repoRoot: fs.realpathSync(process.cwd()), sourceRevision: source, workflowRevision: env("WORKFLOW_SHA"),
    outputRoot: path.join(output, "observed", "proc"), images: [server, native], nodeImage: node,
    sessionSourceSha256: sha(fs.readFileSync("tests/tooling/support/lsp/session.ts")),
    metricsSourceSha256: sha(fs.readFileSync("tests/performance/support/incremental-metrics.ts")) };
  write(configFile, config);
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  assert(!fs.existsSync("target/vize-tests/metrics"), "fresh measurement outputs must be absent");
  assert(!fs.existsSync("target/differential/lsp-sessions"), "fresh wire outputs must be absent");
  const results: Array<{ label: string; exit: Exit; metrics: Record<string, JsonRecord | null>; errors: string[] }> = [];
  for (const [label, task, observed] of [
    ["observed", "test:performance:lsp-incremental", true],
  ] satisfies Array<[string, OriginalTask, boolean]>) {
    const directory = path.join(output, label); const exit = await execute(directory, task, observed, configFile);
    const errors: string[] = []; const metrics: Record<string, JsonRecord | null> = { misskey: null, vben: null };
    for (const suite of ["misskey", "vben"] as const) {
      try { metrics[suite] = validateIncremental(read(path.join(directory, `metrics/${suite}-lsp-incremental/metrics.json`)), suite); }
      catch (error: unknown) { errors.push(error instanceof Error ? error.message : String(error)); }
    }
    results.push({ label, exit, metrics, errors });
    assert.equal(git("rev-parse", "HEAD"), source);
    assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
    unchangedImage(server); unchangedImage(native); unchangedImage(node);
  }
  const observations: Array<{ file: string; bytes: number; sha256: string; summary: JsonRecord | null }> = [];
  const vmaFiles: Array<{ directory: string; file: string; bytes: number; sha256: string }> = [];
  const smapsPackets: unknown[] = []; const smapsErrors: string[] = [];
  let procBytes = 0; let vmaBytes = 0; let vmaDirectories = 0;
  if (fs.existsSync(config.outputRoot)) {
    for (const file of fs.readdirSync(config.outputRoot).sort()) {
      assert.match(file, /^[A-Za-z0-9_.-]+$/);
      const full = path.join(config.outputRoot, file); const info = fs.lstatSync(full);
      if (info.isDirectory() && !info.isSymbolicLink()) {
        assert.match(file, /^vma-client-\d+-server-\d+$/); vmaDirectories += 1; assert(vmaDirectories <= 1);
        const names = fs.readdirSync(full).sort(); assert(names.length <= 24, "smaps file-count cap");
        for (const name of names) {
          assert.match(name, /^[A-Za-z0-9_.-]+$/);
          const filename = path.join(full, name); const entry = fs.lstatSync(filename);
          assert(entry.isFile() && !entry.isSymbolicLink() && entry.size <= SMAPS_LIMITS.outputBytes);
          vmaBytes += entry.size; assert(vmaBytes <= SMAPS_LIMITS.outputBytes);
          const bytes = fs.readFileSync(filename);
          vmaFiles.push({ directory: file, file: name, bytes: bytes.length, sha256: sha(bytes) });
          if (name === "summary.json") { const value: unknown = JSON.parse(bytes.toString("utf8")); smapsPackets.push(value); }
        }
        continue;
      }
      assert(info.isFile() && !info.isSymbolicLink() && info.size <= 8 * 1024 * 1024);
      procBytes += info.size; assert(procBytes <= 8 * 1024 * 1024);
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
  let smaps: JsonRecord | null = null;
  try {
    assert.equal(smapsPackets.length, 1, "one smaps terminal manifest required");
    smaps = validateSmapsSummary(smapsPackets[0], summaries[0]?.["rootPid"], summaries[0]?.["rootBirthTicks"], env("WORKFLOW_SHA"),
      record(summaries[0]?.["cohort"])["target"], native, server);
    assert.equal(smaps["outputBytes"], vmaBytes, "smaps manifest must include all actual output bytes");
  } catch (error: unknown) { smapsErrors.push(error instanceof Error ? error.message : String(error)); }
  // Observed output and all original failures are retained. A successful driver
  // is never a source qualification or a repaired512MiB Vben tree assertion.
  const originalAllPassed = results.every((result) => result.exit.code === 0 && result.exit.signal === null && result.exit.controllerError === null);
  const originalMetricsComplete = results.every((result) => result.errors.length === 0)
    && results.length === 1 && results.every((result) => result.metrics["misskey"] !== null && result.metrics["vben"] !== null);
  write(path.join(output, "campaign.json"), { schema: "vize.lsp.lifecycle-observation-campaign", version: 1,
    source, workflow: env("WORKFLOW_SHA"), qualificationCredit: 0, results,
    observationComplete: coverage, originalAllPassed, originalMetricsComplete, observations, smaps, smapsErrors, vmaFiles,
    procOutputBytes: procBytes, smapsOutputBytes: vmaBytes, combinedObservationOutputBytes: procBytes + vmaBytes,
    smapsCaptureComplete: smaps?.["allReadsComplete"] === true && smaps["incomplete"] === false,
    limits: "Passive client/proc/VMA reads alter timing; four attempts retain busy/refused/overlap/late gaps. Interval labels are not atomic closed points or Rust fences. Identity/layout-valid reads are not allocator ownership/live heap/per-thread RSS or source qualification.50ms sampling can miss short-lived peaks/cohorts; publications are not quiescence." });
  assert.equal(smapsErrors.length, 0, "smaps manifest authority incomplete; original results remain retained");
  assert(originalMetricsComplete, "original metric authority/completeness failed; raw results remain retained");
  assert(coverage, "diagnostic observer coverage incomplete; original results remain retained");
  assert(originalAllPassed, "original command gate failed; diagnostic telemetry remains retained with zero qualification credit");
}
if (process.argv[1] !== undefined && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
