export const requiredReleaseWorkflows = [
  "Check",
  "Benchmark",
  "Native Smoke",
  "Fuzz",
  "Miri",
  "App E2E",
  "Real Project Matrix",
  "Docs build",
];

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
  [
    "App E2E",
    [
      "app-e2e (dev)",
      "app-e2e (vrt)",
      "app-e2e (preview)",
      "app-e2e (build)",
      "app-e2e (check)",
      "app-e2e (lint)",
    ],
  ],
  [
    "Native Smoke",
    [
      ...nativeSmokeTargets.map((target) => `Native host smoke (${target})`),
      ...nativeSmokeTargets.flatMap((target) =>
        ["22", "24"].map((nodeVersion) => `Fresh install smoke (${target}, Node ${nodeVersion})`),
      ),
    ],
  ],
  ["Real Project Matrix", Array.from({ length: 11 }, (_, shard) => `real projects (${shard}/11)`)],
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

export function latestRequiredWorkflowRun(runs, sha, workflowName, qualifier = () => true) {
  const evidence = requiredReleaseWorkflowEvidence.get(workflowName);
  if (evidence == null) {
    throw new Error(`Release evidence is not configured for ${workflowName}`);
  }
  return runs
    .filter((run) => run.head_sha === sha && matchesEvidence(run, evidence) && qualifier(run))
    .sort(compareRuns)[0];
}

export function selectRequiredWorkflowRuns(
  runs,
  sha,
  required = requiredReleaseWorkflows,
  qualifiers = new Map(),
) {
  const selected = new Map();
  const failures = [];
  for (const workflowName of required) {
    const evidence = requiredReleaseWorkflowEvidence.get(workflowName);
    if (evidence == null) {
      throw new Error(`Release evidence is not configured for ${workflowName}`);
    }
    const current = latestRequiredWorkflowRun(
      runs,
      sha,
      workflowName,
      qualifiers.get(workflowName),
    );
    if (current == null) {
      failures.push(
        `${workflowName}: missing ${evidence.events.join("/")} run from ${evidence.path} for ${sha}`,
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
      `Required release gates are not green on ${sha}:\n${failures
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
