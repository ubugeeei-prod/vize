import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { sha256 } from "../differential/manifest.mjs";
import {
  loadFormatterHistoryAudit,
  validateFormatterHistoryAudit,
  validateFormatterHistoryExecution,
} from "../differential/formatter-history-audit.ts";
import {
  retainedFormatterFunction,
  validateCurrentFormatterWitness,
} from "../differential/formatter-history-current-witness.ts";
import { PRESERVED_FORMATTER_SOURCE } from "../differential/formatter-history-source-artifact.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("full formatter denominator rejects missing original fixes, source arms and false native credit", (t) => {
  const { audit, cases } = loadFormatterHistoryAudit(root);
  assert.equal(cases.size, 300);
  const preserved = audit.sourceCatalog.S007;
  const preservedFunctions = Object.values(audit.witnessCatalog)
    .filter((witness) => witness.sourceRef === "S007")
    .map((witness) => witness.function);
  assert.equal(new Set(preservedFunctions).size, 8);
  for (const name of preservedFunctions) {
    assert.doesNotThrow(() => validateCurrentFormatterWitness(root, preserved, name));
  }
  for (const entry of [
    { ...preserved, sha256: "0".repeat(64) },
    { ...preserved, revisions: [] },
    { ...preserved, revisions: [...preserved.revisions, "0".repeat(40)] },
  ])
    assert.throws(() => validateCurrentFormatterWitness(root, entry, preservedFunctions[0]));
  assert.throws(() =>
    validateCurrentFormatterWitness(root, preserved, "unregistered_preserved_function"),
  );
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-retained-law-"));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  for (const relative of [
    PRESERVED_FORMATTER_SOURCE.originalAsset,
    ...preservedFunctions.map(
      (name) => `crates/vize_glyph/tests/snapshots/preserve_authored_content__${name}.snap`,
    ),
  ]) {
    const target = path.join(scratch, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(root, relative), target);
  }
  const retained = [
    ["S084", ["classifies_only_comment_free_empty_statements"]],
    [
      "S210",
      [
        "test_format_tsx_component_script",
        "test_format_jsx_component_script",
        "test_format_js_expression_simple",
        "test_format_js_expression_with_optional_chaining",
        "test_format_js_expression_empty",
        "test_format_simple_script",
        "test_format_with_imports",
        "test_format_object",
        "test_format_empty_source",
        "test_format_whitespace_only",
      ],
    ],
    ["S007", preservedFunctions],
  ];
  assert.equal(
    retained.filter(([ref]) => ref !== "S007").flatMap(([, functions]) => functions).length,
    11,
  );
  assert.equal(retained.flatMap(([, functions]) => functions).length, 19);
  assert(!fs.existsSync(path.join(scratch, ".git")));
  for (const [ref, functions] of retained) {
    const entry = audit.sourceCatalog[ref];
    const target = path.join(scratch, entry.path);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(root, entry.path), target);
    for (const name of functions) {
      assert.doesNotThrow(() => validateCurrentFormatterWitness(scratch, entry, name));
    }
  }
  const owner = audit.sourceCatalog.S210;
  const current = fs.readFileSync(path.join(root, owner.path));
  const file = path.join(scratch, owner.path);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, current);
  assert.throws(
    () =>
      validateCurrentFormatterWitness(
        scratch,
        { ...owner, sha256: sha256(current) },
        "test_format_simple_script",
      ),
    /original Rust law pin changed/,
  );
  assert.throws(
    () =>
      validateCurrentFormatterWitness(
        scratch,
        { ...owner, revisions: [] },
        "test_format_simple_script",
      ),
    /original Rust law revision changed/,
  );
  assert.throws(
    () => validateCurrentFormatterWitness(scratch, owner, "unregistered_retained_function"),
    /unregistered retained Rust law transition/,
  );
  const unknown = { ...owner, path: "crates/vize_glyph/src/unregistered-owner.rs" };
  fs.writeFileSync(path.join(scratch, unknown.path), current);
  assert.throws(
    () => validateCurrentFormatterWitness(scratch, unknown, "test_format_simple_script"),
    /current witness source changed/,
  );
  fs.writeFileSync(file, Buffer.concat([Buffer.from("// changed main owner\n"), current]));
  assert.throws(
    () => validateCurrentFormatterWitness(scratch, owner, "test_format_simple_script"),
    /current witness source changed/,
  );
  const body = retainedFormatterFunction(current, "test_format_simple_script").toString();
  const driftedBody = `${body.slice(0, -1)}panic!("changed retained law");}`;
  const drifted = Buffer.from(current.toString().replace(body, driftedBody));
  assert(!current.equals(drifted));
  assert.notEqual(retainedFormatterFunction(drifted, "test_format_simple_script").toString(), body);
  fs.writeFileSync(file, drifted);
  assert.throws(
    () => validateCurrentFormatterWitness(scratch, owner, "test_format_simple_script"),
    /current witness source changed/,
  );
  for (const mutate of [
    (copy) => {
      copy.fixes.pop();
    },
    (copy) => {
      copy.fixes[1].commit = copy.fixes[0].commit;
    },
    (copy) => {
      copy.otherCommits.pop();
    },
    (copy) => {
      copy.manifestPins[0].sha256 = "0".repeat(64);
    },
    (copy) => {
      copy.fixes[0].requirements[0].caseIds = [];
    },
    (copy) => {
      const witness = copy.witnessCatalog[copy.fixes[0].currentWitnessRefs[0]];
      copy.sourceCatalog[witness.sourceRef].sha256 = "0".repeat(64);
    },
    (copy) => {
      copy.validation.nativeHandled = 1;
    },
    (copy) => {
      copy.supplementalFixes[0].unresolvedRequirements.push({ id: "missing" });
    },
    (copy) => {
      copy.fixes[0].requirements.pop();
    },
    (copy) => {
      copy.otherCommits[0].relation.controlRefs = ["absent-control"];
    },
    (copy) => {
      Object.values(copy.rustLaws)[0].publicOutputCredit = true;
    },
    (copy) => {
      copy.evidence.pendingExecutions.versionedPublicReferences = 0;
    },
    (copy) => {
      copy.evidence.localReplaySummary.plannedCases = 300;
    },
    (copy) => {
      copy.evidence.localReplays[0].reportSha256s[1] = "0".repeat(64);
    },
    (copy) => {
      copy.sourceCatalog.S084.sha256 = "0".repeat(64);
    },
    (copy) => {
      copy.witnessCatalog.W144.function = "unregistered_retained_function";
    },
  ]) {
    const copy = structuredClone(audit);
    mutate(copy);
    assert.throws(() => validateFormatterHistoryAudit(copy, root));
  }
  // Registration alone cannot stand in for the real source-built API reports.
  assert.throws(() => validateFormatterHistoryExecution(root, []));
});

