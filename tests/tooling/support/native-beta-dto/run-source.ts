import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";

const MAX_TRACE_BYTES = 1024 * 1024 * 1024;

function resolveExecutable(name: string): string {
  for (const directory of process.env.PATH?.split(path.delimiter) ?? []) {
    if (!directory) continue;
    const candidate = path.join(directory, name);
    try {
      fs.accessSync(candidate, fs.constants.X_OK);
      return fs.realpathSync(candidate);
    } catch {}
  }
  throw new Error(`missing executable: ${name}`);
}

function hashBoundedFile(file: string): { file: "syscalls.merged"; bytes: number; sha256: string } {
  const descriptor = fs.openSync(file, "r");
  const hash = createHash("sha256");
  const buffer = Buffer.allocUnsafe(1024 * 1024);
  let bytes = 0;
  try {
    for (;;) {
      const count = fs.readSync(descriptor, buffer, 0, buffer.length, null);
      if (count === 0) break;
      bytes += count;
      assert.ok(bytes <= MAX_TRACE_BYTES, "bounded merged trace required");
      hash.update(buffer.subarray(0, count));
    }
  } finally {
    fs.closeSync(descriptor);
  }
  assert.ok(bytes > 0, "actual merged native trace required");
  return { file: "syscalls.merged", bytes, sha256: hash.digest("hex") };
}

type BuildArtifact = {
  reason: string;
  target: { name: string };
  profile: { test: boolean };
  executable: string;
};
const capture = process.env.VIZE_NATIVE_DTO_CAPTURE!;
const selector =
  "lsp_client::editor_lsp::snapshot_source::tests::native_beta::actual_sdk_semantic_error_repair_navigation_and_two_file_scope_are_complete";
const artifacts = fs
  .readFileSync(path.join(capture, "build.jsonl"), "utf8")
  .trim()
  .split("\n")
  .map((line) => JSON.parse(line) as BuildArtifact)
  .filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      row.target.name === "vize_canon" &&
      row.profile.test &&
      row.executable,
  );
assert.equal(artifacts.length, 1, "exactly one actual source test executable required");
const executable = artifacts[0].executable;
const hash = createHash("sha256").update(fs.readFileSync(executable)).digest("hex");
const strace = resolveExecutable("strace");
const straceHash = createHash("sha256").update(fs.readFileSync(strace)).digest("hex");
const straceVersion = spawnSync(strace, ["--version"], {
  encoding: "utf8",
  maxBuffer: 64 * 1024,
});
assert.equal(straceVersion.error, undefined);
assert.equal(straceVersion.signal, null);
assert.equal(straceVersion.status, 0);
const tracePath = path.join(capture, "syscalls.merged");
const args = [
  "-f",
  "-ttt",
  "-s",
  "16777216",
  "-xx",
  "-e",
  "trace=read,write,readv,writev,process",
  "-o",
  tracePath,
  executable,
  selector,
  "--ignored",
  "--exact",
  "--nocapture",
];
const stdout = fs.openSync(path.join(capture, "test.stdout"), "w");
const stderr = fs.openSync(path.join(capture, "test.stderr"), "w");
let result: SpawnSyncReturns<Buffer>;
try {
  result = spawnSync(strace, args, {
    cwd: process.env.VIZE_NATIVE_DTO_SOURCE_ROOT,
    env: process.env,
    stdio: ["ignore", stdout, stderr],
    timeout: 600000,
  });
} finally {
  fs.closeSync(stdout);
  fs.closeSync(stderr);
}
const trace = fs.existsSync(tracePath) ? hashBoundedFile(tracePath) : null;
fs.writeFileSync(
  path.join(capture, "source-execution.json"),
  JSON.stringify(
    {
      source: process.env.VIZE_NATIVE_DTO_SOURCE_SHA,
      executable,
      executableSha256: hash,
      command: strace,
      commandSha256: straceHash,
      commandVersion: straceVersion.stdout.trim(),
      args,
      trace,
      status: result.status,
      roleKind:
        process.env.VIZE_NATIVE_DTO_ROLE === "before" &&
        process.env.VIZE_NATIVE_DTO_SDK_VERSION === "7.1.0-beta"
          ? "baseline-observation-known-dto-refusal"
          : process.env.VIZE_NATIVE_DTO_ROLE === "before" &&
              process.env.VIZE_NATIVE_DTO_SDK_VERSION === "7.0.2"
            ? "baseline-observation-registration-boundary"
            : "functional-source-control",
      signal: result.signal,
      error: result.error?.message ?? null,
      note: "Trace the actual source observer externally. Production receives the actual SDK image path, retaining its native image authentication.",
    },
    null,
    2,
  ) + "\n",
);
assert.equal(result.error, undefined);
assert.equal(result.signal, null);
assert.equal(result.status, 0);
