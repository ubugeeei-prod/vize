import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";

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
  ["Real Project Matrix", Array.from({ length: 11 }, (_, shard) => `real projects (${shard}/11)`)],
]);

export const requiredRealProjectMatrixShardCount = 11;

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

export async function assertRealProjectMatrixReleaseArtifacts({
  run,
  artifacts,
  readArtifactEntries,
}) {
  if (typeof readArtifactEntries !== "function") {
    throw new Error("Real Project Matrix artifact reader is required");
  }
  const expectedNames = Array.from(
    { length: requiredRealProjectMatrixShardCount },
    (_, shard) => `real-project-matrix-${shard}`,
  );
  for (const artifactName of expectedNames) {
    const matches = artifacts.filter((artifact) => artifact.name === artifactName);
    if (matches.length !== 1) {
      throw new Error(
        `Real Project Matrix release evidence must contain exactly one ${artifactName} artifact; found ${matches.length}`,
      );
    }
    const artifact = matches[0];
    assertArtifactBoundToRun(run, artifact);
    const entries = await readArtifactEntries(artifact);
    assertRealProjectShardArtifact({
      run,
      artifactName,
      shard: Number(artifactName.slice("real-project-matrix-".length)),
      entries,
    });
  }
}

export async function downloadArtifactEntries({ artifact, token, fetchImpl = globalThis.fetch }) {
  const url = artifact.archive_download_url;
  if (typeof url !== "string" || url.length === 0) {
    throw new Error(`Real Project Matrix artifact ${String(artifact.name)} has no download URL`);
  }
  const response = await fetchImpl(url, {
    headers: {
      accept: "application/vnd.github+json",
      authorization: `Bearer ${token}`,
      "x-github-api-version": "2022-11-28",
    },
  });
  if (!response.ok) {
    throw new Error(
      `Failed to download Real Project Matrix artifact ${String(artifact.name)}: ${response.status} ${response.statusText}`,
    );
  }
  const scratch = mkdtempSync(join(tmpdir(), "vize-release-matrix-artifact-"));
  try {
    const archive = join(scratch, "artifact.zip");
    const output = join(scratch, "out");
    writeFileSync(archive, Buffer.from(await response.arrayBuffer()));
    const unzip = spawnSync("unzip", ["-q", archive, "-d", output], {
      encoding: "utf8",
      timeout: 30_000,
    });
    if (unzip.error != null || unzip.status !== 0) {
      const detail = [unzip.stdout, unzip.stderr].filter(Boolean).join("\n").trim();
      throw new Error(
        `Failed to unpack Real Project Matrix artifact ${String(artifact.name)}${detail === "" ? "" : `:\n${detail}`}`,
      );
    }
    return collectTextEntries(output);
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

function assertRealProjectShardArtifact({ run, artifactName, shard, entries }) {
  const summary = readJsonEntry(entries, "summary.json", artifactName);
  if (
    summary.schema !== "vize.fixtureToolMatrixReport" ||
    summary.version !== 3 ||
    summary.evidence?.commitSha !== run.head_sha ||
    summary.command?.shardIndex !== shard ||
    summary.command?.shardCount !== requiredRealProjectMatrixShardCount
  ) {
    throw new Error(`${artifactName} summary is not exact release evidence for ${run.head_sha}`);
  }
  const selectedFixtures = readTextEntry(entries, "selected-fixtures.txt", artifactName)
    .split(/\r?\n/)
    .filter(Boolean);
  if (selectedFixtures.length === 0) {
    throw new Error(`${artifactName} selected no authored fixture corpus`);
  }

  const surface = readJsonEntry(entries, "surface-verdict.json", artifactName);
  if (surface.status !== "success") {
    throw new Error(`${artifactName} surface verdict is ${String(surface.status)}`);
  }

  const [divergenceEntry] = exactMatchingEntries(
    entries,
    /(^|\/)[^/]+-typecheck-divergence\.json$/,
    `${artifactName} typecheck divergence artifact`,
  );
  const [dependencyEntry] = exactMatchingEntries(
    entries,
    /(^|\/)[^/]+-typecheck-dependencies\.json$/,
    `${artifactName} typecheck dependency artifact`,
  );
  const divergence = parseJsonText(divergenceEntry.text, divergenceEntry.name);
  const dependency = parseJsonText(dependencyEntry.text, dependencyEntry.name);
  assertReleaseTypecheckDivergenceArtifact({
    artifactName,
    run,
    divergence,
    dependency,
    dependencySha256: sha256(dependencyEntry.text),
  });
}

function assertReleaseTypecheckDivergenceArtifact({
  artifactName,
  run,
  divergence,
  dependency,
  dependencySha256,
}) {
  if (
    divergence.schema !== "vize.fixtureTypecheckDivergenceRun" ||
    divergence.version !== 4 ||
    divergence.evidence?.commitSha !== run.head_sha
  ) {
    throw new Error(
      `${artifactName} typecheck divergence artifact is not bound to ${run.head_sha}`,
    );
  }
  if (divergence.enforcement?.budgetMode !== "enforce") {
    throw new Error(
      `${artifactName} typecheck divergence artifact used ${String(divergence.enforcement?.budgetMode)} mode; release evidence must not be record-only`,
    );
  }
  if (divergence.budget?.passed !== true || divergence.budget?.verdict !== "passed") {
    throw new Error(
      `${artifactName} typecheck divergence budget is ${String(divergence.budget?.verdict)}`,
    );
  }

  const summary = divergence.divergence?.summary;
  if (summary?.falsePositiveCount !== 0 || summary?.falseNegativeCount !== 0) {
    throw new Error(
      `${artifactName} typecheck divergence must have zero unexplained false positives and false negatives; got ${String(summary?.falsePositiveCount)} FP and ${String(summary?.falseNegativeCount)} FN`,
    );
  }
  assertReleaseVueCoverage(artifactName, divergence.baseline?.coverage);
  assertReleaseMutationOracle(artifactName, divergence.mutationOracle);
  assertReleaseDependencyLink({ artifactName, divergence, dependency, dependencySha256 });
}

function assertReleaseVueCoverage(artifactName, coverage) {
  if (
    coverage?.verdict !== "usable" ||
    !Number.isSafeInteger(coverage.sharedVueFileCount) ||
    !Number.isSafeInteger(coverage.vizeVueFileCount) ||
    !Number.isSafeInteger(coverage.baselineVueFileCount) ||
    coverage.sharedVueFileCount <= 0 ||
    coverage.vizeVueFileCount !== coverage.baselineVueFileCount ||
    coverage.sharedVueFileCount !== coverage.vizeVueFileCount ||
    coverage.vizeVueFilesSha256 !== coverage.baselineVueFilesSha256 ||
    !isSha256(coverage.vizeVueFilesSha256) ||
    !Array.isArray(coverage.missingVueFiles) ||
    !Array.isArray(coverage.unexpectedVueFiles) ||
    coverage.missingVueFiles.length !== 0 ||
    coverage.unexpectedVueFiles.length !== 0
  ) {
    throw new Error(
      `${artifactName} did not prove both tools checked the same non-empty authored Vue corpus`,
    );
  }
}

function assertReleaseMutationOracle(artifactName, mutationOracle) {
  const states = mutationOracle?.states ?? [];
  const [clean, broken, repaired] = states;
  if (
    mutationOracle?.schema !== "vize.fixtureTypecheckSeededMutationOracle" ||
    mutationOracle.version !== 1 ||
    mutationOracle.passed !== true ||
    mutationOracle.verdict !== "passed" ||
    clean?.name !== "clean" ||
    broken?.name !== "broken" ||
    repaired?.name !== "repaired" ||
    clean.falsePositiveCount !== 0 ||
    clean.falseNegativeCount !== 0 ||
    broken.sharedCount !== 1 ||
    broken.falsePositiveCount !== 0 ||
    broken.falseNegativeCount !== 0 ||
    repaired.sourceSha256 !== clean.sourceSha256 ||
    repaired.falsePositiveCount !== 0 ||
    repaired.falseNegativeCount !== 0
  ) {
    throw new Error(`${artifactName} has no passing seeded mutation oracle`);
  }
}

function assertReleaseDependencyLink({ artifactName, divergence, dependency, dependencySha256 }) {
  if (
    dependency.schema !== "vize.fixtureTypecheckDependencyInstall" ||
    dependency.version !== 2 ||
    dependency.project !== divergence.project ||
    dependency.revision !== divergence.revision ||
    dependency.evidence?.commitSha !== divergence.evidence?.commitSha
  ) {
    throw new Error(`${artifactName} typecheck dependency evidence is not bound to divergence`);
  }
  if (
    divergence.preparation?.schema !== "vize.fixtureTypecheckPreparationEvidence" ||
    divergence.preparation.version !== 1 ||
    divergence.preparation.payloadSha256 !== dependencySha256
  ) {
    throw new Error(
      `${artifactName} divergence artifact is missing dependency preparation linkage`,
    );
  }
}

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function assertArtifactBoundToRun(run, artifact) {
  if (artifact.expired === true) {
    throw new Error(`Real Project Matrix artifact ${String(artifact.name)} has expired`);
  }
  const source = artifact.workflow_run;
  if (
    source != null &&
    (Number(source.id) !== Number(run.id) ||
      source.head_sha !== run.head_sha ||
      source.head_branch !== run.head_branch)
  ) {
    throw new Error(
      `Real Project Matrix artifact ${String(artifact.name)} is not bound to run ${String(run.id)}`,
    );
  }
}

function exactMatchingEntries(entries, pattern, label) {
  const matches = entryNames(entries)
    .filter((name) => pattern.test(name))
    .sort((left, right) => left.localeCompare(right))
    .map((name) => ({ name, text: readTextEntry(entries, name, label) }));
  if (matches.length !== 1) {
    throw new Error(`${label} must be present exactly once; found ${matches.length}`);
  }
  return matches;
}

function readJsonEntry(entries, name, label) {
  return parseJsonText(readTextEntry(entries, name, label), name);
}

function parseJsonText(text, name) {
  try {
    return JSON.parse(text);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`Invalid release evidence JSON ${name}: ${detail}`, { cause: error });
  }
}

function readTextEntry(entries, name, label) {
  const value =
    entries instanceof Map
      ? entries.get(name)
      : entries != null && typeof entries === "object"
        ? entries[name]
        : undefined;
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${label} is missing ${name}`);
  }
  return value;
}

function entryNames(entries) {
  if (entries instanceof Map) return [...entries.keys()];
  if (entries != null && typeof entries === "object") return Object.keys(entries);
  throw new Error("Real Project Matrix artifact entries must be a map or object");
}

function collectTextEntries(root, directory = root, entries = new Map()) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const absolute = join(directory, entry.name);
    if (entry.isDirectory()) {
      collectTextEntries(root, absolute, entries);
    } else if (entry.isFile()) {
      entries.set(relative(root, absolute).replaceAll("\\", "/"), readFileSync(absolute, "utf8"));
    } else {
      throw new Error(`Real Project Matrix artifact contains unsupported entry: ${entry.name}`);
    }
  }
  if (directory === root && entries.size === 0 && statSync(root).isDirectory()) {
    throw new Error("Real Project Matrix artifact is empty");
  }
  return entries;
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}
