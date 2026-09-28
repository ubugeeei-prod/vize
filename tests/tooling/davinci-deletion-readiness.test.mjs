import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { test } from "node:test";
import {
  assessDeletionReadiness,
  deletionDialects,
  deletionProducts,
} from "../differential/deletion-readiness.mjs";

const candidateSha = "c".repeat(40);
const policySha = "a".repeat(64);
const registrySha = "d".repeat(64);
const receiptSha = "b".repeat(64);
const projects = Array.from({ length: 146 }, (_, index) => `project-${index}`);
const adapters = Object.fromEntries(
  deletionProducts.map((product) => [
    product,
    {
      requiredStages: ["product"],
      verifyBuildReceipt: (report, sourceRevision) => report.sourceRevision === sourceRevision,
      verifyObservation: (observation) => observation?.executed === true,
      verifyComparison: (row) =>
        row.comparison?.state === "equal" && row.comparison?.checked === true,
    },
  ]),
);

function entry(product, tier, sourceRevision) {
  const target = product === "formatter" ? "fmt" : "primary";
  const cases = (tier === "T1" ? deletionDialects : projects).map((name, index) => ({
    id: `${product}/${tier.toLowerCase()}/${name.replaceAll(".", "-")}`,
    state: "active",
    targets: [target],
    ...(tier === "T2" ? { projectId: name } : {}),
    dialectCoverage: {
      state: "verified",
      evidence: [{ dialects: [deletionDialects[index % deletionDialects.length]] }],
    },
  }));
  const rows = cases.map((fixture) => ({
    id: fixture.id,
    target,
    legacy: { state: "completed" },
    native: {
      state: "completed",
      observation: { executed: true },
      provenance: {
        scope: "whole-product",
        sourceRevision,
        buildReceiptSha256: receiptSha,
        contributions: [
          {
            stage: "product",
            implementation: "native",
            factOrigin: "native",
            fallback: false,
          },
        ],
      },
    },
    comparison: { state: "equal", checked: true },
  }));
  return {
    loaded: { manifest: { product }, cases, manifestSha256: policySha },
    report: {
      schema: "vize.differential.result",
      version: 1,
      product,
      manifestSha256: policySha,
      sourceRevision,
      buildReceiptSha256: receiptSha,
      rows,
      summary: { plannedCases: cases.length },
    },
  };
}

function checkpoint(day, sourceRevision, tiers) {
  return {
    day,
    sourceRevision,
    policySha256: policySha,
    registrySha256: registrySha,
    runs: Object.fromEntries(tiers.map((tier, index) => [tier, index + 1])),
    tiers: Object.fromEntries(
      tiers.map((tier) => [
        tier,
        Object.fromEntries(
          deletionProducts.map((product) => [product, entry(product, tier, sourceRevision)]),
        ),
      ]),
    ),
  };
}

function completeEvidence() {
  return {
    policy: {
      minimumStableDays: 7,
      maximumGapDays: 7,
      dialects: [...deletionDialects],
      targets: Object.fromEntries(
        deletionProducts.map((product) => [product, [product === "formatter" ? "fmt" : "primary"]]),
      ),
    },
    projectIds: projects,
    candidate: checkpoint("2026-09-28", candidateSha, ["T1", "T2"]),
    history: [
      checkpoint("2026-09-14", "e".repeat(40), ["T2"]),
      checkpoint("2026-09-21", "f".repeat(40), ["T2"]),
    ],
    headSha: candidateSha,
    today: "2026-09-28",
    policySha256: policySha,
    registrySha256: registrySha,
    adapters,
    verifyRun: (runId) => Number.isSafeInteger(runId) && runId > 0,
  };
}

const codes = (evidence) =>
  new Set(assessDeletionReadiness(evidence).blockers.map((item) => item.code));

