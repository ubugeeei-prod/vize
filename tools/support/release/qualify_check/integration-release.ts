import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { assertRun, type CheckedRun, type ReadApi, type Row } from "./check-state.ts";
import { createIntegrationReadApi } from "./integration-http.ts";
import {
  qualifyIntegration,
  type IntegrationOptions,
  type IntegrationReceipt,
} from "./integration.ts";

type ReleaseIdentity = {
  id: number;
  attempt: number;
  createdAt: string;
  headSha: string;
  headBranch: string;
  displayTitle: string;
  path: string;
  event: string;
  actor: Row;
  triggeringActor: Row;
};
type CompletedJob = { id: number; runAttempt: number; name: string; sha256: string };
export type IntegrationReleaseReceipt = {
  kind: "vize.release.integration-release-v1";
  qualification: IntegrationReceipt;
  release: ReleaseIdentity;
  readyJob: Row;
  completedJobs: CompletedJob[];
};
export type Readiness = { ready: false } | { ready: true; readyJob: Row };
export type IntegrationReleaseOptions = IntegrationOptions & {
  readiness?: (jobs: Row[], remaining: number | null) => Promise<Readiness>;
};
const object = (value: unknown): Row => {
  assert.ok(
    value !== null && typeof value === "object" && !Array.isArray(value),
    "object required",
  );
  return value as Row;
};
const positive = (value: unknown): number => {
  assert.ok(
    typeof value === "number" && Number.isSafeInteger(value) && value > 0,
    "positive ID required",
  );
  return value;
};
function canonical(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonical((value as Row)[key])}`)
      .join(",")}}`;
  }
  const text = JSON.stringify(value);
  assert.notEqual(text, undefined, "actual JSON packet required");
  return text!;
}
export function releaseJobDigest(job: Row): string {
  return createHash("sha256").update(canonical(job)).digest("hex");
}
function identity(run: CheckedRun): ReleaseIdentity {
  const createdAt = run.created_at;
  assert.ok(typeof createdAt === "string", "R timestamp required");
  assert.ok(Number.isFinite(Date.parse(createdAt)), "R timestamp required");
  return {
    id: run.id,
    attempt: run.run_attempt,
    createdAt,
    headSha: String(run.head_sha),
    headBranch: String(run.head_branch),
    displayTitle: String(run.display_title),
    path: String(run.path),
    event: String(run.event),
    actor: object(run.actor),
    triggeringActor: object(run.triggering_actor),
  };
}
function timestamp(value: unknown, now: number): number {
  assert.ok(typeof value === "string", "canonical R/O timestamp required");
  assert.match(value, /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,3})?Z$/u);
  const parsed = Date.parse(value);
  assert.ok(
    Number.isSafeInteger(parsed) && parsed > 0 && parsed <= now,
    "R/O timestamp is invalid/future",
  );
  return parsed;
}
async function releaseRun(q: IntegrationReceipt, api: ReadApi, now: number): Promise<CheckedRun> {
  const { context, repository } = q;
  const title = `Pinned Release ${context.tag} PR #${context.source} @ ${context.H}`;
  const workflow = await api.read("actions/workflows/release.yml");
  positive(workflow.id);
  assert.equal(workflow.path, ".github/workflows/release.yml");
  assert.equal(workflow.name, "Release");
  const source = await api.read(`contents/.github/workflows/release.yml?ref=${context.H}`);
  assert.equal(source.path, ".github/workflows/release.yml");
  assert.equal(source.encoding, "base64");
  assert.equal(typeof source.content, "string");
  const bytes = Buffer.from(String(source.content), "base64");
  assert.equal(
    source.sha,
    createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
    "R workflow Git blob changed",
  );
  assert.equal(
    bytes
      .toString("utf8")
      .split("\n")
      .filter((line) => line.startsWith("name:"))
      .join("\n"),
    `name: ${workflow.name}`,
  );
  const matches = (
    await api.pages("actions/workflows/release.yml/runs", "workflow_runs", {
      event: "workflow_dispatch",
      head_sha: context.H,
    })
  ).filter(
    (row) =>
      row.head_sha === context.H &&
      row.display_title === title &&
      row.path === ".github/workflows/release.yml" &&
      row.event === "workflow_dispatch",
  );
  for (const row of matches) assertRun(row);
  matches.sort((a, b) => Number(b.id) - Number(a.id));
  assert.ok(matches.length > 0, "existing exact pinned R missing; no bootstrap permitted");
  assert.equal(matches.filter((row) => row.id === matches[0].id).length, 1, "ambiguous latest R");
  const listed = matches[0];
  const listedId = positive(listed.id);
  const run = await api.read(`actions/runs/${listedId}`);
  assertRun(run);
  for (const key of [
    "id",
    "run_attempt",
    "head_sha",
    "head_branch",
    "display_title",
    "path",
    "event",
    "created_at",
  ])
    assert.deepEqual(run[key], listed[key], "R inventory/current identity changed");
  assert.equal(run.workflow_id, workflow.id, "R workflow ID changed");
  const created = timestamp(run.created_at, now);
  assert.ok(created >= timestamp(q.checkCreatedAt, now), "R predates original Q");
  assert.equal(run.head_sha, context.H);
  assert.equal(run.head_branch, `release/${context.tag}`);
  assert.equal(run.display_title, title);
  assert.equal(run.path, ".github/workflows/release.yml");
  assert.equal(run.event, "workflow_dispatch");
  for (const field of ["repository", "head_repository"])
    assert.equal(object(run[field]).full_name, repository, "R repository changed");
  if (run.status === "completed") assert.equal(run.conclusion, "success", "R failed/cancelled");
  else {
    assert.ok(["queued", "in_progress", "waiting", "pending", "requested"].includes(run.status));
    assert.equal(run.conclusion, null);
  }
  const operator =
    q.operatorRunId === null ? null : await api.read(`actions/runs/${q.operatorRunId}`);
  if (operator) assert.ok(created >= timestamp(operator.created_at, now), "R predates original O");
  for (const field of ["actor", "triggering_actor"]) {
    const actor = object(run[field]);
    const login = actor.login;
    assert.equal(actor.type, "User", "genuine R maintainer required");
    assert.ok(typeof login === "string");
    assert.match(login, /^[a-zA-Z0-9][a-zA-Z0-9-]*$/u);
    if (operator) assert.equal(actor.login, object(operator.actor).login, "R/O actor changed");
    else {
      const permission = await api.read(`collaborators/${login}/permission`);
      assert.ok(
        ["maintain", "admin"].includes(String(permission.role_name)),
        "R maintainer required",
      );
    }
  }
  return run;
}
function validateJobs(jobs: Row[], run: CheckedRun): void {
  const ids = new Set<number>();
  for (const job of jobs) {
    const id = positive(job.id);
    assert.ok(!ids.has(id), "duplicate R job ID");
    ids.add(id);
    assert.equal(job.run_id, run.id, "foreign R job");
    assert.equal(job.head_sha, run.head_sha, "R job source changed");
    const attempt = positive(job.run_attempt);
    assert.ok(attempt <= run.run_attempt, "future R job attempt");
    if (attempt < run.run_attempt) {
      assert.equal(job.status, "completed", "retained R job must be completed");
      assert.equal(job.conclusion, "success", "only original successful jobs survive rerun");
    }
    assert.equal(typeof job.name, "string");
  }
}
export function completedReleaseJobIndex(jobs: Row[]): CompletedJob[] {
  return jobs
    .filter((job) => job.status === "completed")
    .map((job) => ({
      id: positive(job.id),
      runAttempt: positive(job.run_attempt),
      name: String(job.name),
      sha256: releaseJobDigest(job),
    }))
    .sort((a, b) => a.id - b.id);
}
export async function qualifyIntegrationRelease(
  input: unknown,
  options: IntegrationReleaseOptions,
): Promise<IntegrationReleaseReceipt> {
  const previous = options.mode === "verify" ? object(input) : undefined;
  if (previous) {
    assert.deepEqual(
      Object.keys(previous).sort(),
      ["kind", "qualification", "release", "readyJob", "completedJobs"].sort(),
    );
    assert.equal(previous.kind, "vize.release.integration-release-v1");
  }
  // Immutable Git objects can be reused within this invocation; refs, PRs, O and runs stay fresh.
  const immutable = new Map<string, Row>();
  const cachedApi: ReadApi = {
    read: async (resource) => {
      const cached = immutable.get(resource);
      if (cached) return cached;
      const value = await options.api.read(resource);
      if (
        /^git\/commits\/[0-9a-f]{40}$/u.test(resource) ||
        /^contents\/\.github\/workflows\/release(?:-operator)?\.yml\?ref=[0-9a-f]{40}$/u.test(
          resource,
        )
      )
        immutable.set(resource, value);
      return value;
    },
    pages: (resource, collection, query) => options.api.pages(resource, collection, query),
  };
  const checkedOptions = { ...options, api: cachedApi };
  const qualification = await qualifyIntegration(previous ? previous.qualification : input, {
    ...checkedOptions,
    mode: "verify",
  });
  const now = options.now ?? Date.now;
  const deadline = qualification.qualificationDeadline;
  let pending = false;
  const remaining = () => {
    assert.ok(deadline !== null, "legacy/resume lease cannot wait for pending R");
    const value = deadline - now();
    assert.ok(
      Number.isSafeInteger(value) && value > 0,
      "original R qualification deadline expired",
    );
    return value;
  };
  const api: ReadApi = {
    read: async (resource) => {
      if (pending) remaining();
      const value = await cachedApi.read(resource);
      options.observe?.({ resource, value });
      if (pending) remaining();
      return value;
    },
    pages: async (resource, collection, query) => {
      if (pending) remaining();
      const value = await cachedApi.pages(resource, collection, query);
      options.observe?.({ resource, collection, query, value });
      if (pending) remaining();
      return value;
    },
  };
  const initial = identity(await releaseRun(qualification, api, now()));
  if (previous) assert.deepEqual(previous.release, initial, "catalog R/latest attempt changed");
  for (;;) {
    if (pending) remaining();
    assert.deepEqual(
      await qualifyIntegration(qualification, { ...checkedOptions, mode: "verify" }),
      qualification,
    );
    const current = await releaseRun(qualification, api, now());
    assert.deepEqual(identity(current), initial, "latest R identity/attempt changed");
    const jobs = await api.pages(`actions/runs/${current.id}/jobs`, "jobs", { filter: "latest" });
    validateJobs(jobs, current);
    if (previous) {
      const retained = previous.completedJobs;
      assert.ok(Array.isArray(retained) && retained.length > 0, "completed R custody required");
      const indexed = completedReleaseJobIndex(jobs);
      const ids = new Set<number>();
      for (const row of retained) {
        const entry = object(row);
        const id = positive(entry.id);
        assert.ok(!ids.has(id), "duplicate retained job");
        ids.add(id);
        assert.deepEqual(
          indexed.find((job) => job.id === id),
          entry,
          "whole completed R job changed/disappeared",
        );
      }
      const ready = object(previous.readyJob);
      assert.ok(ids.has(positive(ready.id)), "ready job missing from custody");
      assert.deepEqual(
        jobs.find((job) => job.id === ready.id),
        ready,
        "whole ready job changed",
      );
    } else {
      assert.ok(options.readiness, "original Rust readiness authority required");
      const answer = await options.readiness(
        jobs,
        pending ? remaining() : deadline === null || deadline <= now() ? null : deadline - now(),
      );
      assert.ok(answer.ready === true || answer.ready === false, "invalid Rust readiness answer");
      if (!answer.ready) {
        assert.equal(current.status === "completed", false, "completed R lacks original readiness");
        remaining();
        if (!pending) options.onPendingDeadline?.(deadline!);
        pending = true;
        await (options.sleep ?? ((ms) => new Promise<void>((done) => setTimeout(done, ms))))(
          Math.min(15_000, remaining()),
        );
        remaining();
        continue;
      }
      assert.deepEqual(Object.keys(answer).sort(), ["ready", "readyJob"]);
      assert.deepEqual(
        jobs.find((job) => job.id === answer.readyJob.id),
        answer.readyJob,
        "Rust must return whole actual ready job",
      );
      const result: IntegrationReleaseReceipt = {
        kind: "vize.release.integration-release-v1",
        qualification,
        release: initial,
        readyJob: answer.readyJob,
        completedJobs: completedReleaseJobIndex(jobs),
      };
      assert.ok(
        result.completedJobs.some((job) => job.id === answer.readyJob.id),
        "Rust ready job lacks terminal custody",
      );
      await qualifyIntegration(qualification, { ...checkedOptions, mode: "verify" });
      assert.deepEqual(
        identity(await releaseRun(qualification, api, now())),
        initial,
        "final R identity changed",
      );
      if (pending) remaining();
      assert.ok(
        Buffer.byteLength(JSON.stringify(result)) <= 16_384,
        "combined receipt exceeds same-run output bound",
      );
      return result;
    }
    await qualifyIntegration(qualification, { ...checkedOptions, mode: "verify" });
    assert.deepEqual(
      identity(await releaseRun(qualification, api, now())),
      initial,
      "final R identity changed",
    );
    return previous as IntegrationReleaseReceipt;
  }
}
async function main(): Promise<void> {
  const [mode, ...extra] = process.argv.slice(2);
  assert.ok(
    (mode === "--wait" || mode === "--verify") && extra.length === 0,
    "use --wait or --verify",
  );
  assert.equal(process.env.GITHUB_EVENT_NAME, "merge_group");
  const repository = process.env.GITHUB_REPOSITORY!;
  const candidateSha = process.env.GITHUB_SHA!;
  const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN;
  assert.ok(token, "read-only token required");
  const evidence = process.env.VIZE_RELEASE_INTEGRATION_EVIDENCE;
  assert.ok(
    evidence && path.isAbsolute(evidence),
    "absolute raw integration evidence path required",
  );
  fs.mkdirSync(path.dirname(evidence), { recursive: true });
  const chunks: Buffer[] = [];
  let length = 0;
  for await (const chunk of process.stdin) {
    length += Buffer.byteLength(chunk);
    assert.ok(length <= 16_384, "bounded same-run receipt required");
    chunks.push(Buffer.from(chunk));
  }
  const transport = createIntegrationReadApi({
    repository,
    token,
    apiUrl: process.env.GITHUB_API_URL,
  });
  const receipt = await qualifyIntegrationRelease(
    JSON.parse(Buffer.concat(chunks).toString("utf8")),
    {
      repository,
      candidateSha,
      mode: mode === "--wait" ? "wait" : "verify",
      api: transport.api,
      onPendingDeadline: transport.setPendingDeadline,
      log: console.error,
      observe: (snapshot) => fs.appendFileSync(evidence, `${JSON.stringify(snapshot)}\n`),
      readiness: async (jobs, remaining) => {
        const input = JSON.stringify(jobs);
        assert.ok(
          Buffer.byteLength(input) <= 2 * 1024 * 1024,
          "full R job inventory exceeds Rust bound",
        );
        const result = spawnSync(
          "rust-script",
          ["tools/commands/release/pr.rs", "check-release-readiness"],
          {
            input,
            encoding: "utf8",
            timeout: Math.max(1, Math.min(30_000, remaining ?? 30_000)),
            maxBuffer: 2 * 1024 * 1024,
          },
        );
        if (result.stderr) process.stderr.write(result.stderr);
        assert.ifError(result.error);
        assert.equal(result.status, 0, "original Rust readiness refused R");
        return JSON.parse(result.stdout);
      },
    },
  );
  process.stdout.write(`${JSON.stringify(receipt)}\n`);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
