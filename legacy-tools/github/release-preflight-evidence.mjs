import { requiredRealProjectMatrixShardCount } from "./release-preflight-matrix-evidence.mjs";

/**
 * TEMPORARY — three gates were removed here. Tracked in #4461.
 *
 * A release used to wait hours. Measured wall-clock, recent runs:
 *
 *   Real Project Matrix   123, 125, 126, 303, 317 minutes
 *   App E2E               21
 *   Native Smoke          12-40
 *   Check                 8-11
 *   Benchmark             4-5
 *   Fuzz (replay)         3
 *
 * The release workflow's own build matrix finishes in ~17 minutes, bounded by
 * the Windows native target, so anything slower than that is pure added
 * latency on the critical path.
 *
 * Removed, and why each one is affordable to remove right now:
 *
 * - `Real Project Matrix` — hours, and it currently proves nothing. Every
 *   surface the release dispatches is `record-only` (#4462) and the artifact
 *   parity proof is advisory (#4463), so it cannot fail a release on a finding.
 *   It can only add latency and flakiness: 4 of its 22 shards were cancelled by
 *   fail-fast in run 32217699071, which alone would have failed the artifact
 *   completeness check. Restore it with its verdicts, not before.
 * - `Native Smoke` — the release workflow already runs `Smoke release npm
 *   package installs` against the artifacts this tag built, in-run, at the tag.
 *   The separate gate re-proves that on a slower path.
 * - `App E2E` — 21 minutes, and it runs on its own schedule.
 *
 * What still gates a publish: `Check` (the full suite), `Miri`, `Docs build` —
 * all push-triggered, so they are already green on the release commit's parent
 * and cost nothing to confirm — plus `Benchmark` and `Fuzz` replay, which are
 * dispatched but finish inside the build matrix's own 17 minutes.
 */
export const requiredReleaseWorkflows = ["Check", "Benchmark", "Fuzz", "Miri", "Docs build"];

export const requiredReleaseWorkflowEvidence = new Map([
  [
    "Check",
    { path: ".github/workflows/check.yml", events: ["push"], branches: { push: ["main"] } },
  ],
  ["Benchmark", { path: ".github/workflows/benchmark.yml", events: ["workflow_dispatch"] }],
  [
    "Native Smoke",
    {
      path: ".github/workflows/native-smoke.yml",
      events: ["schedule", "workflow_dispatch"],
      branches: { schedule: ["main"] },
    },
  ],
  [
    "Fuzz",
    {
      path: ".github/workflows/fuzz.yml",
      events: ["schedule", "workflow_dispatch"],
      branches: { schedule: ["main"] },
    },
  ],
  ["Miri", { path: ".github/workflows/miri.yml", events: ["push"], branches: { push: ["main"] } }],
  [
    "App E2E",
    {
      path: ".github/workflows/e2e.yml",
      events: ["schedule", "workflow_dispatch"],
      branches: { schedule: ["main"] },
    },
  ],
  [
    "Real Project Matrix",
    {
      path: ".github/workflows/real-project-matrix.yml",
      events: ["schedule", "workflow_dispatch"],
      branches: { schedule: ["main"] },
    },
  ],
  [
    "Docs build",
    {
      path: ".github/workflows/build-docs.yml",
      events: ["push", "workflow_dispatch"],
      branches: { push: ["main"] },
    },
  ],
]);

const nativeSmokeTargets = [
  "linux-x64-gnu",
  "linux-arm64-gnu",
  "darwin-x64",
  "darwin-arm64",
  "win32-x64-msvc",
  "win32-arm64-msvc",
];

const requiredJobNames = new Map([
  ["Check", ["test-scripts"]],
  ["Benchmark", ["pr-benchmark-budget"]],
  [
    "Fuzz",
    [
      "Fuzz sfc_parse",
      "Fuzz template_lexer",
      "Fuzz js_ts_expression",
      "Fuzz css_parse",
      "Fuzz template_compile",
    ],
  ],
  ["App E2E", ["app-e2e"]],
  [
    "Native Smoke",
    [
      ...nativeSmokeTargets.map((target) => `Native host smoke (${target})`),
      ...nativeSmokeTargets.flatMap((target) =>
        ["22", "24"].map((nodeVersion) => `Fresh install smoke (${target}, Node ${nodeVersion})`),
      ),
    ],
  ],
  [
    "Real Project Matrix",
    Array.from(
      { length: requiredRealProjectMatrixShardCount },
      (_, shard) => `real projects (${shard}/${requiredRealProjectMatrixShardCount})`,
    ),
  ],
]);

