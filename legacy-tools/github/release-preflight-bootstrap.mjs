import {
  acceptedEvidenceShas,
  latestRequiredWorkflowRun,
  requiredReleaseWorkflows,
  selectRequiredWorkflowRuns,
} from "./release-preflight-evidence.mjs";

export function createReleaseGateDispatchPlans({ ref, headSha, baseSha }) {
  for (const [label, value] of [
    ["head", headSha],
    ["base", baseSha],
  ]) {
    if (!/^[0-9a-f]{40}$/.test(value)) {
      throw new Error(`Release ${label} SHA must be a full lowercase commit SHA, got ${value}`);
    }
  }
  if (baseSha === headSha)
    throw new Error("Release base SHA must differ from the release head SHA");
  if (!ref) throw new Error("Release dispatch ref is required");

  // Release evidence replays the known corpus instead of running a fresh
  // campaign. A campaign is a randomized search, so it can fail a tag over an
  // input it discovered minutes earlier that has nothing to do with the release;
  // discovery stays on the nightly schedule, where a find is triaged on its own
  // schedule rather than blocking a publish.
  const fuzzMode = "replay";

  return [
    {
      workflowName: "Benchmark",
      workflowId: "benchmark.yml",
      ref,
      inputs: { base_sha: baseSha, head_sha: headSha },
      expectedRunName: `Benchmark ${baseSha}...${headSha}`,
      acceptsScheduledEvidence: false,
    },
    {
      workflowName: "Fuzz",
      workflowId: "fuzz.yml",
      ref,
      inputs: { mode: fuzzMode },
      expectedRunName: `Fuzz ${fuzzMode} @ ${headSha}`,
      // The nightly schedule runs a campaign, so a scheduled run at the tag SHA
      // is not replay evidence: it must not stand in for the dispatched replay.
      acceptsScheduledEvidence: false,
    },
  ];
}

export function releaseGateRunQualifiers(dispatchPlans) {
  return new Map(
    dispatchPlans.map((plan) => [
      plan.workflowName,
      (run) =>
        (plan.acceptsScheduledEvidence === true && run.event === "schedule") ||
        run.display_title === plan.expectedRunName,
    ]),
  );
}

export async function bootstrapRequiredWorkflowRuns({
  sha,
  dispatchPlans,
  listRuns,
  dispatchWorkflow,
  // Per gate, the SHAs whose runs count as evidence. A version-only release
  // adds its first parent here for the gates that prove the code, so those are
  // never dispatched at all and the release stops waiting on them.
  evidenceShas = new Map(),
  sleep = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds)),
  now = Date.now,
  // Real Project Matrix is no longer a release gate (#4461), so the budget no
  // longer has to cover its two 240-minute waves. Native Smoke is the long pole
  // at 12-40 minutes, App E2E next at 21; 90 minutes leaves room for queueing
  // and a slow runner without letting one stuck gate hold a release all day.
  timeoutMs = 90 * 60 * 1000,
  pollIntervalMs = 15_000,
  onWait = () => {},
}) {
  const qualifiers = releaseGateRunQualifiers(dispatchPlans);
  let runs = await listRuns();
  const dispatchErrors = [];
  for (const plan of dispatchPlans) {
    const run = latestRequiredWorkflowRun(
      runs,
      acceptedEvidenceShas(evidenceShas, plan.workflowName, sha),
      plan.workflowName,
      qualifiers.get(plan.workflowName),
    );
    if (run == null) {
      try {
        await dispatchWorkflow(plan);
      } catch (error) {
        dispatchErrors.push(
          `${plan.workflowName}: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }
  }
  if (dispatchErrors.length > 0) {
    throw new Error(
      `Failed to dispatch release gates:\n${dispatchErrors.map((error) => `- ${error}`).join("\n")}`,
    );
  }

  const deadline = now() + timeoutMs;
  while (true) {
    runs = await listRuns();
    const missing = [];
    const pending = [];
    let failed = false;
    for (const workflowName of requiredReleaseWorkflows) {
      const run = latestRequiredWorkflowRun(
        runs,
        acceptedEvidenceShas(evidenceShas, workflowName, sha),
        workflowName,
        qualifiers.get(workflowName),
      );
      if (run == null) missing.push(workflowName);
      else if (run.status !== "completed") pending.push(workflowName);
      else if (run.conclusion !== "success") failed = true;
    }

    if (failed || (missing.length === 0 && pending.length === 0)) {
      return selectRequiredWorkflowRuns(
        runs,
        sha,
        requiredReleaseWorkflows,
        qualifiers,
        evidenceShas,
      );
    }
    if (now() >= deadline) {
      try {
        return selectRequiredWorkflowRuns(
          runs,
          sha,
          requiredReleaseWorkflows,
          qualifiers,
          evidenceShas,
        );
      } catch (error) {
        const detail = error instanceof Error ? error.message : String(error);
        throw new Error(`Timed out after ${timeoutMs}ms waiting for release gates.\n${detail}`);
      }
    }

    onWait({ missing, pending });
    await sleep(Math.min(pollIntervalMs, Math.max(0, deadline - now())));
  }
}
