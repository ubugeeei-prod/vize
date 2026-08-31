import assert from "node:assert/strict";

import { requiredReleaseWorkflowEvidence } from "../../../legacy-tools/github/release-preflight-evidence.mjs";

export const releaseSha = "a".repeat(40);

export function successfulReleaseRun(name: string, id: number) {
  const evidence = requiredReleaseWorkflowEvidence.get(name);
  assert.ok(evidence, `Missing release evidence configuration for ${name}`);
  return {
    id,
    name,
    display_title: `${name} release evidence`,
    path: evidence.path,
    event: evidence.events[0],
    head_branch: ["push", "schedule"].includes(evidence.events[0]) ? "main" : "v1.2.3",
    head_sha: releaseSha,
    status: "completed",
    conclusion: "success",
    created_at: `2026-07-12T00:${String(id).padStart(2, "0")}:00Z`,
    run_started_at: `2026-07-12T00:${String(id).padStart(2, "0")}:00Z`,
    updated_at: `2026-07-12T00:${String(id).padStart(2, "0")}:00Z`,
    html_url: `https://example.test/runs/${id}`,
  };
}

export function successfulReleaseJob(name: string) {
  return { name, status: "completed", conclusion: "success" };
}
