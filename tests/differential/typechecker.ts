import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  loadProductManifest,
  readPinnedArtifact,
  sha256,
  validateResultEnvelope,
} from "./harness.mjs";

import {
  artDisplayPack,
  selectArtDiagnosticAuthority,
  type ArtDiagnosticAuthority,
} from "./typechecker-art-display.ts";
import { packCases, TYPECHECKER_TESTS, sourceRevisions } from "./typechecker-registry.ts";
import {
  typecheckerDialectCoverage,
  verifiedTypecheckerDialectCoverage,
  type DialectCoverage,
} from "./typechecker-dialect.ts";
export { TYPECHECKER_TESTS } from "./typechecker-registry.ts";
export { assertVerifiedTypecheckerDialectExecutions } from "./typechecker-dialect.ts";
export type Artifact = { path: string; sha256: string };
export type Diagnostic = {
  file: string;
  line: number;
  column: number;
  severity: number;
  code: number | null;
  message: string;
};
export type TypecheckerCase = {
  id: string;
  targets: string[];
  adapters: { legacy: string; native: null };
  state: string;
  pack: keyof typeof TYPECHECKER_TESTS;
  case: string;
  test: string;
  regressionCommit: string;
  sourceRevision?: string;
  checkerOptions?: { optionsApi?: boolean; jsxTypecheck?: boolean };
  projectOptions?: { vuePackage: "absent" };
  packReference: Artifact;
  inputs: Array<Artifact & { file: string }>;
  dialectCoverage: DialectCoverage;
  diagnostics: Diagnostic[];
  diagnosticAuthority?: ArtDiagnosticAuthority;
};
type Pack = {
  version: number;
  issue: number;
  sourceRevision: string;
  diagnosticContract: typeof contract;
  regressionCommit: string;
  checkerOptions?: { optionsApi?: boolean; jsxTypecheck?: boolean };
  projectOptions?: { vuePackage: "absent" };
  cases: Array<{
    id: string;
    inputs: Array<{ file: string; source: string }>;
    diagnostics: Diagnostic[];
  }>;
};
export type Capture = {
  schema: string;
  version: number;
  pack: keyof typeof TYPECHECKER_TESTS;
  test: string;
  fixturePackSha256: string;
  archiveReceiptSha256: string;
  binarySha256: string;
  binaryPath: string;
  missingFields: string[];
  matchedContract: string;
  unbaselinedFields: string[];
  cases: Array<{
    id: string;
    projectRoot: string;
    inputs: Array<{ file: string; sha256: string }>;
    diagnostics: Diagnostic[];
    publicResult: {
      exitCode: number;
      success: boolean;
      diagnostics: Array<Diagnostic & { blockType: string | null }>;
    };
  }>;
};
export type WorkspaceCapture = Omit<Capture, "archiveReceiptSha256"> & {
  version: 2;
  kind: "full-workspace";
  workspaceReceiptSha256: string;
};

export type WorkspaceExecution = { runId: number; runAttempt: number };
export type WorkspaceSource = {
  sha: string;
  tree: string;
  files: Record<string, string>;
  rustc: string;
  cargo: string;
  platform: string;
  arch: string;
  workspaceRoot: string;
  runtime: Record<string, string | null>;
  inferredProfileDefaults: unknown;
};
export type WorkspaceReceipt = {
  schema: "vize.typechecker-workspace-receipt";
  version: 1;
  kind: "full-workspace";
  source: WorkspaceSource;
  execution: WorkspaceExecution;
  cargoProfile: "ci";
  nextestVersion: "0.9.146";
  inventorySha256: string;
  rawInventorySha256: string;
  manifestSha256: string;
  ts40MatrixSha256: string;
  binary: {
    id: string;
    package: string;
    name: string;
    listedPath: string;
    path: string;
    sha256: string;
  };
};

export function validateWorkspaceProducerExecution(value: WorkspaceExecution) {
  for (const number of Object.values(value)) assert(Number.isSafeInteger(number) && number > 0);
  assert.deepEqual(Object.keys(value).sort(), ["runAttempt", "runId"]);
}

export type WorkspaceInventory = {
  source: WorkspaceSource;
  cargoProfile: string;
  rawFile: string;
  rawSha256: string;
  binaries: Array<{
    id: string;
    package: string;
    binary: string;
    cases: Array<{ name: string; ignored: boolean }>;
  }>;
};
export type LoadedTypechecker = {
  manifest: { product: string; baseRevision: string };
  manifestSha256: string;
  cases: TypecheckerCase[];
};

