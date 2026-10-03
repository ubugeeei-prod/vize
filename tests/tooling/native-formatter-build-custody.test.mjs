import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  bytes,
  emittedNativeArtifact,
  captureNativeHistoryBuild,
} from "../../npm/native/scripts/formatter-history-build.mjs";

// Synthetic admission controls only. Real emitted addon custody is proved by
// the mandatory existing hosted native build, not these generated vectors.
void test("Cargo admission requires genuine complete success and unambiguous typed native artifact", () => {
  const artifact = {
    reason: "compiler-artifact",
    target: { name: "vize_vitrine", kind: ["cdylib", "rlib"] },
    profile: { test: false },
    features: ["napi", "legacy"],
    filenames: ["/synthetic/libvize_vitrine.so"],
  };
  const complete = (rows) => Buffer.from(rows.map((row) => JSON.stringify(row)).join("\n") + "\n");
  const finished = { reason: "build-finished", success: true };
  assert.deepEqual(emittedNativeArtifact(complete([artifact, finished])), {
    cargo: artifact,
    filename: artifact.filenames[0],
  });
  for (const rows of [
    [artifact],
    [artifact, { ...finished, success: false }],
    [artifact, artifact, finished],
    [{ ...artifact, features: ["legacy"] }, finished],
    [{ ...artifact, filenames: ["/synthetic/one.so", "/synthetic/two.so"] }, finished],
    [{ ...artifact, target: { name: "vize_vitrine", kind: ["rlib"] } }, finished],
  ])
    assert.throws(() => emittedNativeArtifact(complete(rows)));
});

void test("failed build admission retains raw actual command/process streams before qualification", (t) => {
  const nativeDir = fs.mkdtempSync(path.join(os.tmpdir(), "native-build-custody-"));
  t.after(() => fs.rmSync(nativeDir, { recursive: true, force: true }));
  const stdout = Buffer.from("whole failed build stdout\0\n");
  const stderr = Buffer.from("whole failed build stderr\0\n");
  assert.throws(() =>
    captureNativeHistoryBuild(nativeDir, null, ["synthetic", "control"], {
      stdout,
      stderr,
      status: 1,
      signal: null,
      error: undefined,
    }),
  );
  const directory = path.join(nativeDir, ".artifacts/native/formatter-history");
  assert(fs.readFileSync(path.join(directory, "cargo.stdout.bin")).equals(stdout));
  assert(fs.readFileSync(path.join(directory, "cargo.stderr.bin")).equals(stderr));
  const frame = JSON.parse(fs.readFileSync(path.join(directory, "build-process.json")));
  assert.deepEqual(frame.command, ["synthetic", "control"]);
  assert.deepEqual(frame.stdout, bytes(stdout));
  assert.deepEqual(frame.stderr, bytes(stderr));
  assert.equal(frame.exitStatus, 1);
  assert(!fs.existsSync(path.join(directory, "build-receipt.json")));
});
