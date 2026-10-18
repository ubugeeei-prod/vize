import { selectRustWorkers } from "../../../tools/support/compat/github/rust-worker-selection.mjs";
import {
  RUST_TEST_COHORTS,
  RUST_TEST_COHORT_IDS,
} from "../../../tools/support/compat/github/rust-test-cohorts.ts";

export const cohorts = RUST_TEST_COHORT_IDS;

const names = [
  "Verify Rust archive identity",
  "Run Rust test cohort without rebuilding",
  "Verify actual typechecker fixture observations",
  "Upload Rust cohort results",
];
export const time = (attempt, second) =>
  `2026-10-04T0${attempt}:00:${String(second).padStart(2, "0")}Z`;
export const context = { runId: 123, attempt: 2, sha: "a".repeat(40), repository: "owner/repo" };

export function job(cohort, attempt) {
  const index = RUST_TEST_COHORT_IDS.indexOf(cohort) + 1;
  const definition = RUST_TEST_COHORTS.find(({ id }) => id === cohort);
  return {
    id: attempt * 100 + index,
    run_id: context.runId,
    run_attempt: attempt,
    head_sha: context.sha,
    name: `PR source checks / PR Rust workspace products / Rust tests (${definition.name})`,
    status: "completed",
    conclusion: "success",
    started_at: time(attempt, 0),
    completed_at: time(attempt, 10),
    runner_id: attempt * 10 + index,
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

export function artifact(cohort, attempt) {
  const index = RUST_TEST_COHORT_IDS.indexOf(cohort) + 1;
  return {
    id: attempt * 1000 + index,
    name: `rust-test-cohort-${cohort}-${context.runId}-${attempt}`,
    size_in_bytes: 1234,
    expired: false,
    digest: `sha256:${String(index).repeat(64)}`,
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
  const original = RUST_TEST_COHORT_IDS.map((cohort) => job(cohort, 1));
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
      job("compiler", 2),
      ...original.slice(1).map((previous) => ({
        ...structuredClone(previous),
        id: previous.id + 100,
        run_attempt: 2,
      })),
    ],
    artifacts: [
      artifact("compiler", 2),
      ...RUST_TEST_COHORT_IDS.map((cohort) => artifact(cohort, 1)),
    ],
  };
}
export const latestWorker = (value, cohort = "compiler") =>
  value.jobs.find((worker) => worker.id === 201 + RUST_TEST_COHORT_IDS.indexOf(cohort));
export const select = ({ run, jobs, artifacts }) =>
  selectRustWorkers(context, run, jobs, artifacts);
