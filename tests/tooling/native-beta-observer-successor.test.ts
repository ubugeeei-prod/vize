import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";
import {
  NATIVE_BETA_DTO_PROTOCOL_ID,
  nativeBetaDtoObserverBoundary,
  nativeBetaDtoProducerCustody,
} from "../../tools/support/compat/github/native-beta-dto-fixtures.ts";
import { currentNativeBetaDtoObserver } from "../../tools/support/compat/github/native-beta-dto-observer-successor.ts";

const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const headManifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const inputId = "typechecker/regression/native-overlay-input-custody-11145";
assert.equal(headManifest.cases.filter((row: { id: string }) => row.id === inputId).length, 1);
const baseManifest = structuredClone(headManifest);
baseManifest.cases = baseManifest.cases.filter((row: { id: string }) => row.id !== inputId);
const boundary = nativeBetaDtoObserverBoundary();
const product = "crates/vize_canon/src/lsp_client/session_storage.rs";
type Override = Map<string, Buffer | null>;
const key = (side: string, path: string) => side + ":" + path;
// Synthetic source-boundary controls use actual committed fixture/producer bytes.
// Real b51->cfa Git ancestry/equality is separately replayed, never inferred here.
function read(side: "base" | "head", path: string, override: Override = new Map()) {
  if (override.has(key(side, path))) return override.get(key(side, path))!;
  if (path === manifestPath)
    return Buffer.from(JSON.stringify(side === "base" ? baseManifest : headManifest));
  try {
    return fs.readFileSync(path);
  } catch {
    return null;
  }
}
function report(paths: string[], override: Override = new Map()) {
  return analyzeLegacyFix({
    title: "fix(canon): isolate native LSP overlay storage",
    paths,
    readFile: (side, path) => read(side, path, override),
  }).products.find((row) => row.product === "typechecker")!;
}
const protocol = (manifest: typeof headManifest) =>
  manifest.protocolCases.filter((row: { id: string }) => row.id === NATIVE_BETA_DTO_PROTOCOL_ID);

test("only the exact current observer pair is classified separately from new ABI inputs", () => {
  const value = report([...boundary.observers, product]);
  assert.equal(value.state, "new-inputs-registered");
  assert.deepEqual(value.cases, [inputId]);
  assert.deepEqual(value.currentObserverSuccessor?.sources, boundary.observers);
  assert.equal(value.currentObserverSuccessor?.nativeQualification, false);
  assert.equal(value.currentObserverSuccessor?.abiProductionUnchanged.length, 8);
  assert.equal(value.currentObserverSuccessor?.historicalInputsUnchanged.length, 4);
  assert.ok(
    !value.cases.includes(NATIVE_BETA_DTO_PROTOCOL_ID),
    "an existing protocol is not a new case",
  );
  const sameManifest = new Map([
    [key("head", manifestPath), Buffer.from(JSON.stringify(baseManifest))],
  ]);
  const observerOnly = report(boundary.observers, sameManifest);
  assert.equal(observerOnly.state, "new-inputs-registered");
  assert.deepEqual(observerOnly.cases, []);
  assert.ok(observerOnly.currentObserverSuccessor);
  assert.equal(
    report([...boundary.observers, product], sameManifest).state,
    "new-input-missing",
    "other Canon production still needs the genuine new TC input",
  );
});

test("every real or unreadable ABI production change keeps the original registration requirement", () => {
  for (const path of boundary.production) {
    for (const bytes of [
      Buffer.concat([fs.readFileSync(path), Buffer.from("\n// changed ABI source\n")]),
      null,
    ]) {
      const override = new Map([[key("head", path), bytes]]);
      const value = report([...boundary.observers, product, path], override);
      assert.equal(value.state, "new-input-missing", path);
      assert.equal(value.currentObserverSuccessor, undefined);
    }
  }
  assert.equal(
    currentNativeBetaDtoObserver(boundary.observers, undefined, protocol(headManifest), read)
      .sources.size,
    0,
    "initial ABI registration is not replaced by an observer classification",
  );
});

