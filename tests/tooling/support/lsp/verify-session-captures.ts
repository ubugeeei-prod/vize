import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  BUILD_RECIPE,
  expectedBuildIdentity,
  validateBuildReceipt,
  type BuildIdentity,
} from "../../../differential/build-receipt.ts";
import { decodeFrames } from "../../../differential/lsp-wire.ts";
import { sha256 } from "../../../differential/sha256.ts";

type Snapshot = {
  version: 1;
  binary: BuildIdentity;
  prior: { directory: string; observationSHA256: string | null }[];
};
type Stream = "client" | "server" | "stderr";
export type CaptureObservation = {
  schema: string;
  version: number;
  state: string;
  sourceRevision: string;
  binary: BuildIdentity;
  buildReceipt: unknown;
  captureByteLimit: number;
  argv: string[];
  versionProbe: {
    exitStatus: number | null;
    signal: string | null;
    processError: string | null;
    stdoutBase64: string;
    stderrBase64: string;
  };
  sourceWitnesses: { path: string; line: number; column: number; sha256: string }[];
  process: { exitStatus: number | null; signal: string | null; error: string | null };
  streams: Record<
    Stream,
    {
      path: string;
      capturedBytes: number;
      observedBytes: number;
      sha256: string;
      truncated: boolean;
      framing?: { state: string; messageCount: number };
    }
  >;
};
const captureRoot = (root: string) => path.join(root, "target/differential/lsp-sessions");
const read = (file: string) => JSON.parse(fs.readFileSync(file, "utf8"));
const inventory = (root: string) => {
  if (!fs.existsSync(captureRoot(root))) return [];
  assert.ok(fs.lstatSync(captureRoot(root)).isDirectory());
  return fs.readdirSync(captureRoot(root), { withFileTypes: true }).map((entry) => {
    assert.ok(entry.isDirectory() && !entry.isSymbolicLink());
    const observation = path.join(captureRoot(root), entry.name, "observation.json");
    return {
      directory: entry.name,
      observationSHA256: fs.existsSync(observation) ? sha256(fs.readFileSync(observation)) : null,
    };
  });
};

/** Existing sticky-cache observations remain intact and cannot satisfy a fresh run. */
export function snapshotSessionCaptures(root: string): Snapshot {
  return { version: 1, binary: expectedBuildIdentity(root), prior: inventory(root) };
}

/** Read already captured bytes only; this never invokes the CLI or sends an RPC. */
export function verifySessionCaptures(root: string, snapshot: Snapshot) {
  const expected = expectedBuildIdentity(root);
  const receipt = read(path.join(root, `${expected.binaryPath}.differential-build.json`));
  validateBuildReceipt(receipt, expected, BUILD_RECIPE);
  assert.equal(snapshot.version, 1);
  assert.deepEqual(snapshot.binary, expected);
  const current = inventory(root);
  assert.equal(new Set(snapshot.prior.map((entry) => entry.directory)).size, snapshot.prior.length);
  const prior = new Set(snapshot.prior.map((entry) => entry.directory));
  for (const entry of snapshot.prior)
    assert.deepEqual(
      current.find((candidate) => candidate.directory === entry.directory),
      entry,
    );
  const fresh = current.filter((entry) => !prior.has(entry.directory));
  assert.ok(fresh.length > 0, "no fresh source-bound LSP sessions were captured");
  const sessions = fresh.map((entry) => {
    const directory = path.join(captureRoot(root), entry.directory);
    const observation: CaptureObservation = read(path.join(directory, "observation.json"));
    assert.equal(observation.schema, "vize.lsp.session.observation");
    assert.equal(observation.version, 1);
    assert.equal(observation.state, "process-closed");
    assert.equal(observation.sourceRevision, expected.sourceRevision);
    assert.deepEqual(observation.binary, expected);
    validateBuildReceipt(observation.buildReceipt, expected, BUILD_RECIPE);
    assert.deepEqual(observation.argv, ["lsp"]);
    assert.equal(observation.captureByteLimit, 16 * 1024 * 1024);
    const probe = observation.versionProbe;
    assert.equal(probe.exitStatus, 0);
    assert.equal(probe.signal, null);
    assert.equal(probe.processError, null);
    for (const field of ["stdoutBase64", "stderrBase64"] as const)
      assert.equal(Buffer.from(probe[field], "base64").toString("base64"), probe[field]);
    assert.equal(Buffer.from(probe.stdoutBase64, "base64").toString().trim(), expected.cliVersion);
    assert.equal(observation.process.error, null);
    assert.ok(
      (Number.isSafeInteger(observation.process.exitStatus) &&
        observation.process.exitStatus! >= 0 &&
        observation.process.signal === null) ||
        (observation.process.exitStatus === null &&
          typeof observation.process.signal === "string" &&
          /^SIG[A-Z0-9]+$/u.test(observation.process.signal)),
    );
    assert.ok(observation.sourceWitnesses.length > 0);
    for (const witness of observation.sourceWitnesses) {
      assert.ok(/^(?:tests\/tooling|tests\/snapshots)\//u.test(witness.path));
      assert.ok(!witness.path.split("/").includes("..") && !path.isAbsolute(witness.path));
      assert.ok(Number.isInteger(witness.line) && witness.line > 0);
      assert.ok(Number.isInteger(witness.column) && witness.column > 0);
      assert.equal(sha256(fs.readFileSync(path.join(root, witness.path))), witness.sha256);
    }
    assert.deepEqual(Object.keys(observation.streams).sort(), ["client", "server", "stderr"]);
    for (const stream of ["client", "server", "stderr"] as const) {
      const metadata = observation.streams[stream];
      assert.equal(metadata.path, `${stream}.bin`);
      const bytes = fs.readFileSync(path.join(directory, metadata.path));
      assert.equal(metadata.capturedBytes, bytes.length);
      assert.equal(metadata.observedBytes, bytes.length);
      assert.equal(metadata.sha256, sha256(bytes));
      assert.equal(metadata.truncated, false);
      if (stream !== "stderr") {
        const count = decodeFrames(bytes).messages.length;
        assert.ok(count > 0);
        assert.deepEqual(metadata.framing, { state: "complete", messageCount: count });
      }
    }
    return { ...entry, process: observation.process, sourceWitnesses: observation.sourceWitnesses };
  });
  return {
    version: 1,
    verified: true,
    binary: expected,
    prior: snapshot.prior,
    sessions,
    semanticAcceptance: false,
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [mode, input, output] = process.argv.slice(2);
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
  assert.ok(mode === "snapshot" || mode === "verify");
  assert.ok(input && (mode === "snapshot" ? !output : output));
  assert.equal(process.argv.length, mode === "snapshot" ? 4 : 5);
  const destination = mode === "snapshot" ? input : output!;
  try {
    const result =
      mode === "snapshot"
        ? snapshotSessionCaptures(root)
        : verifySessionCaptures(root, read(input));
    fs.writeFileSync(destination, `${JSON.stringify(result, null, 2)}\n`);
  } catch (error) {
    fs.writeFileSync(
      destination,
      `${JSON.stringify({ version: 1, verified: false, semanticAcceptance: false, error: String(error), stack: error instanceof Error ? error.stack : null }, null, 2)}\n`,
    );
    throw error;
  }
}
