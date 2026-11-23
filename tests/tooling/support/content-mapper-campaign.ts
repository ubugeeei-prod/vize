import { runInNewContext } from "node:vm";

export type CampaignInputs = Partial<
  Record<
    | "event-rename-only"
    | "official-css-options-only"
    | "published-defaults-only"
    | "router-experimental-only",
    boolean
  >
>;

// Evaluates a workflow expression for one trigger, as Actions would.
export function evaluateWorkflowExpression(
  expression: string,
  eventName: string,
  inputs: CampaignInputs,
): unknown {
  return runInNewContext(
    expression
      .replace(/^\$\{\{\s*|\s*\}\}$/gu, "")
      .replace(/\binputs\.([a-z-]+)/gu, 'inputs["$1"]'),
    {
      github: { event_name: eventName, event: { pull_request: {} }, ref: "refs/heads/main" },
      inputs,
    },
  );
}

export function selectedJobs(
  jobs: Record<string, { if: string }>,
  eventName: string,
  inputs: CampaignInputs,
): string[] {
  return Object.entries(jobs)
    .filter(([, job]) => evaluateWorkflowExpression(job.if, eventName, inputs))
    .map(([name]) => name);
}
