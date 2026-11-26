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
import type {
  Boundary,
  CorpusIdentity,
  CorpusProof,
  NativeObservation,
  VueFile,
} from "./canonical-corpus-types.ts";

export interface SnapshotIdentity extends CorpusIdentity {
  schema: string;
  version: number;
  repository: string;
  event?: string;
  providerSha: string;
  repositoryId: number;
  providerRepositoryId: number;
  gitlinksSha256: string;
  modulesSha256: string;
  files: number;
  filesSha256: string;
  committedSha256: string;
}
type FiniteProof = CorpusProof & { kernelBoundary: Boundary; traversal: "finite-physical-edges" };
function assertFiniteProof(proof: CorpusProof): asserts proof is FiniteProof {
  assert(proof.kernelBoundary, "Missing recapture kernel observation");
  assert.equal(proof.traversal, "finite-physical-edges", "Foreign recapture traversal");
}
type Digests = Record<string, string>;
export interface RecordRecaptureRelation {
  schema: "vize.canonical-record-recapture";
  version: 1;
  phase: "snapshot-to-record";
  nativeAcceptanceCredit: false;
  baselineIdentity: SnapshotIdentity;
  recaptureIdentity: SnapshotIdentity;
  baselineObservation: string;
  recaptureObservation: string;
  directory: string;
  baselineArchiveSha256: Digests;
  baselineObservationSha256: Digests;
  recaptureArchiveSha256: Digests;
  recaptureObservationSha256: Digests;
  cut: string[];
  finiteFiles: number;
}

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
] as const;
// Every parsed archive is authenticated below against its source and complete
// shape before it can qualify a record relation. Parsing grants no evidence.
const json = <T>(directory: string, name: string): T =>
  JSON.parse(readFileSync(join(directory, name), "utf8")) as T;
