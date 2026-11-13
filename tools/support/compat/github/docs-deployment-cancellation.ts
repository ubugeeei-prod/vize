import type { GitHub } from "./docs-deployment-api.ts";
import type { PublisherJob, WorkflowRun } from "./docs-deployment-policy.ts";

// actions/deploy-pages prints these after the Pages API accepted each call.
const createdLine = /^(\S+Z) Created deployment for ([0-9a-f]{40}), ID: ([0-9a-f]{40})\r?$/gmu;
const cancelledLine = /^(\S+Z) Canceled deployment with ID ([0-9a-f]{40})\r?$/gmu;
const successLine = /^\S+Z Reported success!\r?$/mu;

/**
 * A failed Pages step is an uncertain effect unless its own log proves that
 * actions/deploy-pages created exactly one deployment of the publisher source
 * and then cancelled that same deployment (its timeout path does this), and the
 * Pages API does not report that deployment as succeeded. A cancelled Pages
 * deployment is never served, so the step cannot have moved the floor.
 */
export async function cancelledPagesEffect(
  api: GitHub,
  publisher: WorkflowRun,
  job: PublisherJob,
): Promise<boolean> {
  const steps = job.steps.filter((step) => step.name === "Deploy to GitHub Pages");
  const step = steps[0];
  if (steps.length !== 1 || job.status !== "completed" || step?.conclusion !== "failure")
    return false;
  const started = Date.parse(step.started_at ?? "");
  // Step timestamps are whole seconds; log lines carry sub-second precision.
  const completed = Date.parse(step.completed_at ?? "") + 1000;
  if (!Number.isFinite(started) || !Number.isFinite(completed)) return false;
  const log = await api.jobLog(job.id);
  const created = [...log.matchAll(createdLine)];
  const cancelled = [...log.matchAll(cancelledLine)];
  if (created.length !== 1 || cancelled.length !== 1 || successLine.test(log)) return false;
  const [, createdAt, source, id] = created[0]!;
  const [, cancelledAt, cancelledId] = cancelled[0]!;
  const createdTime = Date.parse(createdAt!),
    cancelledTime = Date.parse(cancelledAt!);
  if (
    source !== publisher.head_sha ||
    cancelledId !== id ||
    !(started <= createdTime && createdTime <= cancelledTime && cancelledTime < completed)
  )
    return false;
  const state = await api.response("/pages/deployments/" + id);
  if (!state.ok) return false;
  const { status } = (await state.json()) as { status?: unknown };
  return typeof status === "string" && status !== "succeed";
}
