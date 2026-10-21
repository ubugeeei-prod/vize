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
  "split-script-original": "split_script_setup_preserves_complete_original_diagnostics",
  "v-for-source-original": "v_for_source_callbacks_preserve_complete_original_diagnostics",
  "declared-prop-names-original": "declared_prop_names_preserve_complete_original_diagnostics",
  "v-for-component-source-original":
    "component_v_for_source_preserves_complete_original_diagnostics",
  "template-suppression-original": "template_suppression_preserves_complete_original_diagnostics",
  "allow-importing-ts-extensions-9890": "authored_ts_extensions_report_ts5097_by_default",
  "nuxt-plugin-injection-types": "nuxt_reference_graph_preserves_plugin_injection_import_types",
  "listener-attribute-parity-9852": "listener_keeps_class_style_and_spread_attribute_parity",
  "dynamic-component-union-listener-10026":
    "dynamic_component_union_listener_preserves_clean_diagnostics",
  "cache-maintenance-9860": "bounded_cache_maintenance_preserves_project_diagnostics",
  "reactive-props-destructure-10008": "reactive_props_destructure_preserves_unused_diagnostics",
  "nuxt-unknown-template-options-10013": "nuxt_unknown_template_options_preserve_explicit_checks",
  "native-inline-event-handler-9835": "native_inline_event_handler_is_contextually_typed",
  "inferred-project-tsconfig-9718": "inferred_project_tsconfig_preserves_owned_batch_inputs",
  "generic-fallthrough-spread-9956": "generic_fallthrough_spread_preserves_valid_values",
  "optional-fallthrough-spread-10170":
    "optional_fallthrough_spread_preserves_valid_values_and_rejects_invalid_values",
  "art-variant-implicit-component-9854":
    "implicit_art_component_reports_the_same_prop_errors_as_an_explicit_import",
  "misskey-generated-inference-9516":
    "misskey_generated_inference_probes_preserve_authored_diagnostics",
  "component-v-bind-duplicate-diagnostic-10069":
    "named_prop_before_spread_reports_once_with_or_without_listener",
  "runtime-props-report-10100-10101": "runtime_props_reports_10100_10101_retain_expected_types",
  "native-pointer-events-original": "native_click_events_preserve_pointer_event_diagnostics",
  "workspace-alias-cache-10063": "workspace_package_alias_cache_preserves_sequential_scopes",
  "component-render-h-props-9855": "imported_sfc_h_calls_check_missing_and_wrong_props",
  "component-listener-alias-scope-10112": "component_listener_aliases_stay_inside_guard_scope",
  "content-mapper-duplicate-attributes-10184":
    "content_mapper_isolates_duplicate_class_and_style_attribute_probes",
  "dynamic-directive-expression-mapping-10182":
    "dynamic_directive_expressions_preserve_typed_slot_payloads",
  "content-mapper-template-expression-syntax":
    "template_expression_syntax_preserves_complete_authored_diagnostics",
  "content-mapper-const-enum-import": "content_mapper_accepts_imported_const_enum_value_anchors",
  "content-mapper-dynamic-template-ref-10198": "dynamic_template_refs_preserve_batch_acceptance",
  "content-mapper-dynamic-template-ref-union-10251":
    "dynamic_template_ref_unions_preserve_batch_acceptance",
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
  "split-script-original": "b3c2933be5afbdb5f2df07da093f40bc7ce72187",
  "v-for-source-original": "04aedfb8e8b41dfae89fe65256e2703fc3718cf3",
  "declared-prop-names-original": "0f15c8b49d0630c1f2492c7ad8447ef4c9711534",
  "v-for-component-source-original": "04aedfb8e8b41dfae89fe65256e2703fc3718cf3",
  "template-suppression-original": "e199979dfb32dfd8eb9ad32a2928fb010768f239",
  "allow-importing-ts-extensions-9890": "f1b07174a4a86ffc3433f31b8c78406d5bf5b760",
  "nuxt-plugin-injection-types": "2c22691f0f8303a28c425213c423e299f9ad5420",
  "listener-attribute-parity-9852": "11cb03950bc8d2a4e9cb8d164b0aaca79e302291",
  "dynamic-component-union-listener-10026": "0aafae7e41f1ac381d07fa9f0154084cc1e6f211",
  "cache-maintenance-9860": "4714c0f264dedffaa81377df9545fbfa9514a64e",
  "reactive-props-destructure-10008": "f276e4d59cdaf765fd7d3f14eb3d679f5b685f98",
  "nuxt-unknown-template-options-10013": "fb700551a3a3a69f8976efa6fe31bca43ebc19b4",
  "native-inline-event-handler-9835": "8ac6ce1668e9bede8d73411e63a247dbd6f8eb56",
  "inferred-project-tsconfig-9718": "f95557dadaab83dc0f7d4a7613bdb7ed4e59d323",
  "generic-fallthrough-spread-9956": "2b1e41a5b09005a97d24d1ff34f34c7507bbfb67",
  "optional-fallthrough-spread-10170": "8300867552287abb14f4fbb7471c705aea72bbdf",
  "art-variant-implicit-component-9854": "f07065df35d25bc76f355e5f106c80dbefa922b5",
  "misskey-generated-inference-9516": "cf6c83ab16fb7f20f84f0d51112090bbce89c74a",
  "component-v-bind-duplicate-diagnostic-10069": "7174c46d55825c155325e6314305e661d50ec95a",
  "runtime-props-report-10100-10101": "5dfe002a9467d152c8545693df33def3aa5f3bf7",
  "native-pointer-events-original": "e7d8b1c82a8badec89ec9779838dfa50c1b9b790",
  "workspace-alias-cache-10063": "7174c46d55825c155325e6314305e661d50ec95a",
  "component-render-h-props-9855": "f15ca234a4bdd73bde9caea07e652f3a345eb342",
  "component-listener-alias-scope-10112": "7cefe3a00e6184de1e121d39b0e153f08a428aa6",
  "content-mapper-duplicate-attributes-10184": "0038b601e41dccb6f037aa68c3fc0207ea635e3b",
  "dynamic-directive-expression-mapping-10182": "f305bcc8c81b8f6009dabb5a3a61bb838c755701",
  "content-mapper-template-expression-syntax": "a5ae4d313fa333c8bc7168d5da4acf3dc2bb8ee3",
  "content-mapper-const-enum-import": "506529117573a96880435dd6466cfd7cf81fdc92",
  "content-mapper-dynamic-template-ref-10198": "b7fb3a9aefd31002b8b2d449695a9e14d657654a",
  "content-mapper-dynamic-template-ref-union-10251": "6817c853ca1740fd1070c0ec462c0d457f339c2c",
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
type DialectCoverage = {
  state: "unknown" | "source-inspected";
  registryRef: null;
  evidence: Array<{
    kind: "source-inspection";
    dialects: string[];
    selector: string;
  }>;
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
  dialectCoverage: DialectCoverage;
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
const dialectOrder = ["vue3-sfc", "html-template", "js", "ts", "jsx", "tsx"];

function typecheckerDialectCoverage(
  root: string,
  inputs: TypecheckerCase["inputs"],
): DialectCoverage {
  const dialects = new Set<string>();
  for (const input of inputs) {
    const source = readPinnedArtifact(root, input).toString("utf8");
    const file = input.file.toLowerCase();
    if (file.endsWith(".vue")) {
      dialects.add("vue3-sfc");
      if (/<template(?:\s|>)/i.test(source)) dialects.add("html-template");
      for (const match of source.matchAll(/<script\b([^>]*)>/gi)) {
        const attributes = match[1];
        if (/\blang\s*=\s*["']tsx["']/i.test(attributes)) dialects.add("tsx");
        else if (/\blang\s*=\s*["']ts["']/i.test(attributes)) dialects.add("ts");
        else if (/\blang\s*=\s*["']jsx["']/i.test(attributes)) dialects.add("jsx");
        else dialects.add("js");
      }
    } else if (file.endsWith(".tsx")) dialects.add("tsx");
    else if (file.endsWith(".jsx")) dialects.add("jsx");
    else if (file.endsWith(".d.ts") || file.endsWith(".ts")) dialects.add("ts");
    else if (file.endsWith(".js")) dialects.add("js");
  }
  const ordered = dialectOrder.filter((dialect) => dialects.has(dialect));
  if (ordered.length === 0) return { state: "unknown", registryRef: null, evidence: [] };
  return {
    state: "source-inspected",
    registryRef: null,
    evidence: [
      {
        kind: "source-inspection",
        dialects: ordered,
        selector: "batch-typechecker-fixture-v1 pinned authored sources",
      },
    ],
  };
}

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
    assert.deepEqual(
      fixture.dialectCoverage,
      typecheckerDialectCoverage(root, fixture.inputs),
      "dialect coverage must match the pinned authored sources",
    );
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