function compareRuns(left, right) {
  const order = (run) => [
    Date.parse(run.run_started_at ?? run.created_at ?? run.updated_at ?? "") || 0,
    Number(run.run_attempt ?? 0),
    Number(run.id ?? 0),
  ];
  const leftOrder = order(left);
  const rightOrder = order(right);
  for (let index = 0; index < leftOrder.length; index += 1) {
    if (leftOrder[index] !== rightOrder[index]) return rightOrder[index] - leftOrder[index];
  }
  return 0;
}

function matchesEvidence(run, evidence) {
  if (run.path !== evidence.path || !evidence.events.includes(run.event)) return false;
  const branches = evidence.branches?.[run.event];
  return branches == null || branches.includes(run.head_branch);
}

/** `sha` accepts a list so a version-only release can reuse its parent's evidence. */
export function latestRequiredWorkflowRun(runs, sha, workflowName, qualifier = () => true) {
  const evidence = requiredReleaseWorkflowEvidence.get(workflowName);
  if (evidence == null) {
    throw new Error(`Release evidence is not configured for ${workflowName}`);
  }
  const accepted = Array.isArray(sha) ? sha : [sha];
  return runs
    .filter(
      (run) => accepted.includes(run.head_sha) && matchesEvidence(run, evidence) && qualifier(run),
    )
    .sort(compareRuns)[0];
}

/** The SHAs a given gate will accept evidence from, tag SHA first. */
export function acceptedEvidenceShas(evidenceShas, workflowName, sha) {
  const accepted = evidenceShas?.get?.(workflowName);
  if (Array.isArray(accepted) && accepted.length > 0) return accepted;
  return Array.isArray(sha) ? sha : [sha];
}

export function selectRequiredWorkflowRuns(
  runs,
  sha,
  required = requiredReleaseWorkflows,
  qualifiers = new Map(),
  evidenceShas = new Map(),
) {
  const selected = new Map();
  const failures = [];
  for (const workflowName of required) {
    const evidence = requiredReleaseWorkflowEvidence.get(workflowName);
    if (evidence == null) {
      throw new Error(`Release evidence is not configured for ${workflowName}`);
    }
    const accepted = acceptedEvidenceShas(evidenceShas, workflowName, sha);
    const current = latestRequiredWorkflowRun(
      runs,
      accepted,
      workflowName,
      qualifiers.get(workflowName),
    );
    if (current == null) {
      failures.push(
        `${workflowName}: missing ${evidence.events.join("/")} run from ${evidence.path} for ${accepted.join(" or ")}`,
      );
    } else if (current.status !== "completed" || current.conclusion !== "success") {
      failures.push(
        `${workflowName}: ${current.status}/${current.conclusion ?? "no conclusion"} (${current.html_url ?? "no URL"})`,
      );
    } else {
      selected.set(workflowName, current);
    }
  }
  if (failures.length > 0) {
    throw new Error(
      `Required release gates are not green on ${Array.isArray(sha) ? sha[0] : sha}:\n${failures
        .map((failure) => `- ${failure}`)
        .join("\n")}`,
    );
  }
  return selected;
}

export function workflowRequiresJobEvidence(workflowName) {
  return requiredJobNames.has(workflowName);
}

export function assertRequiredWorkflowJobs(workflowName, jobs) {
  for (const jobName of requiredJobNames.get(workflowName) ?? []) {
    const matching = jobs.filter((job) => job.name === jobName);
    if (matching.length !== 1) {
      throw new Error(
        `${workflowName} must contain exactly one successful ${jobName} job; found ${matching.length}`,
      );
    }
    const job = matching[0];
    if (job.status !== "completed" || job.conclusion !== "success") {
      throw new Error(
        `${workflowName} required job ${job.name} is ${job.status}/${job.conclusion ?? "no conclusion"}`,
      );
    }
  }
}