const contract = {
  requiredTier: "T1",
  positionBase: 1,
  positions: "authored UTF16 start",
  comparison: "all production diagnostics in returned order",
  missingFields: ["end", "relatedInformation", "raw backend diagnostics"],
  native: "unsupported",
};
const matchedContract = "batch-start-diagnostics-v1";
const unbaselinedFields = ["diagnostics[].blockType", "exitCode", "success"];
// Each pack remains an independently authored oracle. Registration adds source
// identities and shared accounting; it never recaptures the expected values.
export function loadTypecheckerManifest(manifestPath: string) {
  const loaded = loadProductManifest(manifestPath, "typechecker");
  assert.equal(loaded.manifest.contract, "batch-start-diagnostics-v1");
  assert.equal(loaded.manifest.issue, 6879);
  const root = path.dirname(manifestPath);
  const packs = new Map<string, Pack>();
  const cases = packCases(loaded.cases as TypecheckerCase[]).map((fixture) => {
    assert.deepEqual(fixture.targets, ["batch"]);
    assert.equal(fixture.adapters.legacy, "batch-typechecker-fixture-v1");
    assert.equal(fixture.adapters.native, null);
    assert.equal(fixture.state, "active");
    assert(Object.hasOwn(TYPECHECKER_TESTS, fixture.pack), "unregistered test binary body");
    assert.equal(fixture.test, TYPECHECKER_TESTS[fixture.pack]);
    assert.equal(fixture.packReference.path, `${fixture.pack}/cases.json`);
    const bytes = readPinnedArtifact(root, fixture.packReference);
    const pack: Pack = JSON.parse(bytes.toString("utf8"));
    assert.equal(pack.version, 1);
    assert.equal(pack.issue, 6879);
    assert.equal(pack.sourceRevision, sourceRevisions[fixture.pack]);
    assert.equal(pack.sourceRevision, fixture.sourceRevision ?? loaded.manifest.baseRevision);
    assert.deepEqual(pack.diagnosticContract, contract);
    assert.deepEqual(
      pack.checkerOptions,
      fixture.pack === "options-api-any-instance" ||
        fixture.pack === "options-api-reserved-component-reference" ||
        fixture.pack === "setup-custom-properties" ||
        fixture.pack === "required-default-props"
        ? { optionsApi: true }
        : fixture.pack === "jsx-prop-token-ranges-10494"
          ? { jsxTypecheck: true }
          : undefined,
      "the original explicit checker options must be retained",
    );
    assert.deepEqual(fixture.checkerOptions, pack.checkerOptions);
    assert.deepEqual(
      pack.projectOptions,
      fixture.pack === "authored-unused-symbols" ? { vuePackage: "absent" } : undefined,
      "the original absent Vue package must be retained",
    );
    assert.deepEqual(fixture.projectOptions, pack.projectOptions);
    const original = pack.cases.find((item) => item.id === fixture.case);
    assert(original, "case is absent from its immutable fixture pack");
    assert.equal(fixture.id, `typechecker/${fixture.pack}/${fixture.case}`);
    assert.equal(fixture.regressionCommit, pack.regressionCommit);
    assert.deepEqual(
      fixture.inputs.map((input) => [input.file, input.path]),
      original.inputs.map((input) => [input.file, `${fixture.pack}/${input.source}`]),
    );
    const inspectedCoverage = typecheckerDialectCoverage(root, fixture.inputs);
    assert.deepEqual(
      fixture.dialectCoverage,
      fixture.dialectCoverage.state === "verified"
        ? verifiedTypecheckerDialectCoverage(inspectedCoverage, fixture.test)
        : inspectedCoverage,
      "dialect coverage must match the pinned authored sources and registered execution",
    );
    packs.set(fixture.pack, pack);
    return { ...fixture, diagnostics: original.diagnostics };
  });
  // Both registries follow the sorted manifest packs, so new packs insert apart.
  assert.deepEqual([...packs.keys()], Object.keys(TYPECHECKER_TESTS));
  assert.deepEqual(Object.keys(sourceRevisions), Object.keys(TYPECHECKER_TESTS));
  for (const [name, pack] of packs) {
    assert.deepEqual(
      cases.filter((fixture) => fixture.pack === name).map((fixture) => fixture.case),
      pack.cases.map((fixture) => fixture.id),
      "missing, duplicated or reordered historical projects",
    );
  }
  return {
    ...loaded,
    cases: cases.map((fixture) =>
      fixture.pack === artDisplayPack ? selectArtDiagnosticAuthority(root, fixture) : fixture,
    ),
  };
}

