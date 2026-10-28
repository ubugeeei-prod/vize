import assert from "node:assert/strict";
import { test } from "node:test";
import {
  nativeCheckerImage,
  validateNativeCheckerCapture,
} from "../../tools/benchmarks/scripts/native-checker-cpu-metadata.ts";
void test("captured checker identity rejects unrelated SDKs and rewritten executable paths", () => {
  const original = {
    identity: {
      backend: nativeCheckerImage.path,
      backendSha256: nativeCheckerImage.sha256,
      source: nativeCheckerImage.source,
    },
    binarySha256: nativeCheckerImage.normalSha256,
    command: { args: ["check", "--corsa-path", nativeCheckerImage.path] },
  };
  assert.doesNotThrow(() => validateNativeCheckerCapture(original, nativeCheckerImage.sha256));
  for (const mutation of [
    { ...original, identity: { ...original.identity, backend: "/tmp/unrelated/tsc" } },
    { ...original, identity: { ...original.identity, backendSha256: "0".repeat(64) } },
    { ...original, identity: { ...original.identity, source: "0".repeat(40) } },
    { ...original, binarySha256: "0".repeat(64) },
    { ...original, command: { args: ["check", "--corsa-path", "/tmp/unrelated/tsc"] } },
    { ...original, command: { args: ["check"] } },
    {
      ...original,
      command: {
        args: [
          "check",
          "--corsa-path",
          nativeCheckerImage.path,
          "--corsa-path",
          "/tmp/unrelated/tsc",
        ],
      },
    },
  ])
    assert.throws(() => validateNativeCheckerCapture(mutation, nativeCheckerImage.sha256));
  assert.throws(() => validateNativeCheckerCapture(original, "0".repeat(64)));
});

void test("runtime checker PCs require prior PID mappings, PT_LOAD and independent dsoff agreement", async () => {
  const { bindCheckerProgramCounters } =
    await import("../../tools/benchmarks/scripts/native-checker-cpu-pcs.ts");
  const map =
    "42/42 10.000000000: PERF_RECORD_MMAP2 42/42: [0x400000(0x10000) @ 0 fd:00 1 2]: r-xp /sdk/tsc\n";
  const sample = "42/99 10.010000000: 10101010 401010 (/sdk/tsc+0x1010)\n";
  const other = "7/7 10.020000000: 10101010 700020 (/tool/vize+0x20)\n";
  const elf =
    "Type: EXEC (Executable file)\n LOAD 0x000000 0x400000 0x400000 0x10000 0x10000 R E 0x1000\n";
  const run = (trace = map + sample + other, headers = elf) =>
    bindCheckerProgramCounters(trace, headers, "/sdk/tsc", { all: 2, checker: 1 });
  const result = run();
  assert.equal(result.qualified, true);
  assert.equal(result.mappedCheckerSamples, 1);
  assert.equal(result.rows[0].elfPc, "0x401010");
  assert.equal(result.rows[0].fileOffset, "0x1010");
  assert.equal(result.rows[0].loadBias, "0x0");
  for (const bad of [
    map + sample.replace("+0x1010", "+0x1011") + other,
    map.replace("10.000000000", "10.030000000") + sample + other,
    map.replaceAll("42/42", "41/41") + sample + other,
    map.replace("r-xp", "r--p") + sample + other,
    map + sample.replace("401010", "410000").replace("+0x1010", "+0x10000") + other,
    map + other,
  ])
    assert.equal(run(bad).qualified, false);
  assert.equal(run(undefined, elf.replace("0x10000 0x10000", "0x1000 0x1000")).qualified, false);
  assert.equal(
    run(undefined, elf.replace("0x400000 0x400000", "0x500000 0x500000")).qualified,
    false,
  );
  assert.equal(run(undefined, elf + elf.split("\n")[1] + "\n").qualified, false);
});
void test("Go address decoder retains every unique PC and demands negative controls", async () => {
  const { parseGoAddressFrames } =
    await import("../../tools/benchmarks/scripts/native-checker-cpu-decode.ts");
  const output = "?\n?:0\nchecker.actualFunction\nmodule/checker.go:123\n?\n?:0\n";
  const result = parseGoAddressFrames(output, ["0x401010"]);
  assert.equal(result[0].function, "checker.actualFunction");
  assert.equal(result[0].line, 123);
  assert.equal(result[0].functionResolved, true);
  assert.equal(result[0].sourceTreeOwnershipVerified, false);
  assert.equal(result[0].sourceUrl, null);
  assert.throws(() =>
    parseGoAddressFrames(output.replace("?\n?:0", "unexpected\nfake.go:1"), ["0x401010"]),
  );
  assert.throws(() => parseGoAddressFrames(output.slice(0, -4), ["0x401010"]));
  assert.throws(() =>
    parseGoAddressFrames(output.replace("module/checker.go:123", "invalid"), ["0x401010"]),
  );
});

void test("Rust caller collection selects the exact executable PID and retains its other DSOs", async () => {
  const { rustPidScope } =
    await import("../../tools/benchmarks/scripts/native-checker-cpu-rust-callers.ts");
  const trace =
    "7/7 10.000000000: 10101010 400001 (/bin/vize+0x1)\n" +
    "7/8 10.000000001: 10101010 400002 (/bin/vize+0x2)\n" +
    "7/8 10.000000002: 10101010 700003 (/lib/libc.so+0x3)\n" +
    "9/9 10.000000003: 10101010 800004 (/bin/tsc+0x4)\n";
  assert.deepEqual(rustPidScope(trace, "/bin/vize"), {
    pids: ["7"],
    rustImageSamples: 2,
    selectedPidSamples: 3,
  });
  assert.deepEqual(rustPidScope(trace, "/bin/missing"), {
    pids: [],
    rustImageSamples: 0,
    selectedPidSamples: 0,
  });
});
