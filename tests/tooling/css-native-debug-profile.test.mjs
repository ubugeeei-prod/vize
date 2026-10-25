import assert from "node:assert/strict";
import { test } from "node:test";
import { campaign } from "../../tools/benchmarks/scripts/css-native-attribution.mjs";
import {
  validateCompilerReceipt,
  validateDebugIdentity,
} from "../../tools/benchmarks/scripts/css-native-debug-build.mjs";
import {
  debugArchive,
  validateDebugArchive,
} from "../../tools/benchmarks/scripts/css-native-debug-replay.mjs";
import {
  savedAuthority,
  validateSavedAuthority,
} from "../../tools/benchmarks/scripts/css-native-saved-analysis.mjs";

await test("analysis admits only the actual complete controlled capture without new CPU credit", () => {
  const metadata = {
    id: savedAuthority.artifact,
    size_in_bytes: savedAuthority.bytes,
    digest: `sha256:${savedAuthority.sha256}`,
    expired: false,
    workflow_run: { id: savedAuthority.run, head_sha: savedAuthority.source },
  };
  validateSavedAuthority(metadata);
  for (const change of [
    (value) => {
      value.id++;
    },
    (value) => {
      value.size_in_bytes--;
    },
    (value) => {
      value.digest = "sha256:" + "0".repeat(64);
    },
    (value) => {
      value.expired = true;
    },
    (value) => {
      value.workflow_run.id++;
    },
    (value) => {
      value.workflow_run.head_sha = "f".repeat(40);
    },
  ]) {
    const changed = structuredClone(metadata);
    change(changed);
    assert.throws(() => validateSavedAuthority(changed));
  }
});

await test("replay admits only the actual failed-sampler archive with separately qualified producers", () => {
  const metadata = {
    id: debugArchive.artifact,
    size_in_bytes: debugArchive.bytes,
    digest: `sha256:${debugArchive.sha256}`,
    expired: false,
    workflow_run: { id: debugArchive.run, head_sha: debugArchive.harnessSource },
  };
  validateDebugArchive(metadata);
  for (const change of [
    (value) => {
      value.id++;
    },
    (value) => {
      value.size_in_bytes--;
    },
    (value) => {
      value.digest = "sha256:" + "0".repeat(64);
    },
    (value) => {
      value.expired = true;
    },
    (value) => {
      value.workflow_run.id++;
    },
    (value) => {
      value.workflow_run.head_sha = "f".repeat(40);
    },
  ]) {
    const changed = structuredClone(metadata);
    change(changed);
    assert.throws(() => validateDebugArchive(changed));
  }
});

function fixture() {
  const producer = {
    kind: "debug-source",
    side: "base",
    source: campaign.images.base.source,
    token: "a".repeat(32),
    target: "/owned-absent-target",
    targetInitiallyEmpty: true,
    profile: { inherits: "release", lto: "thin", codegenUnits: 16, debug: 2, strip: "none" },
    binarySha256: "b".repeat(64),
    debugInfo: true,
    emitted: {
      file: "/owned-absent-target/x86_64-unknown-linux-gnu/ci-opt/libvize_vitrine.so",
      sha256: "b".repeat(64),
    },
  };
  producer.compilerReceipts = [
    "vize_vitrine",
    "vize_atelier_sfc",
    "vize_croquis",
    "lightningcss",
  ].map((crate) => ({
    source: producer.source,
    side: producer.side,
    token: producer.token,
    target: producer.target,
    outDir: producer.target + "/x86_64-unknown-linux-gnu/ci-opt/deps",
    crate,
    args: [
      "--crate-name",
      crate,
      "--out-dir",
      producer.target + "/x86_64-unknown-linux-gnu/ci-opt/deps",
      "-C",
      "opt-level=3",
      "-C",
      "debuginfo=2",
      "-C",
      "codegen-units=16",
    ],
  }));
  return producer;
}

await test("debug source admission binds actual per-crate compiler flags and output directory", () => {
  const producer = fixture();
  validateDebugIdentity("base", producer, producer.binarySha256, producer.token);
  for (const change of [
    (value) => {
      value.outDir = "/unused-preflight-target/ci-opt/deps";
    },
    (value) => {
      value.args[value.args.indexOf("--out-dir") + 1] = "/foreign-output";
    },
    (value) => {
      value.args[value.args.indexOf("debuginfo=2")] = "debuginfo=0";
    },
    (value) => {
      value.args[value.args.indexOf("opt-level=3")] = "opt-level=0";
    },
    (value) => {
      value.args.push("-C", "strip=debuginfo");
    },
    (value) => {
      value.token = "c".repeat(32);
    },
    (value) => {
      value.source = "d".repeat(40);
    },
  ]) {
    const row = structuredClone(producer.compilerReceipts[0]);
    change(row);
    assert.throws(() => validateCompilerReceipt(row, producer));
  }
});

await test("debug images cannot inherit release-image or unused-target custody", () => {
  for (const change of [
    (value) => {
      value.source = "f".repeat(40);
    },
    (value) => {
      value.kind = "original-release";
    },
    (value) => {
      value.targetInitiallyEmpty = false;
    },
    (value) => {
      value.debugInfo = false;
    },
    (value) => {
      value.emitted.file = "/shared-target/libvize_vitrine.so";
    },
    (value) => {
      value.emitted.sha256 = "c".repeat(64);
    },
    (value) => {
      value.compilerReceipts.pop();
    },
    (value) => {
      value.profile.lto = false;
    },
  ]) {
    const producer = fixture();
    change(producer);
    assert.throws(() => validateDebugIdentity("base", producer, "b".repeat(64), "a".repeat(32)));
  }
  assert.throws(() => validateDebugIdentity("base", fixture(), "c".repeat(64), "a".repeat(32)));
  assert.throws(() => validateDebugIdentity("base", fixture(), "b".repeat(64), "c".repeat(32)));
});