function digests(directory: string, names: readonly string[]): Digests {
  return Object.fromEntries(
    names.map((name) => {
      const path = join(directory, name);
      assert(lstatSync(path).isFile(), "Non-regular recapture archive input");
      return [name, sha256(readFileSync(path))];
    }),
  );
}
function subdirectory(directory: string, name: unknown, prefix: string): string {
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
function sameInputs(
  left: Pick<SnapshotIdentity, (typeof inputKeys)[number] | "gitlinks">,
  right: Pick<SnapshotIdentity, (typeof inputKeys)[number] | "gitlinks">,
) {
  for (const key of inputKeys) assert.deepEqual(left[key], right[key], `Recapture ${key} changed`);
  assert.deepEqual(left.gitlinks, right.gitlinks, "Recapture gitlinks changed");
}
function observation(
  directory: string,
  name: string,
  identity: SnapshotIdentity,
  proof: FiniteProof,
  original: VueFile[],
  capture: string,
) {
  const path = subdirectory(directory, name, "native-observation-");
  const hashes = digests(path, observationFiles);
  sameInputs(identity, json<SnapshotIdentity>(path, "identity.json"));
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
  const raw = json<NativeObservation>(path, "physical.json");
  assert.deepEqual(raw.files, original, "Raw observation original whole-byte vector changed");
  assert.deepEqual(raw.rejected, proof.kernelBoundary.rejected, "Raw observation boundary changed");
  return { path, raw, hashes };
}
function baseline(
  directory: string,
  identity: SnapshotIdentity,
  files: VueFile[],
  proof: FiniteProof,
  observationName?: string,
  cwd = process.cwd(),
) {
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
  validateNativeWalk(directory, identity, proof, files, cwd);
  const original = json<VueFile[]>(directory, "original-kernel-files.json");
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
export function validateRecordRecapture(
  directory: string,
  identity: SnapshotIdentity,
  files: VueFile[],
  proof: FiniteProof,
  value: unknown,
  cwd = process.cwd(),
): RecordRecaptureRelation {
  const relation = value;

  assert(relation, "Missing snapshot-to-record recapture relation");
  assert(
    typeof relation === "object" && !Array.isArray(relation),
    "Foreign recapture relation shape",
  );
  const fields = relation as Record<string, unknown>;
  assert.equal(fields.schema, "vize.canonical-record-recapture", "Foreign recapture relation");
  assert.equal(fields.version, 1, "Unknown recapture relation");
  assert.equal(fields.phase, "snapshot-to-record", "Foreign recapture phase");
  assert.equal(fields.nativeAcceptanceCredit, false, "Recapture is not native acceptance");
  sameCorpus(identity, fields.baselineIdentity);
  const initialDirectory = subdirectory(
    directory,
    fields.baselineObservation,
    "native-observation-",
  );
  const recaptureDirectory = subdirectory(directory, fields.directory, "record-recapture-");
  const observedDirectory = subdirectory(
    recaptureDirectory,
    fields.recaptureObservation,
    "native-observation-",
  );
  for (const [path, names, expected] of [
    [directory, captureFiles, fields.baselineArchiveSha256],
    [initialDirectory, observationFiles, fields.baselineObservationSha256],
    [recaptureDirectory, captureFiles, fields.recaptureArchiveSha256],
    [observedDirectory, observationFiles, fields.recaptureObservationSha256],
  ] satisfies Array<[string, readonly string[], unknown]>)
    assert.deepEqual(digests(path, names), expected, "Recapture archive bytes changed");
  assert(
    typeof fields.baselineObservation === "string" &&
      typeof fields.recaptureObservation === "string" &&
      typeof fields.directory === "string",
    "Foreign recapture archive names",
  );
  const initial = baseline(directory, identity, files, proof, fields.baselineObservation, cwd);
  const currentIdentity = json<SnapshotIdentity>(recaptureDirectory, "identity.json");
  const currentProof = json<FiniteProof>(recaptureDirectory, "committed-corpus.json");
  const currentFiles = json<VueFile[]>(recaptureDirectory, "files.json");
  sameCorpus(currentIdentity, fields.recaptureIdentity);
  sameInputs(identity, currentIdentity);
  for (const key of ["files", "filesSha256"] as const)
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
  const { receipt, paths, original } = validateNativeSource(
    recaptureDirectory,
    currentIdentity,
    cwd,
  );
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
  assert(original, "Missing original recapture archive");
  const current = observation(
    recaptureDirectory,
    fields.recaptureObservation,
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
  const expected: RecordRecaptureRelation = {
    schema: "vize.canonical-record-recapture",
    version: 1,
    phase: "snapshot-to-record",
    nativeAcceptanceCredit: false,
    baselineIdentity: identity,
    recaptureIdentity: currentIdentity,
    baselineObservation: initial.name,
    recaptureObservation: fields.recaptureObservation,
    directory: fields.directory,
    baselineArchiveSha256: digests(directory, captureFiles),
    baselineObservationSha256: initial.hashes,
    recaptureArchiveSha256: digests(recaptureDirectory, captureFiles),
    recaptureObservationSha256: current.hashes,
    cut: qualified.cut,
    finiteFiles: files.length,
  };
  assert.deepEqual(relation, expected, "Snapshot-to-record evidence relation changed");
  // Full deep equality with the strictly shaped, independently reconstructed
  // relation above is the runtime narrowing authority.
  return relation as RecordRecaptureRelation;
}

export function captureRecordRecapture(
  cwd: string,
  env: NodeJS.ProcessEnv,
  directory: string,
  identity: SnapshotIdentity,
  files: VueFile[],
  proof: FiniteProof,
): RecordRecaptureRelation {
  const initial = baseline(directory, identity, files, proof, undefined, cwd);
  const destination = mkdtempSync(join(directory, "record-recapture-"));
  const current = captureCorpus(cwd, env, {
    directory: destination,
    basis: { raw: initial.raw, boundary: proof.kernelBoundary, files },
  });
  const currentProof: CorpusProof = current.proof;
  assertFiniteProof(currentProof);
  for (const [name, value] of [
    ["identity.json", current.identity],
    ["files.json", current.files],
    ["committed-corpus.json", current.proof],
  ] satisfies Array<[string, unknown]>)
    writeFileSync(join(destination, name), `${JSON.stringify(value)}\n`, { flag: "wx" });
  const names = readdirSync(destination).filter((name) => name.startsWith("native-observation-"));
  assert.equal(names.length, 1, "Missing or ambiguous record observation");
  const raw = json<NativeObservation>(join(destination, names[0]), "physical.json");
  const { nodes, children } = committedGraph(proof, identity.gitlinks);
  const qualified = qualifyRecapturedObservation(
    nodes,
    children,
    { raw: initial.raw, boundary: proof.kernelBoundary, files },
    raw,
    currentProof.kernelBoundary,
  );
  const relation: RecordRecaptureRelation = {
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
  return validateRecordRecapture(directory, identity, files, proof, relation, cwd);
}