function relativePath(input: string) {
  assert.equal(typeof input, "string");
  assert(input.length > 0 && !path.isAbsolute(input));
  assert(input.split(/[\\/]/).every((part) => part && part !== "." && part !== ".."));
}

function publicResultProjection(
  actual: Capture["cases"][number]["publicResult"],
  projectRoot: string,
) {
  assert.equal(typeof projectRoot, "string");
  const paths = /^[A-Za-z]:[\\/]|^\\\\/.test(projectRoot) ? path.win32 : path;
  assert(paths.isAbsolute(projectRoot));
  assert(actual && typeof actual === "object", "actual public result is required");
  assert.deepEqual(Object.keys(actual).sort(), ["diagnostics", "exitCode", "success"]);
  assert(
    Number.isInteger(actual.exitCode) &&
      actual.exitCode >= -2147483648 &&
      actual.exitCode <= 2147483647,
  );
  assert.equal(typeof actual.success, "boolean");
  assert(Array.isArray(actual.diagnostics));
  return actual.diagnostics.map((diagnostic) => {
    assert.deepEqual(Object.keys(diagnostic).sort(), [
      "blockType",
      "code",
      "column",
      "file",
      "line",
      "message",
      "severity",
    ]);
    assert(
      diagnostic.blockType === null ||
        ["template", "script", "scriptSetup", "style"].includes(diagnostic.blockType),
    );
    const { blockType: _blockType, ...fields } = diagnostic;
    assert(paths.isAbsolute(fields.file));
    const file = paths.relative(projectRoot, fields.file).replaceAll("\\", "/");
    relativePath(file);
    return { ...fields, file, line: fields.line + 1, column: fields.column + 1 };
  });
}

export function validateTypecheckerCapture(
  loaded: LoadedTypechecker,
  capture: Capture,
  { receipt, binarySha256 }: { receipt: Buffer; binarySha256: string },
) {
  assert.equal(capture.schema, "vize.typechecker-fixture-observation");
  assert.equal(capture.version, 1);
  assert(!Object.hasOwn(capture, "workspaceReceiptSha256") && !Object.hasOwn(capture, "kind"));
  assert.equal(capture.archiveReceiptSha256, sha256(receipt));
  validateCaptureBody(loaded, capture, binarySha256);
  return capture;
}

function validateCaptureBody(
  loaded: LoadedTypechecker,
  capture: Capture | WorkspaceCapture,
  binarySha256: string,
) {
  assert(Object.hasOwn(TYPECHECKER_TESTS, capture.pack));
  const planned = loaded.cases.filter((fixture) => fixture.pack === capture.pack);
  assert(planned.length > 0);
  assert.equal(capture.test, TYPECHECKER_TESTS[capture.pack]);
  assert.equal(capture.fixturePackSha256, planned[0].packReference.sha256);
  assert.equal(capture.binarySha256, binarySha256);
  assert.deepEqual(capture.missingFields, contract.missingFields);
  assert.equal(capture.matchedContract, matchedContract);
  assert.deepEqual(capture.unbaselinedFields, unbaselinedFields);
  assert.deepEqual(
    capture.cases.map((item) => item.id),
    planned.map((item) => item.case),
  );
  for (const [index, observed] of capture.cases.entries()) {
    assert.deepEqual(observed.diagnostics, planned[index].diagnostics);
    const projection = publicResultProjection(observed.publicResult, observed.projectRoot);
    assert.deepEqual(
      projection,
      observed.diagnostics,
      "public diagnostics must retain the original entire ordered projection",
    );
    assert.deepEqual(
      observed.inputs,
      planned[index].inputs.map((input) => ({
        file: input.file,
        sha256: input.sha256,
      })),
    );
    for (const input of observed.inputs) relativePath(input.file);
  }
}

