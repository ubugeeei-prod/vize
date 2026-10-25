import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  BUILD_RECIPE,
  type BuildIdentity,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../differential/build-receipt.ts";
import { decodeFrames } from "../../../differential/lsp-wire.ts";
import { object, sha256 } from "../project-uri-membership-10622.ts";
import { snapshotSessionCaptures } from "./verify-session-captures.ts";

const git = (root: string, args: string[]) =>
  execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
export type IndependentObserverWitness = {
  authority: "independent-source-observer";
  repoRoot: string;
  revision: string;
  tree: string;
  path: string;
  line: number;
  column: number;
  sha256: string;
};

/** Bind the actual observer call site without assigning it to the producer checkout. */
export function independentObserverWitness(
  repoRoot: string,
  file: string,
  callerStack: string,
): IndependentObserverWitness {
  const relative = path.relative(repoRoot, file);
  assert.ok(relative.startsWith(`tests${path.sep}tooling${path.sep}`));
  const frame = callerStack.split("\n").flatMap((line) => {
    const value = /(file:\/\/[^\s)]+):(\d+):(\d+)\)?$/u.exec(line);
    return value && fileURLToPath(value[1]) === file ? [value] : [];
  })[0];
  assert.ok(frame, "the independent observer has an actual call-site witness");
  const witness: IndependentObserverWitness = {
    authority: "independent-source-observer",
    repoRoot,
    revision: git(repoRoot, ["rev-parse", "HEAD"]),
    tree: git(repoRoot, ["rev-parse", "HEAD^{tree}"]),
    path: relative.split(path.sep).join("/"),
    line: Number(frame[2]),
    column: Number(frame[3]),
    sha256: sha256(fs.readFileSync(file)),
  };
  validateObserverWitness(witness);
  return witness;
}

export function validateObserverWitness(witness: IndependentObserverWitness) {
  assert.equal(witness.authority, "independent-source-observer");
  assert.equal(fs.realpathSync(witness.repoRoot), witness.repoRoot);
  assert.match(witness.revision, /^[a-f0-9]{40}$/u);
  assert.match(witness.tree, /^[a-f0-9]{40}$/u);
  assert.equal(git(witness.repoRoot, ["rev-parse", "HEAD"]), witness.revision);
  assert.equal(git(witness.repoRoot, ["rev-parse", "HEAD^{tree}"]), witness.tree);
  assert.ok(witness.path.startsWith("tests/tooling/"));
  assert.ok(!witness.path.split("/").includes("..") && !path.isAbsolute(witness.path));
  assert.ok(Number.isSafeInteger(witness.line) && witness.line > 0);
  assert.ok(Number.isSafeInteger(witness.column) && witness.column > 0);
  const file = path.join(witness.repoRoot, witness.path);
  assert.equal(fs.realpathSync(file), file);
  assert.equal(
    sha256(fs.readFileSync(file)),
    witness.sha256,
    "the actual independent observer remains exact",
  );
  const lines = fs.readFileSync(file, "utf8").split("\n");
  assert.ok(witness.line <= lines.length, "the witness line belongs to the actual file");
  assert.ok(
    witness.column <= lines[witness.line - 1].length + 1,
    "the witness column belongs to the actual line",
  );
  const committed = execFileSync("git", ["show", `${witness.revision}:${witness.path}`], {
    cwd: witness.repoRoot,
  });
  assert.equal(
    sha256(committed),
    witness.sha256,
    "the witness belongs to its explicit observer revision",
  );
}

