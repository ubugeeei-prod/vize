import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";

import {
  cleanupProjectNative,
  stageProjectNative,
} from "../../npm/oxlint/scripts/project-native-staging.ts";

const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

test("the actual caller stages standalone and emits its exact owned path only when GitHub output exists", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-staging-caller-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const source = path.join(directory, "current.node");
  const sourceReceipt = path.join(directory, "receipt.json");
  const bytes = Buffer.from("current standalone source native binary\0");
  fs.writeFileSync(source, bytes);
  fs.writeFileSync(sourceReceipt, '{"source":"standalone caller"}\n');
  const caller = fs.readFileSync(
    fileURLToPath(new URL("../../npm/oxlint/scripts/check-project-transport.mjs", import.meta.url)),
    "utf8",
  );
  const start = caller.indexOf("const staged = stageProjectNative(");
  const end = caller.indexOf("const mixedDirectory8507 =", start);
  assert.ok(start >= 0 && end > start, "the complete live staging paragraph must be present");
  const context = {
    artifacts: undefined,
    stageProjectNative,
    receipt: { frozen: { path: source, sha256: sha256(bytes) } },
    nativeHistoryReceipt: () => sourceReceipt,
    path,
    root: directory,
    nativeDir: directory,
    fs,
  };
  runInNewContext(caller.slice(start, end), { ...context, process: { env: {} } });
  const artifacts = path.join(directory, "target/oxlint-original-project-transport");
  const standalone = fs.readdirSync(artifacts);
  assert.equal(standalone.length, 1);
  assert.deepEqual(
    fs.readFileSync(path.join(artifacts, standalone[0]!, "source-native.node")),
    bytes,
  );

  const output = path.join(directory, "github-output");
  runInNewContext(caller.slice(start, end), {
    ...context,
    process: { env: { GITHUB_OUTPUT: output } },
  });
  const staged = fs.readdirSync(artifacts).filter((name) => name !== standalone[0]);
  assert.equal(staged.length, 1);
  assert.equal(
    fs.readFileSync(output, "utf8"),
    `staging-directory=${path.join(artifacts, staged[0]!)}\nphase-walltime=${path.join(artifacts, staged[0]!, "phase-walltime.json")}\n`,
  );
  assert.deepEqual(fs.readFileSync(path.join(artifacts, staged[0]!, "source-native.node")), bytes);
});

test("cached fixed-target staging fails exclusively, while consecutive owned stages preserve each source", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-staging-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const artifacts = path.join(directory, "target/oxlint-original-project-transport");
  fs.mkdirSync(artifacts, { recursive: true });
  const stale = path.join(artifacts, "source-native.node");
  const staleBytes = Buffer.from("previous cached source native binary\0");
  fs.writeFileSync(stale, staleBytes);
  const priorEvidence = path.join(artifacts, "qualification.json");
  const priorBytes = Buffer.from('{"source":"previous whole qualification"}\n');
  fs.writeFileSync(priorEvidence, priorBytes);
  const source = path.join(directory, "current.node");
  const receipt = path.join(directory, "receipt.json");
  const firstBytes = Buffer.from("first current native binary\0");
  const firstReceipt = Buffer.from('{"source":"first exact source"}\n');
  fs.writeFileSync(source, firstBytes);
  fs.writeFileSync(receipt, firstReceipt);

  // The original live call at signed main9d fails before either real host starts.
  assert.throws(() => fs.copyFileSync(source, stale, fs.constants.COPYFILE_EXCL), {
    code: "EEXIST",
  });
  const first = stageProjectNative(artifacts, source, receipt, sha256(firstBytes));
  const secondBytes = Buffer.from("second different current native binary\0");
  const secondReceipt = Buffer.from('{"source":"second exact source"}\n');
  fs.writeFileSync(source, secondBytes);
  fs.writeFileSync(receipt, secondReceipt);
  const second = stageProjectNative(artifacts, source, receipt, sha256(secondBytes));

  assert.notEqual(first.artifacts, second.artifacts);
  assert.notEqual(first.binary, second.binary);
  for (const stage of [first, second]) {
    assert.equal(path.dirname(stage.artifacts), artifacts);
    assert.match(path.basename(stage.artifacts), /^run-/u);
    assert.equal(stage.binary, path.join(stage.artifacts, "source-native.node"));
  }
  assert.deepEqual(fs.readFileSync(first.binary), firstBytes);
  assert.deepEqual(fs.readFileSync(second.binary), secondBytes);
  assert.deepEqual(fs.readFileSync(path.join(first.artifacts, "build-receipt.json")), firstReceipt);
  assert.deepEqual(
    fs.readFileSync(path.join(second.artifacts, "build-receipt.json")),
    secondReceipt,
  );
  assert.deepEqual(fs.readFileSync(stale), staleBytes);
  assert.deepEqual(fs.readFileSync(priorEvidence), priorBytes);
  assert.throws(() => fs.copyFileSync(source, first.binary, fs.constants.COPYFILE_EXCL), {
    code: "EEXIST",
  });
  assert.deepEqual(fs.readFileSync(first.binary), firstBytes);
  assert.throws(() => cleanupProjectNative(artifacts, first.artifacts), { code: "ENOENT" });
  fs.writeFileSync(path.join(first.artifacts, "qualification.json"), '{"complete":true}\n');
  cleanupProjectNative(artifacts, first.artifacts);
  assert.equal(fs.existsSync(first.artifacts), false);
  assert.deepEqual(fs.readFileSync(second.binary), secondBytes);
  assert.deepEqual(fs.readFileSync(stale), staleBytes);
  assert.deepEqual(fs.readFileSync(priorEvidence), priorBytes);
});

