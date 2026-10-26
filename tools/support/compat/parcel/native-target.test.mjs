import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { captureAbsentNativeTarget, validateNativeTargetCustody } from "./native-target.mjs";

void test("the producer target must be absent, including dangling symlinks", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-parcel-native-target-"));
  try {
    assert.equal(captureAbsentNativeTarget(root, "a".repeat(40), "1").absent, true);
    fs.mkdirSync(path.join(root, "target"));
    assert.throws(() => captureAbsentNativeTarget(root, "a".repeat(40), "1"), /already exists/);
    fs.rmdirSync(path.join(root, "target"));
    fs.symlinkSync(path.join(root, "missing"), path.join(root, "target"));
    assert.throws(() => captureAbsentNativeTarget(root, "a".repeat(40), "1"), /already exists/);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("cold evidence must describe the compiler's actual path, source, environment and fresh artifact", () => {
  const before = {
    schema: "vize.parcel.native-target-before.v1",
    repository: "/repo",
    target: "/repo/target",
    source: "a".repeat(40),
    runId: "1",
    absent: true,
  };
  const build = {
    source: { head: before.source },
    sourceBefore: { head: before.source },
    request: { runId: "1" },
    environment: { before: { CARGO_TARGET_DIR: null }, after: { CARGO_TARGET_DIR: null } },
    emitted: {
      filename: "/repo/target/x86_64-unknown-linux-gnu/ci/libvize_vitrine.so",
      cargo: { fresh: false },
    },
    generated: { sha256: "b".repeat(64) },
    frozen: { sha256: "b".repeat(64) },
  };
  assert.equal(validateNativeTargetCustody(before, build).imageHash, build.generated.sha256);
  for (const mutate of [
    (x) => {
      x.emitted.filename = "/runner/unused-target/libvize_vitrine.so";
    },
    (x) => {
      x.emitted.filename = "/repo/target-neighbor/libvize_vitrine.so";
    },
    (x) => {
      x.emitted.filename = "/repo";
    },
    (x) => {
      x.emitted.cargo.fresh = true;
    },
    (x) => {
      x.environment.after.CARGO_TARGET_DIR = "/runner/unused-target";
    },
    (x) => {
      x.source.head = "c".repeat(40);
    },
    (x) => {
      x.request.runId = "2";
    },
    (x) => {
      x.frozen.sha256 = "c".repeat(64);
    },
  ]) {
    const changed = structuredClone(build);
    mutate(changed);
    assert.throws(() => validateNativeTargetCustody(before, changed));
  }
});
