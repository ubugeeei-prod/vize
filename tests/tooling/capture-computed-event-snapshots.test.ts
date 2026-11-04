import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { at, parseJson } from "../../tools/support/json-value.ts";
import {
  captureComputedEvents,
  computedEventFiles,
  computedEventFilter,
  selectCanonTestBinary,
} from "../../tools/support/compat/github/capture-computed-event-snapshots.ts";

function fixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "canon-observation-"));
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  git("init", "-q");
  git("config", "user.name", "Observation fixture");
  git("config", "user.email", "fixture@example.invalid");
  for (const file of computedEventFiles) {
    fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    fs.writeFileSync(path.join(root, file), "original exact snapshot\n");
  }
  git("add", ".");
  git("commit", "-qm", "fixture");
  const deps = path.join(root, "target/ci/deps");
  fs.mkdirSync(deps, { recursive: true });
  const binary = path.join(deps, "vize_canon-fixture");
  const pending = path.join(root, `${computedEventFiles[1]}.new`);
  fs.writeFileSync(
    binary,
    `#!${process.execPath}
const fs = require('node:fs');
if (process.argv.includes('--list')) {
  console.log(${JSON.stringify(`${computedEventFilter}: test`)});
} else {
  const assert = require('node:assert/strict');
  assert.deepEqual(process.argv.slice(2), ${JSON.stringify([computedEventFilter, "--exact", "--nocapture", "--test-threads=1"])});
  assert.equal(process.env.INSTA_UPDATE, 'new');
  assert.equal(process.env.INSTA_FORCE_PASS, '1');
  fs.writeFileSync(${JSON.stringify(pending)}, ${JSON.stringify("fresh observed body\n")});
  console.log('test result: ok. 1 passed; 0 failed;');
}
`,
  );
  fs.chmodSync(binary, 0o755);
  const message = {
    reason: "compiler-artifact",
    package_id: `path+file://${root}/crates/vize_canon#0.474.0`,
    target: { name: "vize_canon", kind: ["lib"] },
    profile: { test: true },
    executable: binary,
  };
  const evidence = path.join(root, "evidence");
  fs.mkdirSync(evidence);
  fs.writeFileSync(path.join(evidence, "cargo.jsonl"), `${JSON.stringify(message)}\n`);
  return { root, binary, message, evidence, source: git("rev-parse", "HEAD") };
}

void test("capture reuses one binary and labels generated observations separately from matched references", () => {
  const f = fixture();
  try {
    const receipt = captureComputedEvents(f.root, f.evidence, f.source);
    assert.equal(receipt.qualified, false);
    assert.equal(receipt.mode, "artifact-only");
    assert.equal(receipt.status, 0);
    assert.equal(receipt.observations.length, 8);
    assert.equal(
      receipt.observations.filter((x) => x.origin === "generated-pending-snapshot").length,
      1,
    );
    assert.equal(
      receipt.observations.filter((x) => x.origin === "reference-matched-by-complete-filter")
        .length,
      7,
    );
    for (const file of computedEventFiles)
      assert.equal(fs.readFileSync(path.join(f.root, file), "utf8"), "original exact snapshot\n");
    assert.throws(
      () => captureComputedEvents(f.root, f.evidence, "0".repeat(40)),
      /source changed/,
    );
  } finally {
    fs.rmSync(f.root, { recursive: true, force: true });
  }
});

void test("binary selection rejects missing, foreign, non-test and ambiguous Cargo artifacts", () => {
  const f = fixture();
  try {
    const select = (message: unknown) => selectCanonTestBinary(JSON.stringify(message), f.root);
    assert.equal(
      selectCanonTestBinary(`null\nnot JSON\n${JSON.stringify(f.message)}\n`, f.root),
      fs.realpathSync(f.binary),
    );
    for (const message of [
      { ...f.message, target: { name: "other", kind: ["lib"] } },
      { ...f.message, target: { name: "vize_canon", kind: ["bin"] } },
      { ...f.message, profile: { test: false } },
      { ...f.message, package_id: "other 0.474.0" },
    ])
      assert.throws(() => select(message), /require one/);
    const second = path.join(path.dirname(f.binary), "vize_canon-other");
    fs.copyFileSync(f.binary, second);
    assert.throws(
      () =>
        selectCanonTestBinary(
          `${JSON.stringify(f.message)}\n${JSON.stringify({ ...f.message, executable: second })}`,
          f.root,
        ),
      /require one/,
    );
    const foreign = path.join(f.root, "vize_canon-foreign");
    fs.copyFileSync(f.binary, foreign);
    assert.throws(() => select({ ...f.message, executable: foreign }), /foreign test binary/);
  } finally {
    fs.rmSync(f.root, { recursive: true, force: true });
  }
});

void test("an artifact-only execution cannot modify a checked-in snapshot", () => {
  const f = fixture();
  try {
    const original = fs.readFileSync(f.binary, "utf8");
    fs.writeFileSync(
      f.binary,
      original.replace(
        JSON.stringify(path.join(f.root, `${computedEventFiles[1]}.new`)),
        JSON.stringify(path.join(f.root, computedEventFiles[1]!)),
      ),
    );
    assert.throws(() => captureComputedEvents(f.root, f.evidence, f.source), /source changed/);
    const receipt = parseJson(
      fs.readFileSync(path.join(f.evidence, "computed-event-observations/receipt.json"), "utf8"),
    );
    assert.equal(at(receipt, "qualified"), false);
    assert.equal(at(receipt, "formalSnapshotsUnchanged"), false);
    const observations = at(receipt, "observations");
    assert(Array.isArray(observations));
    assert.equal(
      observations.filter((item) => at(item, "origin") === "reference-matched-by-complete-filter")
        .length,
      0,
    );
  } finally {
    fs.rmSync(f.root, { recursive: true, force: true });
  }
});
