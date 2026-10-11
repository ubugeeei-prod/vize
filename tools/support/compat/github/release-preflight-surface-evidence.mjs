import { realProjectSurfaceNames } from "../fixtures/real-project-surface-verdict.mjs";

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
    throw new Error(`${artifactName} surface verdict is not exact enforced release evidence`);
  }
  if (
    !Array.isArray(verdict.surfaces) ||
    verdict.surfaces.length !== realProjectSurfaceNames.length ||
    verdict.surfaces.some(
      (surface, index) =>
        surface?.name !== realProjectSurfaceNames[index] ||
        surface.budgetMode !== "enforce" ||
        surface.rawOutcome !== "success" ||
        surface.outcome !== "success",
    )
  ) {
    throw new Error(`${artifactName} surface verdict has no complete enforced surface set`);
  }
}

export function assertReleaseLintBudget({ artifactName, summary }) {
  const budget = summary.budget;
  if (
    budget?.status !== "success" ||
    budget.passed !== true ||
    budget.projectCount !== summary.projectCount ||
    budget.passedCount !== summary.projectCount ||
    ["failedCount", "unusableCount", "breachedCount"].some((field) => budget[field] !== 0) ||
    !Array.isArray(budget.failedProjects) ||
    budget.failedProjects.length !== 0
  ) {
    throw new Error(`${artifactName} lint divergence budget is not a complete enforced success`);
  }
}
