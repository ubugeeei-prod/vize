import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { loadFormatterApiManifest } from "./formatter-api.mjs";
import { sha256 } from "./manifest.mjs";

export const FORMATTER_HISTORY_AUDIT =
  "tests/_fixtures/differential/formatter-history/fix-history-audit.json";

// This validates registration and source bindings. It never converts an audit
// row, retained Rust law, or external local report hash into execution credit.
export function validateFormatterHistoryAudit(audit: any, repoRoot: string) {
  assert.equal(audit.schema, "vize.formatter-fix-history-audit");
  assert.equal(audit.version, 1);
  assert.equal(audit.issue, 6882);
  assert.equal(audit.originalScope.revision, "9aaa1fe458a09e0d0c6604dc8835ccf7c737d943");
  assert.equal(audit.originalScope.path, "crates/vize_glyph");
  assert.equal(audit.originalScope.commits, 87);
  assert.equal(audit.originalScope.fixes, 56);
  assert.equal(audit.fixes.length, 56);
  assert.deepEqual(audit.originalScope.classification, {
    semanticBehaviorFixes: 54,
    testReferenceMaintenance: 1,
    engineeringControl: 1,
  });
  assert.equal(audit.validation.nativeHandled, 0);
  assert.equal(audit.validation.nativeEquivalent, 0);
  assert.deepEqual(audit.unresolvedRequirements, []);

  const cases = new Map<string, any>();
  const inventories = new Set<string>();
  for (const entry of audit.manifestInventory) {
    assert(!inventories.has(entry.path), "duplicate manifest inventory");
    inventories.add(entry.path);
    const file = path.join(repoRoot, entry.path);
    assert.equal(sha256(fs.readFileSync(file)), entry.sha256, "audit manifest pin changed");
    const loaded = loadFormatterApiManifest(file, repoRoot);
    assert.equal(loaded.cases.length, entry.cases);
    assert.deepEqual(loaded.manifest.source, entry.source);
    for (const fixture of loaded.cases) {
      assert(!cases.has(fixture.id), "duplicate shared history case");
      cases.set(fixture.id, {
        fixture,
        declared: loaded.manifest.cases.find((item: any) => item.id === fixture.id),
        manifest: entry.path,
      });
    }
  }
  assert.equal(cases.size, 300);
  assert.equal(audit.caseIndex.length, cases.size);
  const indexed = new Set<string>();
  for (const entry of audit.caseIndex) {
    assert(!indexed.has(entry.id), "duplicate audit case");
    indexed.add(entry.id);
    const registered = cases.get(entry.id);
    assert(registered, "audit case missing from shared execution plan");
    assert.equal(entry.manifest, registered.manifest);
    const fixture = registered.declared;
    for (const key of ["api", "profile", "input", "expected", "options", "witness", "vueVersion"]) {
      assert.deepEqual(entry[key], fixture[key], `audit case ${key} drift`);
    }
    assert.equal(entry.outcome, fixture.outcome ?? "success");
  }

  function currentWitness(witness: any) {
    assert.equal(typeof witness.function, "string");
    const source = fs.readFileSync(path.join(repoRoot, witness.path));
    assert.equal(sha256(source), witness.sourceSha256, "current witness source changed");
    assert(source.toString().includes(`fn ${witness.function}(`), "retained Rust law is missing");
  }
  const fixes = new Set<string>();
  const cliCases = new Set(
    JSON.parse(
      fs.readFileSync(
        path.join(repoRoot, "tests/_fixtures/differential/formatter/manifest.json"),
        "utf8",
      ),
    ).cases.map((fixture: any) => fixture.id),
  );
  for (const [index, fix] of [...audit.fixes, ...audit.supplementalFixes].entries()) {
    assert.match(fix.commit, /^[a-f0-9]{40}$/);
    assert.match(fix.subject, /^fix(?:\([^)]+\))?: /);
    assert(!fixes.has(fix.commit), "duplicate fix denominator entry");
    fixes.add(fix.commit);
    if (index < 56) assert.equal(fix.ordinal, index + 1);
    assert.deepEqual(fix.unresolvedRequirements, []);
    for (const id of fix.caseIds)
      assert(cases.has(id) || cliCases.has(id), "unregistered fix case");
    for (const witness of fix.currentWitnesses) currentWitness(witness);
    for (const witness of fix.requiredRustHelperLaws ?? []) currentWitness(witness);
    for (const requirement of fix.requirements) {
      const ids = new Set(requirement.caseIds);
      assert.equal(ids.size, requirement.caseIds.length, "duplicate requirement arm");
      for (const id of ids) assert(cases.has(id) || cliCases.has(id), "unregistered required arm");
      if (requirement.kind === "engineering-control") {
        assert.equal(fix.scope, "engineering-control");
        assert(audit.controls.some((control: any) => control.fix === fix.commit));
      } else {
        const minimum = requirement.minimumDistinctRegisteredArms ?? 1;
        assert(Number.isInteger(minimum) && minimum > 0);
        assert(ids.size >= minimum, "missing distinct source arm");
        assert.equal(requirement.state, "registered");
      }
    }
    for (const reference of fix.supersessionReferences ?? []) {
      assert(
        audit.supersessions.some((entry: any) => entry.commit === reference),
        "missing supersession proof",
      );
    }
  }
  assert.equal(audit.supplementalFixes.length, 3);
  for (const capture of audit.sourceBoundCaptures) {
    const raw = fs.readFileSync(path.join(repoRoot, capture.path));
    assert.equal(sha256(raw), capture.sha256);
    const receipt = JSON.parse(raw.toString());
    assert.deepEqual(receipt.source, capture.source);
    assert.equal(receipt.rows.length, capture.cases);
  }
  return cases;
}

export function loadFormatterHistoryAudit(repoRoot: string) {
  const audit = JSON.parse(fs.readFileSync(path.join(repoRoot, FORMATTER_HISTORY_AUDIT), "utf8"));
  return { audit, cases: validateFormatterHistoryAudit(audit, repoRoot) };
}

export function validateFormatterHistoryExecution(repoRoot: string, reports: any[]) {
  const { cases } = loadFormatterHistoryAudit(repoRoot);
  const rows = reports.flatMap((report) => report.rows);
  assert.equal(rows.length, cases.size);
  assert.deepEqual(new Set(rows.map((row) => row.id)), new Set(cases.keys()));
  for (const row of rows) {
    assert.equal(row.legacy.state, "matched-reference", `history witness failed: ${row.id}`);
    assert.equal(row.native.state, "unsupported");
    assert.equal(row.comparison.state, "not-compared");
  }
}
