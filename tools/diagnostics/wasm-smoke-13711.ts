// Private-branch diagnostic. This builds source version 0.474.0 and publishes nothing.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

interface CommandResult {
  readonly status: number | null;
  readonly signal: NodeJS.Signals | null;
}
interface CommandReceipt {
  readonly command: string;
  readonly args: readonly string[];
  readonly startedAt: string;
  readonly log: string;
  status?: number | null;
  signal?: NodeJS.Signals | null;
  finishedAt?: string;
  logSha256?: string;
}
interface Receipt {
  readonly source: string;
  readonly startedAt: string;
  readonly diagnosticSha: string | undefined;
  readonly ref: string | undefined;
  readonly workflowRef: string | undefined;
  readonly workflowSha: string | undefined;
  readonly runId: string | undefined;
  readonly runAttempt: string | undefined;
  readonly node: string;
  readonly published: false;
  readonly purpose: string;
  readonly commands: CommandReceipt[];
  success: boolean;
  changedFiles?: string[];
  inputs?: Record<string, string>;
  workspaceVersion?: string;
  packageVersion?: unknown;
  rustc?: string;
  cargo?: string;
  wasmBindgen?: string;
  buildEnvironment?: Record<"VIZE_RELEASE_BUILD" | "CARGO_INCREMENTAL", string | undefined>;
  wasm?: { readonly sha256: string; readonly bytes: number };
  generatedFiles?: Record<string, string>;
  error?: string;
  finishedAt?: string;
}

const source = "717189996d41a309c703ccd95c0370b11e7b580e";
const branch = "refs/heads/diag/wasm-smoke-13711-20261126";
const directory = path.join(process.env.RUNNER_TEMP!, "vize-wasm-smoke-13711");
fs.mkdirSync(directory, { recursive: true });
const digest = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
const capture = (command: string, args: readonly string[]): string =>
  execFileSync(command, args, { encoding: "utf8" }).trim();
const receipt: Receipt = {
  source,
  startedAt: new Date().toISOString(),
  diagnosticSha: process.env.GITHUB_SHA,
  ref: process.env.GITHUB_REF,
  workflowRef: process.env.GITHUB_WORKFLOW_REF,
  workflowSha: process.env.GITHUB_WORKFLOW_SHA,
  runId: process.env.GITHUB_RUN_ID,
  runAttempt: process.env.GITHUB_RUN_ATTEMPT,
  node: process.version,
  published: false,
  purpose: "Execute the official WASM smoke against the unchanged source checkout",
  commands: [],
  success: false,
};
function save(): void {
  fs.writeFileSync(path.join(directory, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
}
async function run(name: string, args: readonly string[]): Promise<void> {
  const log = path.join(directory, name + ".log");
  const descriptor = fs.openSync(log, "w");
  const command: CommandReceipt = {
    command: process.execPath,
    args,
    startedAt: new Date().toISOString(),
    log,
  };
  receipt.commands.push(command);
  save();
  try {
    const child = spawn(process.execPath, args, { stdio: ["ignore", "pipe", "pipe"] });
    child.stdout.on("data", (chunk: Buffer) => {
      fs.writeSync(descriptor, chunk);
      process.stdout.write(chunk);
    });
    child.stderr.on("data", (chunk: Buffer) => {
      fs.writeSync(descriptor, chunk);
      process.stderr.write(chunk);
    });
    const result = await new Promise<CommandResult>((resolve, reject) => {
      child.once("error", reject);
      child.once("close", (status, signal) => resolve({ status, signal }));
    });
    Object.assign(command, result, { finishedAt: new Date().toISOString() });
    command.logSha256 = digest(fs.readFileSync(log));
    save();
    assert.equal(result.status, 0, name + " failed; see the retained log");
  } finally {
    fs.closeSync(descriptor);
  }
}

save();
try {
  assert.equal(process.env.GITHUB_EVENT_NAME, "workflow_dispatch");
  assert.equal(process.env.GITHUB_REF, branch);
  assert.equal(capture("git", ["rev-parse", "HEAD"]), process.env.GITHUB_SHA);
  assert.equal(capture("git", ["rev-parse", "HEAD^"]), source);
  const changed = capture("git", ["diff", "--name-only", source, "HEAD"]).split("\n");
  const allowed = new Set([
    ".github/workflows/release-smoke-contracts.yml",
    "tools/config/pkl/workflows/release/release-smoke-contracts.pkl",
    "tools/diagnostics/wasm-smoke-13711.ts",
  ]);
  assert.ok(
    changed.every((file) => allowed.has(file)),
    "diagnostic must preserve all product sources",
  );
  receipt.changedFiles = changed;
  receipt.inputs = Object.fromEntries(
    [
      "tools/support/tasks/build-vize-wasm-package.ts",
      "tools/commands/release/npm/smoke-wasm-package.ts",
      "tools/support/compat/npm/smoke-wasm-package.ts",
      "Cargo.lock",
      "npm/wasm/package.json",
    ].map((file) => [file, digest(fs.readFileSync(file))]),
  );
  receipt.workspaceVersion = fs
    .readFileSync("Cargo.toml", "utf8")
    .split("[workspace.package]")[1]!
    .match(/^version = "([^"]+)"/m)![1]!;
  receipt.packageVersion = (
    JSON.parse(fs.readFileSync("npm/wasm/package.json", "utf8")) as { readonly version?: unknown }
  ).version;
  assert.equal(receipt.workspaceVersion, "0.474.0");
  assert.equal(receipt.packageVersion, receipt.workspaceVersion);
  receipt.rustc = capture("rustc", ["-vV"]);
  receipt.cargo = capture("cargo", ["--version"]);
  receipt.wasmBindgen = capture("wasm-bindgen", ["--version"]);
  assert.equal(receipt.wasmBindgen, "wasm-bindgen 0.2.129");
  receipt.buildEnvironment = {
    VIZE_RELEASE_BUILD: process.env.VIZE_RELEASE_BUILD,
    CARGO_INCREMENTAL: process.env.CARGO_INCREMENTAL,
  };
  save();
  await run("build", ["tools/support/tasks/build-vize-wasm-package.ts"]);
  for (const [file, sha256] of Object.entries(receipt.inputs)) {
    assert.equal(digest(fs.readFileSync(file)), sha256, file + " changed during the build");
  }
  const wasm = fs.readFileSync("npm/wasm/vize_vitrine_bg.wasm");
  receipt.wasm = { sha256: digest(wasm), bytes: wasm.length };
  receipt.generatedFiles = Object.fromEntries(
    ["vize_vitrine.js", "vize_vitrine.d.ts"].map((file) => [
      file,
      digest(fs.readFileSync(path.join("npm/wasm", file))),
    ]),
  );
  assert.equal(
    digest(fs.readFileSync("npm/wasm/package.json")),
    receipt.inputs["npm/wasm/package.json"],
  );
  save();
  await run("smoke", ["tools/commands/release/npm/smoke-wasm-package.ts", "npm/wasm"]);
  receipt.success = true;
} catch (error) {
  receipt.error = String(
    (error as { readonly stack?: unknown } | null | undefined)?.stack ?? error,
  );
  console.error(error);
  process.exitCode = 1;
} finally {
  receipt.finishedAt = new Date().toISOString();
  save();
  console.log("Diagnostic receipt: " + path.join(directory, "receipt.json"));
}
