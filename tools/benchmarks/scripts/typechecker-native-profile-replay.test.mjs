import { readToolingProgram } from "../../../tests/tooling/support/program-path.ts";
/** Real subprocess transport controls, never native performance evidence. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmodSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import test from "node:test";
import { gunzipSync } from "node:zlib";
import { replayNativeProfile } from "./typechecker-native-profile-replay.mjs";

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

function fixture(mode = "valid", large = false) {
  const directory = mkdtempSync(join(tmpdir(), "native-profile-replay-"));
  const runtime = join(directory, "fake-native.mjs");
  const config = join(directory, "fixture.json");
  const stdout = Buffer.concat([
    Buffer.from("file.ts(1,1): error TS2322: 雪🌸\r\nMemory profile: meaningful diagnostic text\n"),
    Buffer.from([0xff, 0x00, 0x0a]),
    Buffer.from(large ? "native-out-🌸\n".repeat(100_000) : ""),
  ]);
  const stderr = Buffer.from(large ? "native-stderr-雪\n".repeat(100_000) : "native stderr\n");
  writeFileSync(
    config,
    JSON.stringify({ stdout: stdout.toString("base64"), stderr: stderr.toString("base64") }),
  );
  writeFileSync(
    runtime,
    readToolingProgram("commands/typechecker-native-profile-replay/runtime.js.in", {
      NODE: process.execPath,
    }),
  );
  chmodSync(runtime, 0o755);
  const args = ["--checkers", "1", "--project", config];
  const env = { ...process.env, PROFILE_CASE: mode };
  const baseline = spawnSync(runtime, args, { cwd: directory, env, maxBuffer: 64 * 1024 * 1024 });
  assert.equal(baseline.error, undefined, baseline.error?.message);
  assert.equal(baseline.status, 3);
  assert(baseline.stdout.equals(stdout) && baseline.stderr.equals(stderr));
  return {
    directory,
    stdout,
    stderr,
    options: {
      runtime,
      args,
      cwd: directory,
      baseline,
      env,
      profileDirectory: join(directory, "profile"),
    },
  };
}

function cleanup(context) {
  rmSync(context.directory, { recursive: true, force: true });
}

await test("real buffered replay preserves large binary streams, status and genuine profiles", () => {
  const context = fixture("valid", true);
  try {
    assert(context.stdout.length > 1024 * 1024 && context.stderr.length > 1024 * 1024);
    const result = replayNativeProfile(context.options);
    assert.equal(result.validation, "passed");
    assert.equal(result.original.status, 3);
    assert.equal(result.replay.status, 3);
    assert.deepEqual(result.replay.command.slice(-2), ["--pprofDir", result.profileDirectory]);
    for (const [stream, bytes] of [
      ["stdout", context.stdout],
      ["stderr", context.stderr],
    ]) {
      const saved = result.original[stream];
      assert.equal(saved.bytes, bytes.length);
      assert.equal(saved.sha256, sha256(bytes));
      assert(readFileSync(saved.path).equals(bytes));
    }
    for (const profile of result.profileFiles) {
      const bytes = readFileSync(profile.path);
      assert.equal(profile.sha256, sha256(bytes));
      assert.equal(profile.bytes, bytes.length);
      assert.equal(profile.unpackedBytes, gunzipSync(bytes).length);
      assert(profile.path.startsWith(`${result.profileDirectory}/`));
    }
    const replay = readFileSync(result.replay.stdout.path);
    assert(replay.subarray(0, context.stdout.length).equals(context.stdout));
    assert.equal(JSON.parse(readFileSync(result.receiptPath)).validation, "passed");
    assert.equal(result.excludedFromTimings, true);
    assert.equal(result.nativeBinaryUnchanged, true);
  } finally {
    cleanup(context);
  }
});

for (const [mode, message] of [
  ["diagnostics", /changed stdout/u],
  ["extra-line", /changed stdout/u],
  ["wrong-line", /changed stdout/u],
  ["order", /changed stdout/u],
  ["stderr", /changed stderr/u],
  ["status", /changed exit status/u],
  ["missing", /missing or unexpected/u],
  ["wrong-pid", /missing or unexpected/u],
  ["extra-file", /missing or unexpected/u],
  ["empty", /invalid profile byte size/u],
  ["bad-gzip", /incorrect header check/u],
  ["symlink", /regular file/u],
  ["hardlink", /alias another file/u],
  ["binary-change", /native binary changed/u],
]) {
  await test(`reject ${mode} without discarding original or replay evidence`, () => {
    const context = fixture(mode);
    try {
      assert.throws(() => replayNativeProfile(context.options), message);
      const receipt = JSON.parse(
        readFileSync(join(context.options.profileDirectory, "evidence", "receipt.json")),
      );
      assert.equal(receipt.validation, "failed");
      assert(receipt.failure.length > 0);
      assert(readFileSync(receipt.original.stdout.path).equals(context.stdout));
      for (const stream of [receipt.replay.stdout, receipt.replay.stderr]) {
        const bytes = readFileSync(stream.path);
        assert.equal(bytes.length, stream.bytes);
        assert.equal(sha256(bytes), stream.sha256);
      }
      assert.equal(receipt.instrumentationOnly, true);
    } finally {
      cleanup(context);
    }
  });
}

await test("reused directories and symlink destinations cannot overwrite existing files", () => {
  const context = fixture();
  try {
    replayNativeProfile(context.options);
    const original = readFileSync(
      join(context.options.profileDirectory, "evidence", "receipt.json"),
    );
    assert.throws(() => replayNativeProfile(context.options), /EEXIST/u);
    assert(
      readFileSync(join(context.options.profileDirectory, "evidence", "receipt.json")).equals(
        original,
      ),
    );
    const alias = join(context.directory, "alias");
    symlinkSync(context.options.profileDirectory, alias);
    assert.throws(
      () => replayNativeProfile({ ...context.options, profileDirectory: alias }),
      /EEXIST/u,
    );
    assert(
      readFileSync(join(context.options.profileDirectory, "evidence", "receipt.json")).equals(
        original,
      ),
    );
  } finally {
    cleanup(context);
  }
});

await test("spawn errors and truncation retain a failed partial receipt", () => {
  for (const maxBuffer of [1, 64]) {
    const context = fixture();
    try {
      assert.throws(() => replayNativeProfile({ ...context.options, maxBuffer }), /ENOBUFS/u);
      const receipt = JSON.parse(
        readFileSync(join(context.options.profileDirectory, "evidence", "receipt.json")),
      );
      assert.equal(receipt.validation, "failed");
      assert.match(receipt.replay.error, /ENOBUFS/u);
      assert(readFileSync(receipt.original.stdout.path).equals(context.stdout));
    } finally {
      cleanup(context);
    }
  }
});

await test("profiling flags and invalid original state are rejected before invocation", () => {
  const context = fixture();
  try {
    for (const flag of [
      "--pprofDir",
      "--pprofDir=/other",
      "--generateTrace",
      "--generateCpuProfile",
    ]) {
      assert.throws(
        () => replayNativeProfile({ ...context.options, args: [...context.options.args, flag] }),
        /already enables profiling/u,
      );
    }
    assert.throws(() => replayNativeProfile({ ...context.options, profileDirectory: "relative" }));
    assert.throws(
      () =>
        replayNativeProfile({
          ...context.options,
          baseline: { ...context.options.baseline, signal: "SIGTERM" },
        }),
      /received a signal/u,
    );
    assert.throws(() =>
      replayNativeProfile({
        ...context.options,
        baseline: { ...context.options.baseline, stdout: "text" },
      }),
    );
    assert.equal(resolve(context.options.profileDirectory), context.options.profileDirectory);
  } finally {
    cleanup(context);
  }
});