test("wrong current source hash is rejected and preserves only its own partial evidence", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-staging-refusal-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const artifacts = path.join(directory, "transport");
  fs.mkdirSync(artifacts);
  const untouched = path.join(artifacts, "unowned-cached-run");
  fs.mkdirSync(untouched);
  fs.writeFileSync(path.join(untouched, "source-native.node"), "unowned previous source");
  const source = path.join(directory, "current.node");
  const receipt = path.join(directory, "receipt.json");
  const bytes = Buffer.from("current source fails expected binary hash\0");
  fs.writeFileSync(source, bytes);
  fs.writeFileSync(receipt, '{"source":"must not qualify"}\n');

  assert.throws(() => stageProjectNative(artifacts, source, receipt, "0".repeat(64)), {
    code: "ERR_ASSERTION",
  });
  const owned = fs.readdirSync(artifacts).filter((name) => name.startsWith("run-"));
  assert.equal(owned.length, 1);
  assert.deepEqual(fs.readFileSync(path.join(artifacts, owned[0]!, "source-native.node")), bytes);
  assert.equal(fs.existsSync(path.join(artifacts, owned[0]!, "build-receipt.json")), false);
  assert.equal(
    fs.readFileSync(path.join(untouched, "source-native.node"), "utf8"),
    "unowned previous source",
  );
  assert.throws(() => cleanupProjectNative(artifacts, artifacts), { code: "ERR_ASSERTION" });
  assert.throws(() => cleanupProjectNative(artifacts, untouched), { code: "ERR_ASSERTION" });
  assert.throws(() => cleanupProjectNative(artifacts, path.join(artifacts, owned[0]!)), {
    code: "ENOENT",
  });
  assert.deepEqual(fs.readFileSync(path.join(artifacts, owned[0]!, "source-native.node")), bytes);
});

test("cleanup refuses an outside directory and a symlink to an unowned completed stage", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-staging-cleanup-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const artifacts = path.join(directory, "transport");
  fs.mkdirSync(artifacts);
  const outside = fs.mkdtempSync(path.join(directory, "run-"));
  fs.writeFileSync(path.join(outside, "qualification.json"), '{"complete":true}\n');
  const link = path.join(artifacts, "run-123456");
  fs.symlinkSync(outside, link, "dir");
  assert.throws(() => cleanupProjectNative(artifacts, outside), { code: "ERR_ASSERTION" });
  assert.throws(() => cleanupProjectNative(artifacts, link), { code: "ERR_ASSERTION" });
  assert.equal(
    fs.readFileSync(path.join(outside, "qualification.json"), "utf8"),
    '{"complete":true}\n',
  );
  assert.equal(fs.lstatSync(link).isSymbolicLink(), true);
});
