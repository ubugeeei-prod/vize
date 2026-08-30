import assert from "node:assert/strict";
import { parse } from "yaml";

import { readRepoFile } from "./github-workflows.ts";

export type WorkflowStep = {
  "continue-on-error"?: boolean;
  env?: Record<string, string>;
  if?: string;
  id?: string;
  name?: string;
  run?: string;
  shell?: string;
  uses?: string;
  with?: Record<string, unknown>;
};

export type WorkflowJob = {
  env?: Record<string, string>;
  name?: string;
  "runs-on"?: string;
  steps?: WorkflowStep[];
  strategy?: { "fail-fast"?: boolean; "max-parallel"?: number; matrix?: { shard?: number[] } };
  "timeout-minutes"?: number;
};

export type RealProjectMatrixWorkflow = {
  concurrency?: { "cancel-in-progress"?: boolean; group?: string };
  jobs?: Record<string, WorkflowJob>;
  on?: {
    schedule?: Array<{ cron?: string }>;
    workflow_dispatch?: { inputs?: Record<string, unknown> };
  };
  permissions?: Record<string, string>;
};

export const shardSummaryCommandPath =
  "tools/commands/ci/github/publish-real-project-shard-summary.rs";

export function readRealProjectMatrixWorkflow(): RealProjectMatrixWorkflow {
  return parse(
    readRepoFile(".github", "workflows", "real-project-matrix.yml"),
  ) as RealProjectMatrixWorkflow;
}

export function realProjectMatrixSteps(): WorkflowStep[] {
  return readRealProjectMatrixWorkflow().jobs?.["real-project-matrix"]?.steps ?? [];
}

export function findStep(steps: WorkflowStep[], name: string): WorkflowStep {
  const step = steps.find((candidate) => candidate.name === name);
  assert.ok(step, `Missing '${name}' step`);
  return step;
}

export function readShardSummaryScript(): string {
  return readRepoFile(...shardSummaryCommandPath.split("/"));
}
