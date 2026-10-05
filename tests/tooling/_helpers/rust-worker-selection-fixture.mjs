import { selectRustWorkers } from "../../../tools/support/compat/github/rust-worker-selection.mjs";

const names = [
  "Verify Rust archive identity",
  "Run Rust test shard without rebuilding",
  "Verify actual typechecker fixture observations",
  "Upload Rust shard results",
];
export const time = (attempt, second) =>
  `2026-10-04T0${attempt}:00:${String(second).padStart(2, "0")}Z`;
export const context = { runId: 123, attempt: 2, sha: "a".repeat(40), repository: "owner/repo" };

export function job(shard, attempt) {
  return {
    id: attempt * 100 + shard,
    run_id: context.runId,
    run_attempt: attempt,
    head_sha: context.sha,
    name: `PR source checks / PR Rust Clippy, tests, and fixtures / Rust tests (${shard}/4)`,
    status: "completed",
    conclusion: "success",
    started_at: time(attempt, 0),
    completed_at: time(attempt, 10),
    runner_id: attempt * 10 + shard,
    runner_name: "runner",
    runner_group_id: 1,
    runner_group_name: "workers",
    labels: ["ubuntu"],
    steps: names.map((name, index) => ({
      name,
      number: index + 1,
      status: "completed",
      conclusion: "success",
      started_at: time(attempt, index * 2 + 1),
      completed_at: time(attempt, index * 2 + 2),
    })),
  };
}

export function artifact(shard, attempt) {
  return {
    id: attempt * 1000 + shard,
    name: `rust-test-shard-${shard}-${context.runId}-${attempt}`,
    size_in_bytes: 1234,
    expired: false,
    digest: `sha256:${String(shard).repeat(64)}`,
    created_at: time(attempt, 7),
    workflow_run: {
      id: context.runId,
      head_sha: context.sha,
      repository_id: 42,
      head_repository_id: 42,
    },
  };
}

export function fixture() {
  const original = [1, 2, 3, 4].map((shard) => job(shard, 1));
  return {
    run: {
      id: context.runId,
      run_attempt: 2,
      head_sha: context.sha,
      event: "merge_group",
      path: ".github/workflows/check.yml",
      repository: { id: 42, full_name: context.repository },
    },
    jobs: [
      ...original,
      job(1, 2),
      ...original.slice(1).map((previous) => ({
        ...structuredClone(previous),
        id: previous.id + 100,
        run_attempt: 2,
      })),
    ],
    artifacts: [artifact(1, 2), ...[1, 2, 3, 4].map((shard) => artifact(shard, 1))],
  };
}
export const select = ({ run, jobs, artifacts }) =>
  selectRustWorkers(context, run, jobs, artifacts);
