import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  loadProductManifest,
  readPinnedArtifact,
  sha256,
  validateResultEnvelope,
} from "./harness.mjs";

export const TYPECHECKER_TESTS = {
  "event-handler-narrowing": "inline_event_assignments_preserve_exact_diagnostics",
  "template-definite-assignment": "deferred_template_reads_preserve_script_diagnostics",
  "required-props-edges": "required_props_keep_exact_unicode_diagnostics",
  "unicode-reserved-props": "unicode_reserved_props_preserve_value_diagnostics",
  "reserved-expression-shapes": "reserved_prop_shapes_preserve_literal_and_member_diagnostics",
  "component-event-tuples": "component_event_tuples_preserve_all_argument_diagnostics",
  "typed-import-meta": "typed_import_meta_preserves_exact_authored_diagnostics",
  "slot-outlet-key": "slot_outlet_keys_preserve_complete_project_diagnostics",
  "options-api-any-instance": "options_api_any_instance_preserves_complete_original_diagnostics",
  "authored-unused-symbols": "authored_unused_symbols_preserve_complete_original_diagnostics",
} as const;

const originalFixtureRevision = "9aaa1fe458a09e0d0c6604dc8835ccf7c737d943";
const sourceRevisions = {
  "event-handler-narrowing": originalFixtureRevision,
  "template-definite-assignment": originalFixtureRevision,
  "required-props-edges": originalFixtureRevision,
  "unicode-reserved-props": originalFixtureRevision,
  "reserved-expression-shapes": originalFixtureRevision,
  "component-event-tuples": originalFixtureRevision,
  "typed-import-meta": "6e0f763bd0f5e986036244c2c17c9b658b0596bd",
  "slot-outlet-key": "c6e43ca98cbffe099a6ef323006139d796d62208",
  "options-api-any-instance": "35bdad84760e6251edd52bb2e3e52701974a9d63",
  "authored-unused-symbols": "cd7156d28386e072953476fdbc354a963758dc89",
} satisfies Record<keyof typeof TYPECHECKER_TESTS, string>;

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
  checkerOptions?: { optionsApi: boolean };
  projectOptions?: { vuePackage: "absent" };
  packReference: Artifact;
  inputs: Array<Artifact & { file: string }>;
  diagnostics: Diagnostic[];
};
type Pack = {
  version: number;
  issue: number;
  sourceRevision: string;
  diagnosticContract: typeof contract;
  regressionCommit: string;
  checkerOptions?: { optionsApi: boolean };
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
  const cases = (loaded.cases as TypecheckerCase[]).map((fixture) => {
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
      fixture.pack === "options-api-any-instance" ? { optionsApi: true } : undefined,
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
    for (const input of fixture.inputs) readPinnedArtifact(root, input);
    packs.set(fixture.pack, pack);
    return { ...fixture, diagnostics: original.diagnostics };
  });
  assert.deepEqual([...packs.keys()], Object.keys(TYPECHECKER_TESTS));
  for (const [name, pack] of packs) {
    assert.deepEqual(
      cases.filter((fixture) => fixture.pack === name).map((fixture) => fixture.case),
      pack.cases.map((fixture) => fixture.id),
      "missing, duplicated or reordered historical projects",
    );
  }
  return { ...loaded, cases };
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
  assert(Object.hasOwn(TYPECHECKER_TESTS, capture.pack));
  const planned = loaded.cases.filter((fixture) => fixture.pack === capture.pack);
  assert(planned.length > 0);
  assert.equal(capture.test, TYPECHECKER_TESTS[capture.pack]);
  assert.equal(capture.fixturePackSha256, planned[0].packReference.sha256);
  assert.equal(capture.archiveReceiptSha256, sha256(receipt));
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
  assert.match(sourceRevision, /^[a-f0-9]{40}$/);
  const byPack = new Map<string, Capture>();
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
        matchedContract,
        unbaselinedFields,
        observation: observed,
        provenance: {
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