void test("optimized suppression owner preserves the original placement law and rejects owner drift", (t) => {
  const { audit } = loadFormatterHistoryAudit(root);
  const entry = audit.sourceCatalog.S095;
  const name = "ranges_track_pragma_placement";
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-suppression-law-"));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  const file = path.join(scratch, entry.path);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  const current = fs.readFileSync(path.join(root, entry.path));
  fs.writeFileSync(file, current);
  assert.doesNotThrow(() => validateCurrentFormatterWitness(scratch, entry, name));
  assert.throws(
    () => validateCurrentFormatterWitness(scratch, { ...entry, sha256: "0".repeat(64) }, name),
    /original Rust law pin changed/,
  );
  assert.throws(
    () => validateCurrentFormatterWitness(scratch, { ...entry, revisions: [] }, name),
    /original Rust law revision changed/,
  );
  assert.throws(
    () => validateCurrentFormatterWitness(scratch, entry, "unregistered_retained_function"),
    sha256(current) === entry.sha256
      ? /retained Rust law is missing/
      : /unregistered retained Rust law transition/,
  );
  fs.writeFileSync(file, Buffer.concat([current, Buffer.from("\n// drifted owner\n")]));
  assert.throws(
    () => validateCurrentFormatterWitness(scratch, entry, name),
    /current witness source changed/,
  );
});
