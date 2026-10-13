import { realProjectSurfaceNames } from "../fixtures/real-project-surface-verdict.mjs";

/**
 * How release preflight treats each Real Project Matrix surface.
 *
 * - `enforced`: runs in `enforce` mode and must succeed without normalization.
 * - `recorded`: runs in `record-only` mode; the raw outcome is retained as
 *   evidence and a raw failure does not refuse the release.
 * - `skipped`: dispatched as `skip` and must report that it did not run.
 *
 * The three parity surfaces do not gate a release until per-project budgets
 * gate them again. Typecheck divergence is skipped because one fixture
 * exhausts its 100-minute step timeout.
 */
export const releaseSurfacePolicies = Object.freeze({
  "waiver-audit": "enforced",
  "typecheck-dependencies": "enforced",
  "core-tools": "enforced",
  lsp: "recorded",
  "lint-divergence": "recorded",
  "syntax-highlighter": "enforced",
  glyph: "enforced",
  "typecheck-divergence": "skipped",
});

function surfaceMatchesPolicy(surface, policy) {
  if (policy === "enforced") {
    return surface.budgetMode === "enforce" && surface.rawOutcome === "success";
  }
  if (policy === "recorded") {
    return (
      surface.budgetMode === "record-only" &&
      (surface.rawOutcome === "success" || surface.rawOutcome === "failure")
    );
  }
  return policy === "skipped" && surface.budgetMode === "skip" && surface.rawOutcome === "skipped";
}

export function assertReleaseSurfaceVerdict({ artifactName, headSha, shard, verdict }) {
  if (verdict.status !== "success") {
    throw new Error(`${artifactName} surface verdict is ${String(verdict.status)}`);
  }
  if (
    verdict.schema !== "vize.realProjectSurfaceVerdict" ||
    verdict.version !== 2 ||
    verdict.sourceCommit !== headSha ||
    verdict.shardIndex !== String(shard) ||
    !Array.isArray(verdict.failedSurfaceNames) ||
    verdict.failedSurfaceNames.length !== 0
  ) {
    throw new Error(`${artifactName} surface verdict is not exact release-gated evidence`);
  }
  if (
    !Array.isArray(verdict.surfaces) ||
    verdict.surfaces.length !== realProjectSurfaceNames.length ||
    verdict.surfaces.some(
      (surface, index) =>
        surface?.name !== realProjectSurfaceNames[index] ||
        surface.outcome !== "success" ||
        !surfaceMatchesPolicy(surface, releaseSurfacePolicies[surface.name]),
    )
  ) {
    throw new Error(`${artifactName} surface verdict has no complete release-gated surface set`);
  }
}

/**
 * Lint divergence is a recorded surface: breached and unusable projects stay
 * visible in the release log, but the retained budget must account for every
 * measured project so a truncated summary cannot pass as evidence.
 */
export function assertReleaseLintBudget({ artifactName, summary }) {
  const budget = summary.budget;
  const counts = ["projectCount", "passedCount", "failedCount", "unusableCount", "breachedCount"];
  if (
    budget == null ||
    counts.some((field) => !Number.isSafeInteger(budget[field]) || budget[field] < 0) ||
    budget.projectCount !== summary.projectCount ||
    budget.passedCount + budget.failedCount !== budget.projectCount ||
    budget.unusableCount + budget.breachedCount !== budget.failedCount ||
    !Array.isArray(budget.failedProjects) ||
    budget.failedProjects.length !== budget.failedCount ||
    budget.failedProjects.some((project) => typeof project !== "string") ||
    budget.passed !== (budget.failedCount === 0) ||
    budget.status !== (budget.passed ? "success" : "failure")
  ) {
    throw new Error(
      `${artifactName} lint divergence budget does not account for every measured project`,
    );
  }
  if (!budget.passed) {
    console.warn(
      `recorded lint parity: ${artifactName} retains ${budget.breachedCount} breached and ${budget.unusableCount} unusable lint divergence project(s): ${budget.failedProjects.join(", ")}`,
    );
  }
}
