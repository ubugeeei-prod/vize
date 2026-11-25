import assert from "node:assert/strict";
import { lstatSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import { committedGraph, verifyCommittedInventory } from "./canonical-corpus-inventory.mjs";
import { captureCorpus, sameCorpus, sha256, validateFiles } from "./canonical-corpus-identity.mjs";
import { nativeWalkIdentity } from "./canonical-corpus-finite-walk.mjs";
import {
  parseNativeVector,
  validateNativeSource,
  validateNativeWalk,
} from "./canonical-corpus-native-walk.ts";
import { qualifyRecapturedObservation } from "./canonical-corpus-recapture.ts";

const observationFiles = [
  "identity.json",
  "committed-corpus.json",
  "physical.json",
  "native-vector.nul",
  "native-collector-source.rs",
  "native-collector.rs",
  "native-rustc.txt",
  "native-collector",
];
const captureFiles = [
  "identity.json",
  "committed-corpus.json",
  "files.json",
  "native-walk.json",
  "original-kernel-vector.nul",
  "original-kernel-files.json",
  "native-vector.nul",
  "native-collector-source.rs",
  "native-collector.rs",
  "native-rustc.txt",
  "native-collector",
];
const sourceFiles = [
  "native-vector.nul",
  "native-collector-source.rs",
  "native-collector.rs",
  "native-rustc.txt",
  "native-collector",
];
const inputKeys = [
  "schema",
  "version",
  "repository",
  "runId",
  "attempt",
  "event",
  "sha",
  "tree",
  "providerSha",
  "repositoryId",
  "providerRepositoryId",
  "gitlinksSha256",
  "modulesSha256",
];
const json = (directory, name) => JSON.parse(readFileSync(join(directory, name), "utf8"));
function digests(directory, names) {
  return Object.fromEntries(
    names.map((name) => {
      const path = join(directory, name);
      assert(lstatSync(path).isFile(), "Non-regular recapture archive input");
      return [name, sha256(readFileSync(path))];
    }),
  );
}
function subdirectory(directory, name, prefix) {
  assert(
    typeof name === "string" && new RegExp(`^${prefix}[a-zA-Z0-9]+$`, "u").test(name),
    "Foreign recapture directory",
  );
  const path = join(directory, name);
  assert(
    lstatSync(path).isDirectory() && !lstatSync(path).isSymbolicLink(),
    "Foreign recapture archive",
  );
  return path;
}
function sameInputs(left, right) {
  for (const key of inputKeys) assert.deepEqual(left[key], right[key], `Recapture ${key} changed`);
  assert.deepEqual(left.gitlinks, right.gitlinks, "Recapture gitlinks changed");
}
function observation(directory, name, identity, proof, original, capture) {
  const path = subdirectory(directory, name, "native-observation-");
  const hashes = digests(path, observationFiles);
  sameInputs(identity, json(path, "identity.json"));
  const archivedProof = json(path, "committed-corpus.json");
  assert.deepEqual(
    archivedProof,
    {
      schema: proof.schema,
      version: proof.version,
      repositories: proof.repositories,
    },
    "Raw observation committed owner changed",
  );
  for (const name of sourceFiles)
    assert.deepEqual(
      readFileSync(join(path, name)),
      readFileSync(join(capture, name)),
      `Raw observation ${name} changed`,
    );
  const raw = json(path, "physical.json");
  assert.deepEqual(raw.files, original, "Raw observation original whole-byte vector changed");
  assert.deepEqual(raw.rejected, proof.kernelBoundary.rejected, "Raw observation boundary changed");
  return { path, raw, hashes };
}
function baseline(directory, identity, files, proof, observationName) {
  assert.equal(
    proof.traversal,
    "finite-physical-edges",
    "Recapture requires strict finite snapshot",
  );
  assert(identity.nativeWalk, "Missing strict snapshot native identity");
  assert.equal(
    sha256(JSON.stringify(proof)),
    identity.committedSha256,
    "Strict snapshot proof changed",
  );
  assert.equal(
    validateFiles(files, verifyCommittedInventory(proof, identity.gitlinks)),
    identity.filesSha256,
    "Strict snapshot files changed",
  );
  validateNativeWalk(directory, identity, proof, files);
  const original = json(directory, "original-kernel-files.json");
  const names = observationName
    ? [observationName]
    : readdirSync(directory).filter((name) => name.startsWith("native-observation-"));
  assert.equal(names.length, 1, "Missing or ambiguous strict snapshot observation");
  return {
    ...observation(directory, names[0], identity, proof, original, directory),
    name: names[0],
    original,
  };
}

// This relation carries TWO observations. Neither identity nor its raw boundary,
// original alias vector or whole-byte digest is rewritten to claim equality.
export function validateRecordRecapture(directory, identity, files, proof, relation) {
  assert(relation, "Missing snapshot-to-record recapture relation");
  assert.equal(relation.schema, "vize.canonical-record-recapture", "Foreign recapture relation");
  assert.equal(relation.version, 1, "Unknown recapture relation");
  assert.equal(relation.phase, "snapshot-to-record", "Foreign recapture phase");
  assert.equal(relation.nativeAcceptanceCredit, false, "Recapture is not native acceptance");
  sameCorpus(identity, relation.baselineIdentity);
  const initialDirectory = subdirectory(
    directory,
    relation.baselineObservation,
    "native-observation-",
  );
  const recaptureDirectory = subdirectory(directory, relation.directory, "record-recapture-");
  const observedDirectory = subdirectory(
    recaptureDirectory,
    relation.recaptureObservation,
    "native-observation-",
  );
  for (const [path, names, expected] of [
    [directory, captureFiles, relation.baselineArchiveSha256],
    [initialDirectory, observationFiles, relation.baselineObservationSha256],
    [recaptureDirectory, captureFiles, relation.recaptureArchiveSha256],
    [observedDirectory, observationFiles, relation.recaptureObservationSha256],
  ])
    assert.deepEqual(digests(path, names), expected, "Recapture archive bytes changed");
  const initial = baseline(directory, identity, files, proof, relation.baselineObservation);
  const currentIdentity = json(recaptureDirectory, "identity.json");
  const currentProof = json(recaptureDirectory, "committed-corpus.json");
  const currentFiles = json(recaptureDirectory, "files.json");
  sameCorpus(currentIdentity, relation.recaptureIdentity);
  sameInputs(identity, currentIdentity);
  for (const key of ["files", "filesSha256"])
    assert.deepEqual(identity[key], currentIdentity[key], `Finite recapture ${key} changed`);
  assert.deepEqual(
    currentProof.repositories,
    proof.repositories,
    "Recapture committed objects changed",
  );
  assert.equal(currentProof.schema, proof.schema, "Recapture proof schema changed");
  assert.equal(currentProof.version, proof.version, "Recapture proof version changed");
  assert.equal(currentProof.traversal, proof.traversal, "Recapture traversal changed");
  assert.equal(
    sha256(JSON.stringify(currentProof)),
    currentIdentity.committedSha256,
    "Recapture proof digest changed",
  );
  const { receipt, paths, original } = validateNativeSource(recaptureDirectory, currentIdentity);
  assert.deepEqual(
    nativeWalkIdentity(receipt),
    currentIdentity.nativeWalk,
    "Recapture native identity changed",
  );
  assert.deepEqual(
    receipt.boundary,
    currentProof.kernelBoundary,
    "Recapture native boundary changed",
  );
  const current = observation(
    recaptureDirectory,
    relation.recaptureObservation,
    currentIdentity,
    currentProof,
    original,
    recaptureDirectory,
  );
  for (const name of sourceFiles)
    assert.deepEqual(
      readFileSync(join(initial.path, name)),
      readFileSync(join(current.path, name)),
      `Recapture collector ${name} changed`,
    );
  const { nodes, children } = committedGraph(proof, identity.gitlinks);
  const qualified = qualifyRecapturedObservation(
    nodes,
    children,
    { raw: initial.raw, boundary: proof.kernelBoundary, files },
    current.raw,
    currentProof.kernelBoundary,
  );
  assert.equal(
    validateFiles(currentFiles, qualified.files),
    currentIdentity.filesSha256,
    "Recapture finite bytes changed",
  );
  assert.deepEqual(currentFiles, files, "Recapture finite whole-byte vector changed");
  assert.deepEqual(
    paths,
    files.map(([path]) => path),
    "Recapture finite native order changed",
  );
  const originalPaths = parseNativeVector(
    readFileSync(join(recaptureDirectory, "original-kernel-vector.nul")),
  );
  assert.deepEqual(
    originalPaths,
    current.raw.files.map(([path]) => path),
    "Recapture raw alias vector changed",
  );
  const expected = {
    schema: relation.schema,
    version: 1,
    phase: "snapshot-to-record",
    nativeAcceptanceCredit: false,
    baselineIdentity: identity,
    recaptureIdentity: currentIdentity,
    baselineObservation: initial.name,
    recaptureObservation: relation.recaptureObservation,
    directory: relation.directory,
    baselineArchiveSha256: digests(directory, captureFiles),
    baselineObservationSha256: initial.hashes,
    recaptureArchiveSha256: digests(recaptureDirectory, captureFiles),
    recaptureObservationSha256: current.hashes,
    cut: qualified.cut,
    finiteFiles: files.length,
  };
  assert.deepEqual(relation, expected, "Snapshot-to-record evidence relation changed");
  return relation;
}

export function captureRecordRecapture(cwd, env, directory, identity, files, proof) {
  const initial = baseline(directory, identity, files, proof);
  const destination = mkdtempSync(join(directory, "record-recapture-"));
  const current = captureCorpus(cwd, env, {
    directory: destination,
    basis: { raw: initial.raw, boundary: proof.kernelBoundary, files },
  });
  for (const [name, value] of [
    ["identity.json", current.identity],
    ["files.json", current.files],
    ["committed-corpus.json", current.proof],
  ])
    writeFileSync(join(destination, name), `${JSON.stringify(value)}\n`, { flag: "wx" });
  const names = readdirSync(destination).filter((name) => name.startsWith("native-observation-"));
  assert.equal(names.length, 1, "Missing or ambiguous record observation");
  const raw = json(join(destination, names[0]), "physical.json");
  const { nodes, children } = committedGraph(proof, identity.gitlinks);
  const qualified = qualifyRecapturedObservation(
    nodes,
    children,
    { raw: initial.raw, boundary: proof.kernelBoundary, files },
    raw,
    current.proof.kernelBoundary,
  );
  const relation = {
    schema: "vize.canonical-record-recapture",
    version: 1,
    phase: "snapshot-to-record",
    nativeAcceptanceCredit: false,
    baselineIdentity: identity,
    recaptureIdentity: current.identity,
    baselineObservation: initial.name,
    recaptureObservation: names[0],
    directory: basename(destination),
    baselineArchiveSha256: digests(directory, captureFiles),
    baselineObservationSha256: initial.hashes,
    recaptureArchiveSha256: digests(destination, captureFiles),
    recaptureObservationSha256: digests(join(destination, names[0]), observationFiles),
    cut: qualified.cut,
    finiteFiles: files.length,
  };
  return validateRecordRecapture(directory, identity, files, proof, relation);
}
