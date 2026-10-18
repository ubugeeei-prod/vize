import assert from "node:assert/strict";
import {
  createCheckObserver,
  type Candidate,
  type CheckedRun,
  type Options,
  type ReadOptions,
} from "./check-state.ts";

const fullQualificationLimit = 345 * 60 * 1000;
export type StartedCheck = {
  kind: "vize-release-check-start-v1";
  repository: string;
  branch: string;
  head: string;
  cut: string;
  runId: number;
  runAttempt: number;
  createdAt: string;
  qualificationDeadline: number;
};

export function parseQualificationDeadline(value: string): number {
  assert.match(
    value,
    /^[1-9][0-9]*$/u,
    "qualification deadline must be canonical Unix milliseconds",
  );
  const deadline = Number(value);
  assert.ok(Number.isSafeInteger(deadline), "qualification deadline must be an exact integer");
  return deadline;
}

function existingQualificationCap(run: CheckedRun, now: number): number {
  assert.equal(typeof run.created_at, "string", "existing Check creation time required");
  assert.match(String(run.created_at), /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,3})?Z$/u);
  const created = Date.parse(String(run.created_at));
  assert.ok(
    Number.isSafeInteger(created) && created > 0 && created <= now,
    "invalid existing Check creation time",
  );
  return created + fullQualificationLimit;
}

function assertActive(run: CheckedRun): void {
  assert.ok(
    ["queued", "in_progress", "waiting", "requested", "pending", "completed"].includes(run.status),
    "unknown existing Check status",
  );
  assert.equal(
    run.conclusion,
    run.status === "completed" ? "success" : null,
    `Existing Check ${run.id}/${run.run_attempt} failed or was cancelled, or has an inconsistent conclusion: ${run.conclusion}`,
  );
}

function assertSameRun(current: CheckedRun, expected: CheckedRun): void {
  for (const key of [
    "id",
    "run_attempt",
    "head_sha",
    "head_branch",
    "event",
    "path",
    "display_title",
    "created_at",
  ])
    assert.equal(current[key], expected[key], "latest Check changed during the read-only join");
}

/** The official operator starts Q once; pending qualification does not block R's builds. */
export async function ensureCheckStarted(
  candidate: Candidate,
  options: Options,
): Promise<StartedCheck> {
  const observer = createCheckObserver(candidate, options);
  let current = observer.latest(await observer.listRuns());
  if (!current) {
    const [plan] = observer.plans;
    await observer.branchStillFrozen();
    observer.log(
      `Dispatching full Check for frozen ${candidate.head} on ${candidate.branch}; publication remains gated.`,
    );
    await options.api.dispatch(`actions/workflows/${plan.workflowId}/dispatches`, {
      ref: plan.ref,
      inputs: plan.inputs,
    });
    observer.remaining();
    // Reuse the existing qualification poll and absolute deadline; never dispatch twice.
    do {
      current = observer.latest(await observer.listRuns());
      if (!current) await observer.sleep(15_000);
    } while (!current);
  }
  assertActive(current);
  const deadline = Math.min(options.deadline, existingQualificationCap(current, observer.now()));
  if (current.status === "completed") await observer.finish(current);
  else {
    assert.ok(deadline > observer.now(), "full Check qualification deadline expired");
    const failure = await observer.requiredFailure(current);
    assert.equal(failure, undefined, failure ?? undefined);
    const latest = observer.latest(await observer.listRuns());
    assert.ok(latest, "exact existing Check disappeared during startup handoff");
    assertSameRun(latest, current);
    assertActive(latest);
    current = latest;
    if (current.status === "completed") await observer.finish(current);
    await observer.branchStillFrozen();
  }
  observer.log(
    `Started frozen Check ${current.id}/${current.run_attempt} for ${candidate.head}; qualification and release builds must still join before publication.`,
  );
  return {
    kind: "vize-release-check-start-v1",
    ...candidate,
    runId: current.id,
    runAttempt: current.run_attempt,
    createdAt: String(current.created_at),
    qualificationDeadline: deadline,
  };
}

/** Read capabilities only: this join cannot bootstrap, dispatch, rerun, or renew Q. */
export async function waitForExistingCheck(candidate: Candidate, options: ReadOptions) {
  const observer = createCheckObserver(candidate, options);
  let current = observer.latest(await observer.listRuns(true));
  assert.ok(current, "exact existing Check is missing; the release join never bootstraps Q");
  assertActive(current);
  const cap = existingQualificationCap(current, observer.now());
  assert.ok(
    Number.isSafeInteger(options.deadline) && options.deadline > 0,
    "valid qualification deadline required",
  );
  assert.ok(
    options.deadline <= cap,
    "qualification deadline exceeds the existing Check's 345-minute cap",
  );
  assert.ok(
    options.deadline >= cap - fullQualificationLimit,
    "qualification deadline predates Check creation",
  );
  // An official resume may authenticate already completed evidence, even after the
  // old wait deadline. It gets no renewed polling, backoff, or qualification budget.
  if (current.status === "completed") return observer.finish(current, true);
  const initial = current;
  while (current.status !== "completed") {
    observer.remaining();
    const failure = await observer.requiredFailure(current);
    assert.equal(failure, undefined, failure ?? undefined);
    await observer.sleep(15_000);
    const latest = observer.latest(await observer.listRuns());
    assert.ok(latest, "exact existing Check disappeared during the read-only join");
    assertSameRun(latest, initial);
    assertActive(latest);
    current = latest;
  }
  return observer.finish(current);
}
