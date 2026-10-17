import { isDeepStrictEqual } from "node:util";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import {
  createReleaseGateDispatchPlans,
  releaseGateRunQualifiers,
} from "../compat/github/release-preflight-bootstrap.mjs";
import {
  latestRequiredWorkflowRun,
  requiredWorkflowJobNames,
} from "../compat/github/release-preflight-evidence.mjs";

type Row = Record<string, unknown>;

function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`Pre-release qualification: ${message}`);
}
function object(value: unknown, label: string): Row {
  requireValue(
    value !== null && typeof value === "object" && !Array.isArray(value),
    `${label} object required`,
  );
  return value as Row;
}
function positive(value: unknown, label: string): number {
  requireValue(
    typeof value === "number" && Number.isSafeInteger(value) && value > 0,
    `${label} positive safe integer required`,
  );
  return value;
}
function text(value: unknown, label: string): string {
  requireValue(typeof value === "string" && value.length > 0, `${label} string required`);
  return value;
}
function array(value: unknown, label: string, max = 10_000): unknown[] {
  requireValue(Array.isArray(value) && value.length <= max, `${label} bounded array required`);
  return value;
}
function inventory(value: unknown, key: "workflow_runs" | "jobs", label: string): Row[] {
  const pages = array(value, `${label} pages`, 100);
  requireValue(pages.length > 0, `${label} complete pages required`);
  let total: number | undefined;
  const rows: Row[] = [];
  const ids = new Set<number>();
  for (const [index, raw] of pages.entries()) {
    const page = object(raw, `${label} page`);
    requireValue(
      !("error" in page) && !("errors" in page) && !("message" in page),
      `${label} API error page`,
    );
    const count = page.total_count;
    requireValue(
      typeof count === "number" && Number.isSafeInteger(count) && count >= 0 && count <= 10_000,
      `${label} bounded total required`,
    );
    requireValue(total === undefined || count === total, `${label} page totals differ`);
    total = count;
    const entries = array(page[key], `${label} rows`, 100);
    requireValue(
      index === pages.length - 1 || entries.length === 100,
      `${label} incomplete intermediate page`,
    );
    for (const rawRow of entries) {
      const row = object(rawRow, `${label} row`);
      const id = positive(row.id, `${label} ID`);
      requireValue(!ids.has(id), `${label} duplicate ID ${id}`);
      ids.add(id);
      rows.push(row);
    }
  }
  requireValue(rows.length === total, `${label} incomplete inventory total`);
  return rows;
}
function sameRows(left: Row[], right: Row[], label: string) {
  const byId = new Map(right.map((row) => [row.id, row]));
  requireValue(
    left.length === right.length && left.every((row) => isDeepStrictEqual(row, byId.get(row.id))),
    `${label} inventory differs`,
  );
}
function state(row: Row, label: string, terminal = false) {
  const status = text(row.status, `${label} status`);
  requireValue(
    ["queued", "in_progress", "waiting", "pending", "requested", "completed"].includes(status),
    `${label} unknown status`,
  );
  requireValue(!terminal || status === "completed", `${label} must be terminal`);
  if (status === "completed") {
    requireValue(
      [
        "success",
        "failure",
        "timed_out",
        "cancelled",
        "action_required",
        "neutral",
        "skipped",
        "stale",
        "startup_failure",
      ].includes(text(row.conclusion, `${label} conclusion`)),
      `${label} unknown conclusion`,
    );
  } else requireValue(row.conclusion === null, `${label} nonterminal conclusion`);
}
function own(run: Row, repository: string, label: string) {
  positive(run.id, `${label} ID`);
  positive(run.run_attempt, `${label} attempt`);
  for (const key of ["repository", "head_repository"]) {
    const repo = object(run[key], `${label} ${key}`);
    requireValue(repo.full_name === repository, `${label} foreign repository`);
    positive(repo.id, `${label} ${key} ID`);
  }
  state(run, label);
}
function jobsForRun(jobs: Row[], run: Row, seenIds: Set<number>, label: string) {
  const names = new Set<string>();
  for (const job of jobs) {
    const id = positive(job.id, `${label} job ID`);
    requireValue(!seenIds.has(id), `${label} duplicate/cross-run job ID ${id}`);
    seenIds.add(id);
    const attempt = positive(job.run_attempt, `${label} job attempt`);
    requireValue(
      job.run_id === run.id &&
        job.head_sha === run.head_sha &&
        attempt <= positive(run.run_attempt, `${label} attempt`),
      `${label} job run/head/attempt differs`,
    );
    const name = text(job.name, `${label} job name`);
    const key = JSON.stringify([attempt, name]);
    requireValue(!names.has(key), `${label} duplicate job name in attempt: ${name}`);
    names.add(key);
    state(job, `${label} job ${name}`, true);
    requireValue(Array.isArray(job.steps), `${label} job steps required`);
  }
}
function hRuns(value: unknown, repository: string, head: string, label: string): Row[] {
  const rows = inventory(value, "workflow_runs", label);
  for (const row of rows) {
    own(row, repository, label);
    requireValue(row.head_sha === head, `${label} foreign H`);
    const path = text(row.path, `${label} workflow path`);
    // GitHub also emits dynamic/* paths; retain its identity rather than resolving filenames.
    for (const field of ["event", "head_branch", "display_title"])
      text(row[field], `${label} ${field}`);
    const started = text(
      row.run_started_at ?? row.created_at ?? row.updated_at,
      `${label} ordering timestamp`,
    );
    requireValue(
      Number.isFinite(Date.parse(started)),
      `${label} finite ordering timestamp required`,
    );
    requireValue(
      path !== ".github/workflows/release.yml",
      `${label} Release run forbids pre-release retirement`,
    );
  }
  return rows;
}
function selectedQualification(rows: Row[], identity: Row, qualification: Row) {
  const head = text(identity.head, "identity H");
  const plans = createReleaseGateDispatchPlans({
    ref: `release/${text(identity.tag, "identity tag")}`,
    headSha: head,
    baseSha: text(identity.cut, "identity C"),
  });
  const qualifier = releaseGateRunQualifiers(plans).get("Check");
  const latest = latestRequiredWorkflowRun(rows, head, "Check", qualifier);
  requireValue(
    latest && isDeepStrictEqual(latest, qualification),
    "latest eligible full Check identity/attempt differs or is missing",
  );
}

