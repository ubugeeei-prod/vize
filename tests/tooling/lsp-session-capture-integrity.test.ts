import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { pathToFileURL } from "node:url";
import { writeBuildReceipt } from "../differential/build-receipt.ts";
import { frameMessage } from "../differential/lsp-wire.ts";
import { resolveVizeLaunchCommand, type VerifiedLspLaunch } from "./support/lsp/launch.ts";
import { LspSessionCapture } from "./support/lsp/session-capture.ts";
import {
  snapshotSessionCaptures,
  verifySessionCaptures,
  type CaptureObservation,
} from "./support/lsp/verify-session-captures.ts";

function fixture(t: TestContext) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-capture-integrity-law-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const git = (...args: string[]) => execFileSync("git", args, { cwd: root });
  git("init", "-q");
  git("config", "user.name", "CI Test");
  git("config", "user.email", "ci@example.invalid");
  fs.writeFileSync(path.join(root, "Cargo.toml"), '[workspace.package]\nversion = "1.0.0"\n');
  fs.mkdirSync(path.join(root, "tests/tooling"), { recursive: true });
  const witness = path.join(root, "tests/tooling/witness.test.ts");
  fs.writeFileSync(witness, "// Synthetic capture-integrity law; no product acceptance.\n");
  git("add", ".");
  git("commit", "-qm", "synthetic integrity source");
  const binary = path.join(root, "target/ci/vize");
  fs.mkdirSync(path.dirname(binary), { recursive: true });
  fs.writeFileSync(binary, '#!/bin/sh\nprintf "vize 1.0.0-dev\\n"\n', { mode: 0o755 });
  writeBuildReceipt(root);
  let launch: VerifiedLspLaunch | undefined;
  resolveVizeLaunchCommand(undefined, binary, {
    required: true,
    repoRoot: root,
    onVerifiedLaunch: (verified) => {
      launch = verified;
    },
  });
  assert.ok(launch);
  const verifiedLaunch = launch;
  const capture = () =>
    new LspSessionCapture({
      repoRoot: root,
      outputRoot: path.join(root, "target/differential/lsp-sessions"),
      launch: verifiedLaunch,
      callerStack: `Error\n    at fixture (${pathToFileURL(witness).href}:1:1)`,
    });
  const complete = (exitStatus = 0) => {
    const session = capture();
    session.append("client", frameMessage({ jsonrpc: "2.0", id: 1, method: "initialize" }));
    session.append("server", frameMessage({ jsonrpc: "2.0", id: 1, result: { fixture: true } }));
    session.append("stderr", Buffer.from("synthetic retained stderr\n"));
    session.finish(exitStatus, null);
    return session.directory;
  };
  return { root, capture, complete, witness, binary };
}

test("fresh whole captures retain failed process status without semantic acceptance or old cache credit", (t) => {
  const f = fixture(t);
  const old = f.complete();
  const priorBytes = fs.readFileSync(path.join(old, "observation.json"));
  const snapshot = snapshotSessionCaptures(f.root);
  assert.throws(() => verifySessionCaptures(f.root, snapshot), /no fresh/u);
  const fresh = f.complete(23);
  const result = verifySessionCaptures(f.root, snapshot);
  assert.equal(result.sessions.length, 1);
  assert.equal(result.sessions[0].directory, path.basename(fresh));
  assert.equal(result.sessions[0].process.exitStatus, 23);
  assert.equal(result.prior.length, 1);
  assert.equal(result.semanticAcceptance, false);
  assert.deepEqual(fs.readFileSync(path.join(old, "observation.json")), priorBytes);
});

test("missing captures and unclosed or truncated whole streams fail explicitly", (t) => {
  const f = fixture(t);
  const snapshot = snapshotSessionCaptures(f.root);
  assert.throws(() => verifySessionCaptures(f.root, snapshot), /no fresh/u);
  const capture = f.capture();
  assert.throws(() => verifySessionCaptures(f.root, snapshot), /process-closed/u);
  capture.append("client", Buffer.alloc(16 * 1024 * 1024 + 1, 120));
  capture.append("server", frameMessage({ jsonrpc: "2.0", id: 1, result: null }));
  capture.finish(0, null);
  assert.throws(() => verifySessionCaptures(f.root, snapshot));
  const raw = JSON.parse(fs.readFileSync(path.join(capture.directory, "observation.json"), "utf8"));
  assert.equal(raw.streams.client.truncated, true);
  assert.equal(raw.streams.server.truncated, true);
});

test("receipt, source, version, witness, lifecycle and framing substitutions refuse", (t) => {
  const f = fixture(t);
  const snapshot = snapshotSessionCaptures(f.root);
  const directory = f.complete();
  const file = path.join(directory, "observation.json");
  const bytes = fs.readFileSync(file);
  const mutations = [
    (row: CaptureObservation) => {
      row.sourceRevision = "0".repeat(40);
    },
    (row: CaptureObservation) => {
      row.binary.binarySha256 = "0".repeat(64);
    },
    (row: CaptureObservation) => {
      (row.buildReceipt as { recipe: string }).recipe = "cargo build --release -p vize";
    },
    (row: CaptureObservation) => {
      row.versionProbe.exitStatus = 1;
    },
    (row: CaptureObservation) => {
      row.versionProbe.stdoutBase64 = Buffer.from("vize 9.9.9").toString("base64");
    },
    (row: CaptureObservation) => {
      row.sourceWitnesses[0].sha256 = "0".repeat(64);
    },
    (row: CaptureObservation) => {
      row.sourceWitnesses[0].path = "tests/tooling/../../Cargo.toml";
    },
    (row: CaptureObservation) => {
      row.sourceWitnesses = [];
    },
    (row: CaptureObservation) => {
      row.process.error = "original capture error";
    },
    (row: CaptureObservation) => {
      row.process.exitStatus = null;
    },
    (row: CaptureObservation) => {
      row.streams.client.path = "../client.bin";
    },
    (row: CaptureObservation) => {
      row.streams.server.framing!.messageCount++;
    },
    (row: CaptureObservation) => {
      row.streams.server.framing!.state = "invalid-or-truncated";
    },
    (row: CaptureObservation) => {
      row.streams.stderr.sha256 = "0".repeat(64);
    },
    (row: CaptureObservation) => {
      row.streams.stderr.truncated = true;
    },
    (row: CaptureObservation) => {
      row.streams.stderr.observedBytes++;
    },
  ];
  for (const mutate of mutations) {
    const row = JSON.parse(bytes.toString());
    mutate(row);
    fs.writeFileSync(file, JSON.stringify(row));
    assert.throws(() => verifySessionCaptures(f.root, snapshot));
  }
  fs.writeFileSync(file, bytes);
  assert.equal(verifySessionCaptures(f.root, snapshot).sessions.length, 1);
});

test("raw byte changes, missing streams and changed prior observations cannot qualify", (t) => {
  const f = fixture(t);
  const old = f.complete();
  const snapshot = snapshotSessionCaptures(f.root);
  const current = f.complete();
  const raw = path.join(current, "server.bin");
  const bytes = fs.readFileSync(raw);
  fs.writeFileSync(raw, Buffer.concat([bytes, Buffer.from("unrecorded")]));
  assert.throws(() => verifySessionCaptures(f.root, snapshot));
  fs.rmSync(raw);
  assert.throws(() => verifySessionCaptures(f.root, snapshot));
  fs.writeFileSync(raw, bytes);
  const oldRecord = path.join(old, "observation.json");
  fs.appendFileSync(oldRecord, " ");
  assert.throws(() => verifySessionCaptures(f.root, snapshot));
});
