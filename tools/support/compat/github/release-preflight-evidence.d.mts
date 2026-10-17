export type WorkflowRun = Record<string, unknown>;
export type EvidenceShas = Map<string, string[]>;
export type RunQualifiers = Map<string, (run: WorkflowRun) => boolean>;
export const requiredReleaseWorkflows: string[];
export const requiredReleaseWorkflowEvidence: Map<
  string,
  { path: string; events: string[]; branches?: Record<string, string[]> }
>;
export function requiredWorkflowJobNames(
  workflowName: string,
  run?: WorkflowRun,
  repoRoot?: string,
): string[];
export function latestRequiredWorkflowRun(
  runs: WorkflowRun[],
  sha: string | string[],
  workflowName: string,
  qualifier?: (run: WorkflowRun) => boolean,
): WorkflowRun | undefined;
export function acceptedEvidenceShas(
  evidenceShas: EvidenceShas | undefined,
  workflowName: string,
  sha: string | string[],
): string[];
export function selectRequiredWorkflowRuns(
  runs: WorkflowRun[],
  sha: string | string[],
  required?: string[],
  qualifiers?: RunQualifiers,
  evidenceShas?: EvidenceShas,
  jobFailureSummaries?: Map<string, string>,
): Map<string, WorkflowRun>;
export function workflowRequiresJobEvidence(workflowName: string): boolean;
export function summarizeRequiredWorkflowJobFailures(
  workflowName: string,
  jobs: WorkflowRun[],
  run?: WorkflowRun,
  repoRoot?: string,
): string | null;
export function assertRequiredWorkflowJobs(
  workflowName: string,
  jobs: WorkflowRun[],
  run?: WorkflowRun,
  repoRoot?: string,
): void;
