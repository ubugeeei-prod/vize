import { realProjectJobNamesAtRevision } from "../../../tools/support/compat/github/real-project-matrix-plan.ts";
import { matrixSourceFixture } from "./real-project-matrix-source.ts";

/** Failed-only retry: copied jobs have new API IDs but retain their real timestamps. */
export function retriedMatrixEvidence(template: "legacy" | "named" = "legacy") {
  const source = matrixSourceFixture();
  const headSha = template === "legacy" ? source.legacySha : source.namedSha;
  const names = realProjectJobNamesAtRevision(headSha, source.repoRoot);
  const run = {
    id: 900,
    run_attempt: 2,
    head_sha: headSha,
    head_branch: "release/v0.435.1",
    path: ".github/workflows/real-project-matrix.yml",
    status: "completed",
    conclusion: "success",
    created_at: "2026-10-08T00:00:00Z",
    run_started_at: "2026-10-08T00:10:00Z",
    updated_at: "2026-10-08T00:20:00Z",
  };
  const previousJobs = Array.from({ length: 22 }, (_, shard) => ({
    id: 1_000 + shard,
    name: names[shard],
    run_id: run.id,
    run_attempt: 1,
    head_sha: run.head_sha,
    status: "completed",
    conclusion: shard === 19 ? "failure" : "success",
    started_at: "2026-10-08T00:00:01Z",
    completed_at: shard === 19 ? "2026-10-08T00:00:20Z" : "2026-10-08T00:00:10Z",
  }));
  const currentJobs = previousJobs.map((job, shard) => ({
    ...job,
    id: 2_000 + shard,
    run_attempt: 2,
    conclusion: "success",
    started_at: shard === 19 ? "2026-10-08T00:10:01Z" : job.started_at,
    completed_at: shard === 19 ? "2026-10-08T00:10:20Z" : job.completed_at,
  }));
  const artifacts = previousJobs.map((_job, shard) => ({
    id: shard === 19 ? 9_999 : 3_000 + shard,
    name: `real-project-matrix-${shard}`,
    expired: false,
    created_at: shard === 19 ? "2026-10-08T00:00:15Z" : "2026-10-08T00:00:05Z",
    workflow_run: { id: run.id, head_sha: run.head_sha, head_branch: run.head_branch },
  }));
  artifacts.push({
    ...artifacts[19],
    id: 3_019,
    created_at: "2026-10-08T00:10:10Z",
  });
  return { run, previousJobs, currentJobs, artifacts, repoRoot: source.repoRoot };
}