test("historical row, all four ABI inputs and all five original producer archives are immutable", () => {
  const changed = structuredClone(headManifest);
  const row = changed.protocolCases.find(
    (row: { id: string }) => row.id === NATIVE_BETA_DTO_PROTOCOL_ID,
  );
  row.tests[3].sha256 = "f9fe37e13f4a566cc01daadfc48ebb96b1842d6f3215a4020312af41bcbb97e1";
  assert.equal(
    report(
      boundary.observers,
      new Map([[key("head", manifestPath), Buffer.from(JSON.stringify(changed))]]),
    ).state,
    "invalid-manifest-or-input",
    "a current observer must not overwrite historical 26eb identity",
  );
  assert.throws(
    () =>
      currentNativeBetaDtoObserver(boundary.observers, protocol(changed), protocol(changed), read),
    "mixed historical/current row identities still reject when both sides are forged alike",
  );
  const custody = nativeBetaDtoProducerCustody(read);
  for (const path of [...boundary.inputs, ...custody.map((row) => row.archivedPath)])
    for (const side of ["base", "head"] as const) {
      const override = new Map([
        [key(side, path), Buffer.concat([fs.readFileSync(path), Buffer.from("changed")])],
      ]);
      assert.equal(
        report(boundary.observers, override).state,
        "invalid-manifest-or-input",
        side + ":" + path,
      );
    }
});

test("missing or forged current source, dependency, runtime or four-role producer cannot qualify", () => {
  const paths = [
    ...boundary.observers,
    "tests/tooling/support/native-beta-dto/prepare-role.ts",
    "tests/tooling/support/native-beta-dto/audit-wire.ts",
    "tests/tooling/support/native-beta-dto/audit-wire-types.ts",
    "tests/tooling/support/native-beta-dto/beta-attached-route.ts",
  ];
  for (const path of paths)
    for (const bytes of [null, Buffer.from("unregistered current observer/dependency\n")])
      assert.equal(
        report(boundary.observers, new Map([[key("head", path), bytes]])).state,
        "invalid-manifest-or-input",
        path,
      );
  const workflow = ".github/workflows/native-beta-diagnostic-dto.yml";
  const original = fs.readFileSync(workflow, "utf8");
  for (const mutated of [
    original.replace("role: [before, after]", "role: [after]"),
    original.replace("version: 7.0.2", "version: 7.1.0-beta"),
  ]) {
    assert.notEqual(mutated, original);
    assert.equal(
      report(boundary.observers, new Map([[key("head", workflow), Buffer.from(mutated)]])).state,
      "invalid-manifest-or-input",
      "all four genuine roles remain required",
    );
  }
});

test("unknown Canon paths and other ABI tests never inherit the two-path classification", () => {
  const noInput = new Map([[key("head", manifestPath), Buffer.from(JSON.stringify(baseManifest))]]);
  for (const path of [
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta/future.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta/../native_beta.rs",
    "crates/vize_canon/src/check.rs",
  ]) {
    const value = report([...boundary.observers, path], noInput);
    assert.equal(value.state, "new-input-missing");
    assert.ok(!value.currentObserverSuccessor?.sources.includes(path));
  }
  const dtoTest =
    "crates/vize_canon/src/lsp_client/initial_diagnostics/diagnostic_response/beta_tests.rs";
  assert.equal(
    report([...boundary.observers, product, dtoTest]).state,
    "new-input-missing",
    "the existing protocol test-source gate is not broadened",
  );
  const returned = nativeBetaDtoObserverBoundary();
  returned.production.length = 0;
  returned.observers.push("unregistered");
  assert.equal(nativeBetaDtoObserverBoundary().production.length, 8);
  assert.equal(nativeBetaDtoObserverBoundary().observers.length, 2);
});