/** Exact closed outer bytes, without inventing a producer-root source witness. */
export function verifyIndependentObservation(directory: string, expected: BuildIdentity) {
  assert.equal(fs.lstatSync(directory).isSymbolicLink(), false);
  const observation = object(
    JSON.parse(fs.readFileSync(path.join(directory, "observation.json"), "utf8")),
  );
  assert.equal(observation.schema, "vize.lsp.session.observation");
  assert.equal(observation.version, 1);
  assert.equal(observation.state, "process-closed");
  assert.equal(observation.sourceRevision, expected.sourceRevision);
  assert.deepEqual(observation.binary, expected);
  validateBuildReceipt(observation.buildReceipt, expected, BUILD_RECIPE);
  assert.deepEqual(observation.argv, ["lsp"]);
  assert.equal(observation.captureByteLimit, 16 * 1024 * 1024);
  assert.deepEqual(
    observation.sourceWitnesses,
    [],
    "the observer is not relabeled as producer source",
  );
  const probe = object(observation.versionProbe);
  assert.equal(probe.exitStatus, 0);
  assert.equal(probe.signal, null);
  assert.equal(probe.processError, null);
  for (const field of ["stdoutBase64", "stderrBase64"]) {
    const encoded = probe[field];
    assert.ok(typeof encoded === "string");
    assert.equal(Buffer.from(encoded, "base64").toString("base64"), encoded);
  }
  const stdout = probe.stdoutBase64;
  assert.ok(typeof stdout === "string");
  assert.equal(Buffer.from(stdout, "base64").toString().trim(), expected.cliVersion);
  assert.deepEqual(observation.process, { exitStatus: 0, signal: null, error: null });
  const streams = object(observation.streams);
  assert.deepEqual(Object.keys(streams).sort(), ["client", "server", "stderr"]);
  for (const stream of ["client", "server", "stderr"]) {
    const metadata = object(streams[stream]);
    assert.equal(metadata.path, `${stream}.bin`);
    const file = path.join(directory, `${stream}.bin`);
    assert.equal(fs.lstatSync(file).isSymbolicLink(), false);
    const bytes = fs.readFileSync(file);
    assert.equal(metadata.capturedBytes, bytes.length);
    assert.equal(metadata.observedBytes, bytes.length);
    assert.equal(metadata.sha256, sha256(bytes));
    assert.equal(metadata.truncated, false);
    if (stream !== "stderr") {
      const messages = decodeFrames(bytes).messages;
      assert.ok(messages.length > 0);
      assert.deepEqual(metadata.framing, { state: "complete", messageCount: messages.length });
    }
  }
  return observation;
}

/**
 * The generic verifier intentionally requires a same-root source witness.
 * This experiment has two independently committed roots, so retain the empty
 * producer witness and validate the separate observer authority explicitly.
 * This does not grant generic source-session or semantic acceptance.
 */
export function verifyIndependentObserverCaptures(
  producerRoot: string,
  snapshot: ReturnType<typeof snapshotSessionCaptures>,
  observer: IndependentObserverWitness,
) {
  validateObserverWitness(observer);
  assert.notEqual(
    fs.realpathSync(producerRoot),
    observer.repoRoot,
    "dual-root observations require distinct roots",
  );
  const expected = expectedBuildIdentity(producerRoot);
  const receipt = JSON.parse(
    fs.readFileSync(
      path.join(producerRoot, `${expected.binaryPath}.differential-build.json`),
      "utf8",
    ),
  );
  validateBuildReceipt(receipt, expected, BUILD_RECIPE);
  assert.equal(snapshot.version, 1);
  assert.deepEqual(snapshot.binary, expected);
  const current = snapshotSessionCaptures(producerRoot).prior;
  const prior = new Set(snapshot.prior.map((entry) => entry.directory));
  assert.equal(prior.size, snapshot.prior.length);
  for (const entry of snapshot.prior)
    assert.deepEqual(
      current.find((row) => row.directory === entry.directory),
      entry,
    );
  const fresh = current.filter((entry) => !prior.has(entry.directory));
  assert.equal(fresh.length, 1, "each independent trial captures its one actual outer server");
  const sessions = fresh.map((entry) => {
    const directory = path.join(producerRoot, "target/differential/lsp-sessions", entry.directory);
    assert.equal(fs.lstatSync(directory).isSymbolicLink(), false);
    const observation = verifyIndependentObservation(directory, expected);
    return { ...entry, process: observation.process, sourceWitnesses: observation.sourceWitnesses };
  });
  validateObserverWitness(observer);
  return {
    authority: "independent-source-observer",
    producer: { repoRoot: producerRoot, receipt, binary: expected },
    observer,
    sessions,
    genericSourceWitnessAdmission: false,
    semanticAcceptance: false,
  };
}