void test("complete exact-candidate and stable scheduled evidence can satisfy the deletion gate", () => {
  const result = assessDeletionReadiness(completeEvidence());
  assert.equal(result.ready, true, JSON.stringify(result.blockers.slice(0, 5)));

  const recovered = completeEvidence();
  const older = checkpoint("2026-09-07", "9".repeat(40), ["T2"]);
  older.tiers.T2.compiler.report.rows[0].native.state = "unsupported";
  recovered.history.unshift(older);
  assert.equal(
    assessDeletionReadiness(recovered).ready,
    true,
    "failures before the stability window expire",
  );
});

void test("missing, forged and fallback evidence never gets native deletion credit", () => {
  const missing = completeEvidence();
  delete missing.candidate.tiers.T1.linter;
  assert(codes(missing).has("missing-result"));

  const omitted = completeEvidence();
  omitted.candidate.tiers.T1.compiler.report.rows.pop();
  assert(codes(omitted).has("invalid-result"));

  const fallback = completeEvidence();
  fallback.candidate.tiers.T1.compiler.report.rows[0].native.provenance.contributions[0].fallback = true;
  assert(codes(fallback).has("non-native"));

  const mismatch = completeEvidence();
  mismatch.history[0].tiers.T2.lsp.report.rows[0].comparison.checked = false;
  assert(codes(mismatch).has("non-equivalent"));

  const untrusted = completeEvidence();
  untrusted.history[0].runs.T2 = 0;
  assert(codes(untrusted).has("run-provenance"));
});

void test("scope and stability cannot pass on reduced plans or selective projects", () => {
  const scope = completeEvidence();
  scope.policy.targets.typechecker = [];
  scope.policy.dialects.pop();
  scope.history[0].tiers.T2.compiler.loaded.cases.pop();
  scope.history[0].tiers.T2.compiler.report.rows.pop();
  scope.history[0].tiers.T2.compiler.report.summary.plannedCases--;
  const scopeCodes = codes(scope);
  assert(scopeCodes.has("target-policy"));
  assert(scopeCodes.has("dialect-policy"));
  assert(scopeCodes.has("missing-project"));

  const tierDialect = completeEvidence();
  tierDialect.candidate.tiers.T1.compiler.loaded.cases[0].dialectCoverage.evidence = [
    { dialects: ["js"] },
  ];
  assert(codes(tierDialect).has("missing-dialect"), "T2 cannot cover a T1 dialect hole");

  const unstable = completeEvidence();
  unstable.policy.maximumGapDays = 6;
  unstable.candidate.sourceRevision = "0".repeat(40);
  unstable.history[0].registrySha256 = "0".repeat(64);
  const unstableCodes = codes(unstable);
  assert(unstableCodes.has("stability-history"));
  assert(unstableCodes.has("candidate-sha"));
  assert(unstableCodes.has("scope-drift"));
});

void test("current repository state is explicitly blocked without native product evidence", () => {
  assert.throws(
    () =>
      execFileSync(
        process.execPath,
        [
          "tests/differential/check-deletion-readiness.mjs",
          "/nonexistent/deletion-evidence",
          candidateSha,
        ],
        { encoding: "utf8", stdio: "pipe" },
      ),
    (error) => {
      const output = error.stderr.toString();
      assert.match(output, /stability-policy/);
      assert.match(output, /product-verifier/);
      assert.match(output, /candidate-evidence/);
      return true;
    },
  );
});

void test("unmet deletion readiness runs only on explicit Actions dispatch", () => {
  const workflow = fs.readFileSync(
    new URL("../../.github/workflows/level-deletion-readiness.yml", import.meta.url),
    "utf8",
  );
  assert.match(workflow, /^  workflow_dispatch:/m);
  assert.doesNotMatch(workflow, /^  (pull_request|push|schedule|merge_group):/m);
  assert.match(workflow, /\.head_sha == \$sha and \.conclusion == "success"/);
  assert.match(workflow, /check-deletion-readiness\.mjs/);
});
