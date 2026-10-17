import type { WorkflowRun, EvidenceShas, RunQualifiers } from "./release-preflight-evidence.mjs";
export interface DispatchPlan {
  workflowName: string;
  workflowId: string;
  ref: string;
  inputs: Record<string, unknown>;
  expectedRunName: string;
  acceptsScheduledEvidence: boolean;
}
export function createReleaseGateDispatchPlans(input: {
  ref: string;
  headSha: string;
  baseSha: string;
}): DispatchPlan[];
export function releaseGateRunQualifiers(
  plans: DispatchPlan[],
  evidenceShas?: EvidenceShas,
): RunQualifiers;
export function bootstrapRequiredWorkflowRuns(options: {
  sha: string;
  dispatchPlans: DispatchPlan[];
  listRuns: () => Promise<WorkflowRun[]>;
  dispatchWorkflow: (plan: DispatchPlan) => Promise<unknown>;
  evidenceShas?: EvidenceShas;
  sleep?: (milliseconds: number) => Promise<unknown>;
  now?: () => number;
  timeoutMs?: number;
  pollIntervalMs?: number;
  onWait?: (state: { missing: string[]; pending: string[] }) => void;
  requiredJobFailure?: (
    workflowName: string,
    run: WorkflowRun,
  ) => Promise<string | null | undefined>;
}): Promise<Map<string, WorkflowRun>>;
