/** Untimed native --pprofDir replay; original byte streams remain authoritative. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  lstatSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, isAbsolute, join, resolve } from "node:path";
import { gunzipSync } from "node:zlib";

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

function directoryIsUnchanged(path) {
  const stat = lstatSync(path);
  assert(stat.isDirectory() && !stat.isSymbolicLink(), "replay replaced evidence directory");
  assert.equal(realpathSync(path), path, "replay redirected evidence directory");
}

function saveBytes(directory, name, bytes) {
  assert(Buffer.isBuffer(bytes), "native stream must retain raw Buffer bytes");
  const path = join(directory, name);
  writeFileSync(path, bytes, { flag: "wx" });
  assert(readFileSync(path).equals(bytes), "saved native stream differs from observed bytes");
  return { path, bytes: bytes.length, sha256: sha256(bytes) };
}

function profileFile(path, kind) {
  const stat = lstatSync(path);
  assert(stat.isFile() && !stat.isSymbolicLink(), "profile must be a regular file");
  assert.equal(stat.nlink, 1, "profile must not alias another file");
  assert(stat.size > 0 && stat.size <= 256 * 1024 * 1024, "invalid profile byte size");
  const bytes = readFileSync(path);
  assert.equal(bytes.length, stat.size, "profile changed while reading");
  const unpacked = gunzipSync(bytes, { maxOutputLength: 256 * 1024 * 1024 });
  assert(unpacked.length > 0, "empty compressed profile");
  return { kind, path, bytes: bytes.length, sha256: sha256(bytes), unpackedBytes: unpacked.length };
}

/**
 * Parent integration owns pinned runtime/version, members, configs and freshness.
 * profileDirectory must be new; its parent must already exist. It retains raw
 * evidence even when replay validation fails. Profile payloads are gzip-validated;
 * semantic pprof decoding belongs to the subsequent native-profile report.
 */
export function replayNativeProfile({
  runtime,
  args,
  cwd,
  baseline,
  profileDirectory,
  env = process.env,
  spawn = spawnSync,
  timeout = 300_000,
  maxBuffer = 64 * 1024 * 1024,
}) {
  assert(isAbsolute(runtime) && isAbsolute(cwd), "native runtime/cwd must be absolute");
  assert(Array.isArray(args) && args.every((arg) => typeof arg === "string"));
  assert(
    !args.some((arg) => /^--(?:pprofDir|generateTrace|generateCpuProfile)(?:=|$)/iu.test(arg)),
    "original invocation already enables profiling",
  );
  assert.equal(baseline.error, undefined, "original native invocation failed to run");
  assert.equal(baseline.signal, null, "original native invocation received a signal");
  assert(Number.isInteger(baseline.status) && baseline.status >= 0, "invalid original status");
  assert(Buffer.isBuffer(baseline.stdout) && Buffer.isBuffer(baseline.stderr));
  assert(isAbsolute(profileDirectory) && !/[\r\n]/u.test(profileDirectory));
  const requestedDirectory = resolve(profileDirectory);
  const directory = join(realpathSync(dirname(requestedDirectory)), basename(requestedDirectory));
  mkdirSync(directory, { mode: 0o700 }); // EEXIST rejects reused directories and symlinks.
  const evidence = join(directory, "evidence");
  mkdirSync(evidence, { mode: 0o700 });
  const receiptPath = join(evidence, "receipt.json");
  const runtimeSha256 = sha256(readFileSync(runtime));
  const replayArgs = [...args, "--pprofDir", directory];
  const record = {
    schemaVersion: 1,
    instrumentationOnly: true,
    excludedFromTimings: true,
    profileDirectory: directory,
    runtime,
    runtimeSha256,
    original: {
      command: [runtime, ...args],
      cwd,
      status: baseline.status,
      signal: baseline.signal,
      stdout: saveBytes(evidence, "original.stdout.bin", baseline.stdout),
      stderr: saveBytes(evidence, "original.stderr.bin", baseline.stderr),
    },
    validation: "pending",
  };
  const persist = () => writeFileSync(receiptPath, `${JSON.stringify(record, null, 2)}\n`);
  persist();
  try {
    const result = spawn(runtime, replayArgs, { cwd, env, timeout, maxBuffer });
    directoryIsUnchanged(directory);
    directoryIsUnchanged(evidence);
    record.replay = {
      command: [runtime, ...replayArgs],
      cwd,
      pid: result.pid,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: saveBytes(evidence, "replay.stdout.bin", result.stdout ?? Buffer.alloc(0)),
      stderr: saveBytes(evidence, "replay.stderr.bin", result.stderr ?? Buffer.alloc(0)),
    };
    persist(); // Failed executions retain complete observed streams before assertions.
    assert.equal(result.error, undefined, result.error?.message);
    assert.equal(result.signal, null, "profile replay received a signal");
    assert.equal(result.status, baseline.status, "profile replay changed exit status");
    assert(result.stderr.equals(baseline.stderr), "profile replay changed stderr bytes");
    assert(Number.isInteger(result.pid) && result.pid > 0, "missing actual replay PID");
    const cpu = join(directory, `${result.pid}-cpuprofile.pb.gz`);
    const allocations = join(directory, `${result.pid}-memprofile.pb.gz`);
    assert.deepEqual(
      readdirSync(directory).sort((a, b) => a.localeCompare(b, "en")),
      ["evidence", basename(cpu), basename(allocations)].sort((a, b) => a.localeCompare(b, "en")),
      "missing or unexpected native profile files",
    );
    const suffix = Buffer.from(`Memory profile: ${allocations}\nCPU profile: ${cpu}\n`);
    assert(
      result.stdout.equals(Buffer.concat([baseline.stdout, suffix])),
      "profile replay changed stdout beyond the two exact native profile path lines",
    );
    record.profileFiles = [profileFile(cpu, "cpu"), profileFile(allocations, "allocations")];
    assert.equal(
      sha256(readFileSync(runtime)),
      runtimeSha256,
      "native binary changed during replay",
    );
    record.statusEqual = true;
    record.stderrBytesEqual = true;
    record.stdoutBytesEqualAfterExactProfileSuffix = true;
    record.nativeBinaryUnchanged = true;
    record.validation = "passed";
    persist();
    return { ...record, receiptPath };
  } catch (error) {
    record.validation = "failed";
    record.failure = error.message;
    directoryIsUnchanged(directory);
    directoryIsUnchanged(evidence);
    persist();
    throw error;
  }
}