export function validateWorkspaceTypecheckerCapture(
  loaded: LoadedTypechecker,
  capture: WorkspaceCapture,
  { receipt, binarySha256 }: { receipt: Buffer; binarySha256: string },
) {
  assert.equal(capture.schema, "vize.typechecker-fixture-observation");
  assert.equal(capture.version, 2);
  assert.equal(capture.kind, "full-workspace");
  assert.deepEqual(
    Object.keys(capture).sort(),
    [
      "schema",
      "version",
      "kind",
      "pack",
      "test",
      "fixturePackSha256",
      "workspaceReceiptSha256",
      "binarySha256",
      "binaryPath",
      "missingFields",
      "matchedContract",
      "unbaselinedFields",
      "cases",
    ].sort(),
  );
  assert(!Object.hasOwn(capture, "archiveReceiptSha256"));
  assert.equal(capture.workspaceReceiptSha256, sha256(receipt));
  validateCaptureBody(loaded, capture, binarySha256);
  return capture;
}

// The receipt was verified against the actual transferred Cargo archive before
// execution. This consumer also requires successful JUnit cases, so a writer or
// an expected-only object cannot confer runtime acceptance on its own.
export function typecheckerReport(
  loaded: LoadedTypechecker,
  captures: Capture[],
  sourceRevision: string,
) {
  for (const capture of captures) {
    assert.equal(capture.version, 1);
    assert(!Object.hasOwn(capture, "workspaceReceiptSha256") && !Object.hasOwn(capture, "kind"));
  }
  return capturedReport(loaded, captures, sourceRevision);
}

export function workspaceTypecheckerReport(
  loaded: LoadedTypechecker,
  captures: WorkspaceCapture[],
  sourceRevision: string,
) {
  for (const capture of captures) {
    assert.equal(capture.version, 2);
    assert.equal(capture.kind, "full-workspace");
    assert(!Object.hasOwn(capture, "archiveReceiptSha256"));
  }
  return capturedReport(loaded, captures, sourceRevision);
}

function capturedReport(
  loaded: LoadedTypechecker,
  captures: Array<Capture | WorkspaceCapture>,
  sourceRevision: string,
) {
  assert.match(sourceRevision, /^[a-f0-9]{40}$/);
  const byPack = new Map<string, Capture | WorkspaceCapture>();
  for (const capture of captures) {
    assert(!byPack.has(capture.pack), "duplicate captured test body");
    assert.equal(capture.matchedContract, matchedContract);
    assert.deepEqual(capture.unbaselinedFields, unbaselinedFields);
    byPack.set(capture.pack, capture);
  }
  assert.deepEqual(new Set(byPack.keys()), new Set(Object.keys(TYPECHECKER_TESTS)));
  const rows = loaded.cases.map((fixture) => {
    const capture = byPack.get(fixture.pack);
    assert(capture);
    const observed = capture.cases.find((item) => item.id === fixture.case);
    assert(observed, "missing actual production observation");
    assert.deepEqual(observed.diagnostics, fixture.diagnostics);
    assert.deepEqual(
      publicResultProjection(observed.publicResult, observed.projectRoot),
      observed.diagnostics,
    );
    return {
      id: fixture.id,
      target: "batch",
      legacy: {
        state: "matched-reference",
        ...(fixture.diagnosticAuthority
          ? { diagnosticAuthority: fixture.diagnosticAuthority }
          : {}),
        matchedContract,
        unbaselinedFields,
        observation: observed,
        provenance:
          "workspaceReceiptSha256" in capture
            ? {
                sourceRevision,
                test: capture.test,
                kind: "full-workspace",
                workspaceReceiptSha256: capture.workspaceReceiptSha256,
                binarySha256: capture.binarySha256,
                fixturePackSha256: capture.fixturePackSha256,
              }
            : {
                sourceRevision,
                test: capture.test,
                archiveReceiptSha256: capture.archiveReceiptSha256,
                binarySha256: capture.binarySha256,
                fixturePackSha256: capture.fixturePackSha256,
              },
      },
      native: { state: "unsupported", reason: "whole-product native typechecker path unavailable" },
      comparison: { state: "not-compared" },
    };
  });
  const report = {
    schema: "vize.differential.result",
    version: 1,
    product: "typechecker",
    sourceRevision,
    manifestSha256: loaded.manifestSha256,
    rows,
    summary: {
      plannedCases: loaded.cases.length,
      legacyMatches: rows.length,
      legacyMatchContract: matchedContract,
      nativeUnsupported: rows.length,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
  };
  validateResultEnvelope(loaded, report, sourceRevision);
  return report;
}

export function readCapture(pathname: string): Capture {
  return JSON.parse(fs.readFileSync(pathname, "utf8"));
}
