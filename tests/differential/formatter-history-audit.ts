import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { loadFormatterApiManifest } from "./formatter-api.mjs";
import { sha256 } from "./manifest.mjs";
import { validateCurrentFormatterWitness } from "./formatter-history-current-witness.ts";
import { preservedDirectiveWidthCliManifest } from "./formatter-directive-width-cli-artifact.ts";

export const FORMATTER_HISTORY_AUDIT =
  "tests/_fixtures/differential/formatter-history/fix-history-audit.json";

// This validates registration and source bindings. It never converts an audit
// row, retained Rust law, or external local report hash into execution credit.
export function validateFormatterHistoryAudit(audit: any, repoRoot: string) {
  assert.equal(audit.schema, "vize.formatter-fix-history-audit");
  assert.equal(audit.version, 2);
  assert.equal(audit.issue, 6882);
  assert.equal(audit.originalScope.revision, "9aaa1fe458a09e0d0c6604dc8835ccf7c737d943");
  assert.equal(audit.originalScope.path, "crates/vize_glyph");
  assert.equal(audit.originalScope.commits, 87);
  assert.equal(audit.originalScope.fixes, 56);
  assert.equal(audit.fixes.length, 56);
  assert.equal(audit.otherCommits.length, 31);
  assert.deepEqual(audit.originalScope.classification, {
    semanticBehaviorFixes: 54,
    testReferenceMaintenance: 1,
    engineeringControl: 1,
  });
  assert.equal(audit.validation.nativeHandled, 0);
  assert.equal(audit.validation.nativeEquivalent, 0);
  assert.deepEqual(audit.unresolvedRequirements, []);

  // Fixed fingerprints come from the original pinned Git denominator and its
  // semantic requirements. They cannot be reduced by reclassifying a commit.
  const denominator = [...audit.fixes, ...audit.otherCommits];
  assert.equal(new Set(denominator.map((entry: any) => entry.commit)).size, 87);
  assert.equal(
    sha256(
      Buffer.from(
        JSON.stringify(
          denominator
            .map(({ commit, subject }: any) => ({ commit, subject }))
            .sort((a: any, b: any) => a.commit.localeCompare(b.commit)),
        ),
      ),
    ),
    "65979e8fc7ec2dbdbf419509aeb28a2ea31a602e5cfd5e6ab413216b43fb5966",
    "original Git denominator changed",
  );
  const requirements = audit.fixes.flatMap((fix: any) =>
    fix.requirements.map((requirement: any) => ({
      fix: fix.commit,
      id: requirement.id,
      kind: requirement.kind,
      minimumDistinctRegisteredArms: requirement.minimumDistinctRegisteredArms,
      caseIds: requirement.caseIds,
    })),
  );
  assert.equal(requirements.length, 150);
  assert.equal(
    sha256(Buffer.from(JSON.stringify(requirements))),
    "6123607e79c204e17a54f790b2687d9dca2c68b2c86dc1073616d5a4f80ac87e",
    "original semantic obligations changed",
  );

  const cases = new Map<string, any>();
  const inventories = new Set<string>();
  assert.equal(audit.manifestPins.length, 8);
  for (const entry of audit.manifestPins) {
    assert(!inventories.has(entry.path), "duplicate manifest inventory");
    inventories.add(entry.path);
    const file = path.join(repoRoot, entry.path);
    assert.equal(sha256(fs.readFileSync(file)), entry.sha256, "audit manifest pin changed");
    const loaded = loadFormatterApiManifest(file, repoRoot);
    assert.equal(loaded.cases.length, entry.cases);
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

  function source(ref: string) {
    assert(Object.hasOwn(audit.sourceCatalog, ref), "missing source authority");
    const entry = audit.sourceCatalog[ref];
    assert.equal(typeof entry.path, "string");
    assert(!path.isAbsolute(entry.path) && !entry.path.split("/").includes(".."));
    assert.match(entry.sha256, /^[a-f0-9]{64}$/);
    assert(entry.revisions.length > 0);
    for (const revision of entry.revisions) assert.match(revision, /^[a-f0-9]{40}$/);
    return entry;
  }
  function witness(ref: string, current = false) {
    assert(Object.hasOwn(audit.witnessCatalog, ref), "missing witness authority");
    const witness = audit.witnessCatalog[ref];
    assert.equal(typeof witness.function, "string");
    const entry = source(witness.sourceRef);
    if (current) {
      assert(entry.revisions.includes(audit.sourceIdentity.productRevision));
      validateCurrentFormatterWitness(repoRoot, entry, witness.function);
    }
  }
  for (const ref of Object.keys(audit.sourceCatalog)) source(ref);
  for (const ref of Object.keys(audit.witnessCatalog)) witness(ref);
  for (const law of Object.values(audit.rustLaws) as any[]) {
    assert.equal(law.publicOutputCredit, false);
    assert(law.contracts.length > 0);
    assert(law.currentWitnessRefs.length > 0);
    for (const ref of law.originalWitnessRefs) witness(ref);
    for (const ref of law.currentWitnessRefs) witness(ref, true);
  }
  for (const control of Object.values(audit.controls) as any[]) {
    assert.equal(control.publicOutputCredit, false);
    assert.match(control.commit, /^[a-f0-9]{40}$/);
    assert(control.contract.length > 0);
    assert(
      (control.sourceRefs?.length ?? 0) +
        (control.paths?.length ?? 0) +
        (control.pairs?.length ?? 0) >
        0,
    );
    for (const ref of control.sourceRefs ?? []) source(ref);
    for (const pair of control.pairs ?? []) {
      source(pair.inputSourceRef);
      source(pair.expectedSourceRef);
    }
    if (control.bijectionSourceRef) source(control.bijectionSourceRef);
  }
  function linked(entry: any) {
    for (const ref of entry.lawRefs ?? [])
      assert(Object.hasOwn(audit.rustLaws, ref), "missing Rust law");
    for (const ref of entry.controlRefs ?? [])
      assert(Object.hasOwn(audit.controls, ref), "missing engineering control");
    for (const ref of entry.supersessionRefs ?? [])
      assert(
        audit.supersessions.some((item: any) => item.commit === ref),
        "missing supersession proof",
      );
  }
  const fixes = new Set<string>();
  const cliPin = audit.cliManifestPin;
  assert.equal(cliPin.path, "tests/_fixtures/differential/formatter/manifest.json");
  const currentCli = fs.readFileSync(path.join(repoRoot, cliPin.path));
  const originalCli =
    preservedDirectiveWidthCliManifest(repoRoot, cliPin, currentCli) ?? currentCli;
  assert.equal(sha256(originalCli), cliPin.sha256);
  const cliCases = new Set(
    JSON.parse(originalCli.toString()).cases.map((fixture: any) => fixture.id),
  );
  assert.equal(cliCases.size, cliPin.cases);
  // Later correctness regressions extend the corpus without retiring any of
  // the original five source-bound CLI obligations.
  assert.deepEqual([...cliCases].slice(0, 5), [
    "formatter/sfc/split-v-pre-indentation",
    "formatter/sfc/plain-style-nested-comments",
    "formatter/sfc/vue2-filter-chain-crlf",
    "formatter/sfc/vue2-7-filter-chain",
    "formatter/sfc/vue3-bitwise-or",
  ]);
  for (const [index, fix] of [...audit.fixes, ...audit.supplementalFixes].entries()) {
    assert.match(fix.commit, /^[a-f0-9]{40}$/);
    assert.match(fix.subject, /^fix(?:\([^)]+\))?: /);
    assert(!fixes.has(fix.commit), "duplicate fix denominator entry");
    fixes.add(fix.commit);
    if (index < 56) assert.equal(fix.ordinal, index + 1);
    assert.deepEqual(fix.unresolvedRequirements, []);
    for (const id of fix.caseIds)
      assert(cases.has(id) || cliCases.has(id), "unregistered fix case");
    for (const ref of fix.originalWitnessRefs) witness(ref);
    for (const ref of fix.currentWitnessRefs) witness(ref, true);
    linked(fix);
    for (const requirement of fix.requirements) {
      const ids = new Set<string>(requirement.caseIds);
      assert.equal(ids.size, requirement.caseIds.length, "duplicate requirement arm");
      for (const id of ids) assert(cases.has(id) || cliCases.has(id), "unregistered required arm");
      for (const ref of requirement.witnessRefs ?? []) witness(ref);
      if (requirement.kind === "engineering-control") {
        assert.equal(fix.scope, "engineering-control");
        assert(Object.values(audit.controls).some((control: any) => control.commit === fix.commit));
      } else {
        const minimum = requirement.minimumDistinctRegisteredArms ?? 1;
        assert(Number.isInteger(minimum) && minimum > 0);
        assert(ids.size >= minimum, "missing distinct source arm");
        assert.equal(requirement.state, "registered");
      }
    }
  }
  assert.equal(audit.supplementalFixes.length, 3);
  for (const entry of audit.otherCommits) {
    assert(entry.classification.length > 0 && entry.rationale.length > 0);
    assert(Object.values(entry.relation).some((refs: any) => refs.length > 0));
    for (const ref of entry.relation.fixRefs)
      assert(fixes.has(ref), "missing classified fix relation");
    for (const id of entry.relation.caseIds) assert(cases.has(id), "unregistered classified case");
    linked(entry.relation);
  }
  for (const entry of audit.supersessions) {
    assert(entry.contract.length > 0 && entry.sourceRefs.length > 0);
    for (const ref of entry.sourceRefs) source(ref);
    for (const id of entry.preservedCaseIds) assert(cases.has(id), "superseded input missing");
  }
  for (const capture of audit.evidence.captures) {
    const raw = fs.readFileSync(path.join(repoRoot, capture.path));
    assert.equal(sha256(raw), capture.sha256);
    const receipt = JSON.parse(raw.toString());
    assert.equal(receipt.source.sourceRevision, capture.sourceRevision);
    assert.equal(receipt.source.observerSourceSha256, capture.observerSourceSha256);
    assert.equal(receipt.rows.length, capture.cases);
  }
  for (const entry of audit.evidence.sharedCaptureAssets) {
    assert.equal(sha256(fs.readFileSync(path.join(repoRoot, entry.path))), entry.sha256);
  }
  assert.deepEqual(audit.evidence.pendingExecutions, {
    versionedPublicReferences: 14,
    cliVerdictPlans: 5,
  });
  assert.equal(audit.evidence.localReplays.length, 7);
  assert.equal(
    new Set(audit.evidence.localReplays.map((replay: any) => replay.manifestRef)).size,
    7,
  );
  const totals: Record<string, number> = {};
  for (const replay of audit.evidence.localReplays) {
    assert(inventories.has(replay.manifestRef));
    assert.equal(
      audit.manifestPins.find((entry: any) => entry.path === replay.manifestRef).cases,
      replay.summary.plannedCases,
    );
    assert.equal(replay.summary.nativeUnsupported, replay.summary.plannedCases);
    assert.equal(replay.summary.pairedComparisons, 0);
    assert.equal(
      replay.summary.legacyByteMatches +
        replay.summary.legacyErrorMatches +
        replay.summary.legacyInternalObservations,
      replay.summary.plannedCases,
    );
    assert.match(replay.executedManifestSha256, /^[a-f0-9]{64}$/);
    assert.equal(replay.reportPaths.length, 2);
    assert.equal(replay.reportSha256s.length, 2);
    assert.match(replay.reportSha256s[0], /^[a-f0-9]{64}$/);
    assert.equal(replay.reportSha256s[0], replay.reportSha256s[1]);
    for (const key of [
      "plannedCases",
      "legacyByteMatches",
      "legacyErrorMatches",
      "legacyInternalObservations",
      "legacyFailures",
      "nativeHandled",
      "nativeEquivalent",
    ])
      totals[key] = (totals[key] ?? 0) + replay.summary[key];
  }
  assert.deepEqual(totals, audit.evidence.localReplaySummary);
  assert.deepEqual(totals, {
    plannedCases: 286,
    legacyByteMatches: 262,
    legacyErrorMatches: 21,
    legacyInternalObservations: 3,
    legacyFailures: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
  });
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
