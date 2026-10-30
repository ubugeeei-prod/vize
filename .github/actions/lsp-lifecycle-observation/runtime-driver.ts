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
const source = "90800904bcfbf3ad073ba95349852ee71ffb37e2";
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
  if (observed) fs.mkdirSync(path.join(directory, "proc"), { mode: 0o700 });
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
    sessionSourceSha256: sha(fs.readFileSync("tests/tooling/support/lsp/session.ts")) };
  write(configFile, config);
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  assert(!fs.existsSync("target/vize-tests/metrics"), "fresh measurement outputs must be absent");
  assert(!fs.existsSync("target/differential/lsp-sessions"), "fresh wire outputs must be absent");
  const results: Array<{ label: string; exit: Exit; rawChurn: JsonRecord | null; errors: string[] }> = [];
  for (const [label, task, observed] of [
    ["incremental", "test:performance:lsp-incremental", false],
    ["observed", "test:performance:lsp-churn", true],
  ] satisfies Array<[string, OriginalTask, boolean]>) {
    const directory = path.join(output, label); const exit = await execute(directory, task, observed, configFile);
    const errors: string[] = []; let rawChurn: JsonRecord | null = null;
    if (task === "test:performance:lsp-churn") {
      try { rawChurn = validateChurn(read(path.join(directory, "metrics/misskey-lsp-churn/metrics.json"))); }
      catch (error: unknown) { errors.push(error instanceof Error ? error.message : String(error)); }
    }
    results.push({ label, exit, rawChurn, errors });
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
  const originalAllPassed = results.every((result) => result.exit.code === 0 && result.exit.signal === null && result.exit.controllerError === null);
  write(path.join(output, "campaign.json"), { schema: "vize.lsp.lifecycle-observation-campaign", version: 1,
    source, workflow: env("WORKFLOW_SHA"), qualificationCredit: 0, results,
    observationComplete: coverage, originalAllPassed, observations,
    limits: "Client observer, proc reads and existing wire writes alter timing.50ms/asynchronous snapshots can miss short-lived peaks/threads. Publications are not quiescence; no per-thread RSS or allocator live-byte claim." });
  assert(coverage, "diagnostic observer coverage incomplete; original results remain retained");
  assert(originalAllPassed, "original command gate failed; diagnostic telemetry remains retained with zero qualification credit");
}
if (process.argv[1] !== undefined && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
