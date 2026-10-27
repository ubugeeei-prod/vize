/**
 * Exact-source functional proof for compiler-option snapshots.
 * Runtime Vue provider authority and performance are separate qualifications.
 */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  closeSync,
  existsSync,
  mkdirSync,
  openSync,
  readFileSync,
  readSync,
  statSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

type SourceIdentity = {
  head: string;
  tree: string;
  parents: string[];
  status: string;
  lockSha256: string;
};
type TestSummary = {
  state: string;
  passed: number;
  failed: number;
  ignored: number;
  measured: number;
  filtered: number;
};
type Image = { path: string; bytes: number; sha256: string };
type ProcessReceipt = {
  label: string;
  command: string[];
  cwd: string;
  status: number | null;
  timedOut: boolean;
  elapsedNs: number;
  stdoutBytes: number;
  stderrBytes: number;
  stdoutSha256: string;
  stderrSha256: string;
  executables: Image[];
};
type TestGroup = { label: string; names: string[]; summary: TestSummary };
type Proof = {
  scope: string;
  baseline: string;
  candidate: string;
  qualified: boolean;
  processes: ProcessReceipt[];
  testGroups: TestGroup[];
  candidateSource?: SourceIdentity;
  baselineSource?: SourceIdentity;
  candidateSourceAfter?: SourceIdentity;
  observerSha256?: string;
  beforeLockFailure?: TestSummary;
  host?: {
    platform: string;
    node: string;
    functionalProfile: string;
    cargoBuildJobs: string;
    rustFlags: string | null;
    targetLinker: string | null;
  };
  failure?: { name: string; message: string; stack?: string };
};
const root = process.cwd();
const out = process.env.COMPILER_OPTIONS_PROOF_OUT;
const expected = process.env.QUALIFICATION_HEAD_SHA;
assert.ok(out && path.isAbsolute(out), "owned absolute output is required");
assert.ok(expected && /^[0-9a-f]{40}$/u.test(expected), "exact source head is required");
const base = "9dfc33388e704e321fca1cc2a4feffa818117bf8";
const observer = "crates/vize_canon/tests/compiler_options_snapshot_io.rs";
const filters = ["tsconfig_extends", "compiler_snapshot", "compiler_inputs"];
mkdirSync(out, { recursive: true });
const proof: Proof = {
  scope: "source functional only; runtime Vue authority unproved",
  baseline: base,
  candidate: expected,
  qualified: false,
  processes: [],
  testGroups: [],
};
function saveResult() {
  writeFileSync(path.join(out!, "result.json"), JSON.stringify(proof, null, 2) + "\n");
}
process.on("exit", saveResult);
process.on("uncaughtExceptionMonitor", (error) => {
  proof.failure = { name: error.name, message: error.message, stack: error.stack };
  saveResult();
});
const sha = (data: Uint8Array) => createHash("sha256").update(data).digest("hex");
function fileImage(filename: string): Image {
  const digest = createHash("sha256");
  const buffer = Buffer.alloc(1024 * 1024);
  const descriptor = openSync(filename, "r");
  try {
    let bytes: number;
    while ((bytes = readSync(descriptor, buffer, 0, buffer.length, null)) !== 0) {
      digest.update(buffer.subarray(0, bytes));
    }
  } finally {
    closeSync(descriptor);
  }
  return { path: filename, bytes: statSync(filename).size, sha256: digest.digest("hex") };
}
function git(...args: string[]) {
  const result = spawnSync("git", args, {
    cwd: root,
    encoding: "utf8",
    timeout: 30_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.error, undefined, String(result.error));
  return result.stdout.trim();
}
function sourceIdentity(): SourceIdentity {
  const commit = git("cat-file", "-p", "HEAD");
  return {
    head: git("rev-parse", "HEAD"),
    tree: git("rev-parse", "HEAD^{tree}"),
    parents: [...commit.matchAll(/^parent (\w+)$/gmu)].map((match) => match[1]),
    status: git("status", "--porcelain"),
    lockSha256: sha(readFileSync(path.join(root, "Cargo.lock"))),
  };
}
function command(label: string, args: string[]): [number | null, string] {
  console.info("Running " + label);
  const started = process.hrtime.bigint();
  const result = spawnSync(args[0], args.slice(1), {
    cwd: root,
    timeout: 2_100_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const stdout = result.stdout ?? Buffer.alloc(0);
  const stderr = result.stderr ?? Buffer.alloc(0);
  const elapsedNs = Number(process.hrtime.bigint() - started);
  const timedOut = (result.error as NodeJS.ErrnoException | undefined)?.code === "ETIMEDOUT";
  writeFileSync(path.join(out!, label + ".stdout"), stdout);
  writeFileSync(path.join(out!, label + ".stderr"), stderr);
  const output = Buffer.concat([stdout, Buffer.from("\n"), stderr]).toString("utf8");
  const executables: Image[] = [];
  for (const match of output.matchAll(/Running [^\n]* \(([^()\n]+)\)/gu)) {
    const filename = path.resolve(root, match[1]);
    if (existsSync(filename) && statSync(filename).isFile()) executables.push(fileImage(filename));
  }
  const receipt: ProcessReceipt = {
    label,
    command: args,
    cwd: root,
    status: result.status,
    timedOut,
    elapsedNs,
    stdoutBytes: stdout.length,
    stderrBytes: stderr.length,
    stdoutSha256: sha(stdout),
    stderrSha256: sha(stderr),
    executables,
  };
  writeFileSync(path.join(out!, label + ".process.json"), JSON.stringify(receipt, null, 2) + "\n");
  proof.processes.push(receipt);
  saveResult();
  assert.equal(result.error, undefined, String(result.error));
  assert.equal(result.signal, null, label + " terminated by signal");
  return [result.status, output];
}
function cargo(...args: string[]) {
  return ["cargo", "test", "--locked", "--profile", "ci", "-p", "vize_canon", ...args];
}
function summary(output: string): TestSummary {
  const matches = [
    ...output.matchAll(
      /test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out/gu,
    ),
  ];
  assert.equal(matches.length, 1, "one actually executed test packet is required");
  const [, state, passed, failed, ignored, measured, filtered] = matches[0];
  return {
    state,
    passed: Number(passed),
    failed: Number(failed),
    ignored: Number(ignored),
    measured: Number(measured),
    filtered: Number(filtered),
  };
}
function requireTests(label: string, args: string[], count?: number) {
  const [listStatus, listing] = command(label + "-list", [
    ...args,
    "--",
    "--list",
    "--format",
    "terse",
  ]);
  assert.equal(listStatus, 0);
  const names = listing
    .split("\n")
    .filter((line) => line.endsWith(": test"))
    .map((line) => line.slice(0, -6))
    .sort();
  assert.ok(names.length > 0, label + " cannot qualify zero tests");
  assert.equal(names.length, new Set(names).size, label + " duplicate names");
  if (count !== undefined) assert.equal(names.length, count);
  const [status, output] = command(label, [
    ...args,
    "--",
    "--nocapture",
    "--test-threads=1",
    "--color",
    "never",
  ]);
  const packet = summary(output);
  const executed = [...output.matchAll(/^test (\S+) \.\.\. ok$/gmu)]
    .map((match) => match[1])
    .sort();
  assert.equal(status, 0);
  assert.equal(packet.state, "ok");
  assert.equal(packet.passed, names.length);
  assert.equal(packet.failed, 0);
  assert.equal(packet.ignored, 0);
  assert.equal(packet.measured, 0);
  assert.deepEqual(executed, names);
  proof.testGroups.push({ label, names, summary: packet });
  return names;
}
const candidateSource = sourceIdentity();
assert.equal(candidateSource.head, expected);
assert.equal(candidateSource.status, "");
const observerBytes = readFileSync(path.join(root, observer));
writeFileSync(path.join(out, "compiler_options_snapshot_io.rs"), observerBytes);
writeFileSync(path.join(out, "proof-driver.ts"), readFileSync(fileURLToPath(import.meta.url)));
proof.candidateSource = candidateSource;
proof.observerSha256 = sha(observerBytes);
proof.host = {
  platform: [os.type(), os.release(), os.arch()].join("-"),
  node: process.version,
  functionalProfile: "ci",
  cargoBuildJobs: process.env.CARGO_BUILD_JOBS ?? "",
  rustFlags: process.env.RUSTFLAGS ?? null,
  targetLinker: process.env.CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER ?? null,
};
saveResult();
for (const [label, args] of [
  ["rustc", ["rustc", "--version", "--verbose"]],
  ["cargo", ["cargo", "--version"]],
] as const) {
  const [status] = command(label, [...args]);
  assert.equal(status, 0);
}
const beforeNames = new Map<string, string[]>();
let injected = false;
try {
  git("checkout", "--detach", base);
  const baselineSource = sourceIdentity();
  assert.equal(baselineSource.head, base);
  assert.equal(baselineSource.status, "");
  assert.ok(baselineSource.parents.includes("1f27dba30f1be1d2d9c445860797ce1523dc8f8b"));
  proof.baselineSource = baselineSource;
  assert.equal(existsSync(path.join(root, observer)), false, "baseline already owns observer");
  writeFileSync(path.join(root, observer), observerBytes);
  injected = true;
  assert.equal(git("diff", "--", "crates/vize_canon/src"), "");
  const [status, output] = command(
    "before-cache-lock",
    cargo(
      "--test",
      "compiler_options_snapshot_io",
      "--",
      "--nocapture",
      "--test-threads=1",
      "--color",
      "never",
    ),
  );
  const packet = summary(output);
  assert.equal(status, 101);
  assert.deepEqual(packet, {
    state: "FAILED",
    passed: 0,
    failed: 1,
    ignored: 0,
    measured: 0,
    filtered: 0,
  });
  assert.ok(output.includes("reading compiler options created generated-project cache locks"));
  proof.beforeLockFailure = packet;
  for (const [index, filter] of filters.entries()) {
    beforeNames.set(filter, requireTests("before-chain-" + index, cargo("--lib", filter)));
  }
} finally {
  if (injected) {
    assert.deepEqual(readFileSync(path.join(root, observer)), observerBytes);
    unlinkSync(path.join(root, observer));
  }
  git("checkout", "--detach", expected);
}
assert.deepEqual(sourceIdentity(), candidateSource, "candidate did not restore exactly");
requireTests("candidate-observed", cargo("--lib", "compiler_options_snapshot::observed::tests"), 7);
requireTests("candidate-cache-lock", cargo("--test", "compiler_options_snapshot_io"), 1);
for (const [index, filter] of filters.entries()) {
  const names = requireTests("candidate-chain-" + index, cargo("--lib", filter));
  assert.deepEqual(names, beforeNames.get(filter), "original chain inventory changed");
}
assert.deepEqual(sourceIdentity(), candidateSource, "candidate source changed during tests");
proof.qualified = true;
proof.candidateSourceAfter = sourceIdentity();
saveResult();
console.info(
  JSON.stringify(
    {
      qualified: true,
      candidate: expected,
      testGroups: proof.testGroups.map((group) => ({
        label: group.label,
        count: group.names.length,
      })),
    },
    null,
    2,
  ),
);