/** Validate captured Q/O evidence only. This grants no publication or replacement-cut authority. */
export function assertPreReleaseQualification(receiptOrGuardsEnvelope: unknown): void {
  const envelope = object(receiptOrGuardsEnvelope, "envelope");
  const guards = "guards" in envelope ? object(envelope.guards, "guards") : envelope;
  const identity = object(guards.identity, "identity");
  if ("guards" in envelope) {
    requireValue(isDeepStrictEqual(envelope.identity, identity), "receipt/guard identity differs");
    if ("schema" in envelope)
      requireValue(envelope.schema === "vize-unpublished-retirement-v3", "v3 receipt required");
  }
  requireValue(identity.origin === "pre-release-qualification", "pre-release origin required");
  requireValue(
    !("releaseRun" in identity) &&
      !("releaseAttempt" in identity) &&
      !("releaseRun" in guards) &&
      !("publicationJobs" in guards),
    "pre-release identity must not contain Release/publication authority",
  );
  const repository = text(identity.repository, "identity repository");
  requireValue(
    /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository),
    "repository owner/name required",
  );
  const head = text(identity.head, "identity H");
  const cut = text(identity.cut, "identity C");
  requireValue(
    /^[a-f0-9]{40}$/.test(head) && /^[a-f0-9]{40}$/.test(cut) && head !== cut,
    "distinct full H/C required",
  );
  const tag = text(identity.tag, "identity tag");
  requireValue(/^v0\.(?:0|[1-9][0-9]*)\.0$/.test(tag), "stable minor tag required");
  const qid = positive(identity.qualificationRun, "identity Q");
  const attempt = positive(identity.qualificationAttempt, "identity Q attempt");
  const oid = positive(identity.operatorRun, "identity O");
  requireValue(qid !== oid, "Q and O must differ");
  const source = object(guards.sourcePr, "source PR");
  requireValue(
    source.number === positive(identity.sourcePr, "identity source PR"),
    "source PR identity differs",
  );
  const author = text(object(source.user, "source PR user").login, "source PR author");
  const qualification = object(guards.qualificationRun, "Q");
  own(qualification, repository, "Q");
  requireValue(
    qualification.id === qid &&
      qualification.run_attempt === attempt &&
      qualification.head_sha === head &&
      qualification.head_branch === `release/${tag}`,
    "Q identity/head/branch/attempt differs",
  );
  requireValue(
    qualification.path === ".github/workflows/check.yml" &&
      qualification.event === "workflow_dispatch",
    "Q must be dispatched Check workflow",
  );
  for (const key of ["actor", "triggering_actor"])
    requireValue(
      object(qualification[key], `Q ${key}`).login === author,
      "Q source author custody differs",
    );
  requireValue(
    qualification.status === "completed" &&
      ["failure", "timed_out"].includes(String(qualification.conclusion)),
    "Q must be terminal failed/timed_out, never success/pending/cancel-only",
  );
  const operator = object(guards.operatorRun, "O");
  own(operator, repository, "O");
  requireValue(
    operator.id === oid &&
      operator.run_attempt === positive(identity.operatorAttempt, "identity O attempt"),
    "O identity/attempt differs",
  );
  const absence = object(guards.releaseRunAbsence, "Release run absence");
  requireValue(
    inventory(absence.pages, "workflow_runs", "Release-only").length === 0,
    "any Release-only row forbids pre-release retirement",
  );
  const rows = hRuns(absence.hRunPages, repository, head, "all H");
  selectedQualification(rows, identity, qualification);
  const failurePages = object(guards.originalFailureJobPages, "original failure pages");
  requireValue(!("release" in failurePages), "pre-release evidence cannot contain R jobs");
  const jobs = inventory(failurePages.qualification, "jobs", "Q jobs");
  const listedJobs = array(guards.qualificationJobs, "qualificationJobs").map((value) =>
    object(value, "qualification job"),
  );
  requireValue(
    new Set(listedJobs.map((job) => positive(job.id, "qualificationJobs ID"))).size ===
      listedJobs.length,
    "qualificationJobs duplicate ID",
  );
  sameRows(jobs, listedJobs, "qualificationJobs/original pages");
  const ids = new Set<number>();
  jobsForRun(jobs, qualification, ids, "Q");
  const operatorJobs = inventory(failurePages.operator, "jobs", "O jobs");
  jobsForRun(operatorJobs, operator, ids, "O");
  for (const name of requiredWorkflowJobNames("Check", qualification)) {
    const matching = jobs.filter((job) => job.name === name);
    requireValue(matching.length > 0, `required Check job missing: ${name}`);
    const latestAttempt = Math.max(
      ...matching.map((job) => positive(job.run_attempt, "required job attempt")),
    );
    requireValue(
      matching.filter((job) => job.run_attempt === latestAttempt).length === 1,
      `required Check job ambiguous: ${name}`,
    );
  }
  requireValue(
    jobs.some(
      (job) =>
        job.run_attempt === attempt && ["failure", "timed_out"].includes(String(job.conclusion)),
    ),
    "actual failed/timed-out job in current Q attempt required",
  );
  if ("originalEvidence" in envelope) {
    const original = object(envelope.originalEvidence, "original evidence");
    const retainedRuns = hRuns(original.HRunPages, repository, head, "original all H");
    selectedQualification(retainedRuns, identity, qualification);
    requireValue(
      rows.length === retainedRuns.length &&
        rows.every((row) =>
          retainedRuns.some(
            (retained) => retained.id === row.id && retained.run_attempt === row.run_attempt,
          ),
        ),
      "original H run identities/attempts changed",
    );
    const observations = array(original.observations, "original observations", 10_001).map(
      (value) => object(value, "observation"),
    );
    const observedIds = observations.map((observation) =>
      positive(object(observation.run, "observation run").id, "observation ID"),
    );
    requireValue(
      new Set(observedIds).size === observedIds.length,
      "duplicate original run observation",
    );
    for (const [run, expectedJobs] of [
      [qualification, jobs],
      [operator, operatorJobs],
    ] as const) {
      const matching = observations.filter(
        (observation) => object(observation.run, "observation run").id === run.id,
      );
      requireValue(
        matching.length === 1 && isDeepStrictEqual(matching[0].run, run),
        "original Q/O observation identity differs",
      );
      sameRows(
        inventory(matching[0].jobPages, "jobs", "original observed jobs"),
        expectedJobs,
        "original Q/O observed jobs",
      );
    }
  }
}

async function main() {
  requireValue(
    process.argv.length === 2,
    "CLI accepts one JSON envelope on stdin and no arguments",
  );
  const chunks: Buffer[] = [];
  let length = 0;
  for await (const chunk of process.stdin) {
    const bytes = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk as string);
    length += bytes.length;
    requireValue(length <= 64 * 1024 * 1024, "CLI envelope exceeds 64 MiB");
    chunks.push(bytes);
  }
  assertPreReleaseQualification(
    JSON.parse(
      new TextDecoder("utf-8", { fatal: true }).decode(Buffer.concat(chunks, length)),
    ) as unknown,
  );
}
if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  main().catch((error: unknown) => {
    console.error(error instanceof Error ? (error.stack ?? error.message) : String(error));
    process.exitCode = 1;
  });
}
