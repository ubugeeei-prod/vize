import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { operatorDeadline } from "../qualify_check/main.ts";
import { publishedCrates } from "../publish_crates/main.ts";
import {
  assertRequiredWorkflowJobs,
  latestRequiredWorkflowRun,
} from "../../compat/github/release-preflight-evidence.mjs";
import {
  candidate,
  crates,
  decision,
  hasPause,
  positive,
  record,
  withPause,
  type Candidate,
} from "./policy.ts";
import { boundedRequest, canonicalOwner, namespace, sha256, type Request } from "./registry.ts";
import {
  checkpointPath,
  policyFiles,
  readCheckpoint as authenticatedCheckpoint,
  readHandoff as authenticatedHandoff,
} from "./custody.ts";
export { checkpointPath, policyFiles } from "./custody.ts";

export function commands(root: string, deadline: number) {
  const run = (program: string, args: string[], input?: string | Uint8Array, long = false) => {
    const remaining = deadline - Date.now();
    assert.ok(remaining > 0, "original operator deadline expired");
    return execFileSync(program, args, {
      cwd: root,
      input,
      timeout: Math.min(long ? remaining : 30_000, remaining),
      maxBuffer: 16 * 1024 * 1024,
      stdio: [input === undefined ? "ignore" : "pipe", "pipe", "pipe"],
    });
  };
  const git = (...args: string[]) =>
    run("git", ["--no-replace-objects", ...args])
      .toString()
      .trim();
  const api = (resource: string, body?: unknown) =>
    JSON.parse(
      run(
        "gh",
        ["api", resource, ...(body === undefined ? [] : ["--method", "PATCH", "--input", "-"])],
        body === undefined ? undefined : JSON.stringify(body),
      ).toString(),
    );
  return {
    run,
    git,
    api,
    read: (relative: string) => readFileSync(path.join(root, relative), "utf8"),
    exists: (relative: string) => existsSync(path.join(root, relative)),
  };
}
export type Commands = ReturnType<typeof commands>;

/** Reauthenticate H/C and refuse historical, refreshed, already-publishing cuts. */
export function authenticate(value: Candidate, io: Commands, requirePause = false) {
  candidate(value);
  assert.equal(io.git("rev-parse", "HEAD"), value.head);
  assert.equal(io.git("rev-parse", "HEAD^"), value.cut);
  assert.equal(
    io.git("status", "--porcelain", "--untracked-files=normal"),
    "",
    "clean exact H required",
  );
  assert.equal(
    io.git("ls-remote", "origin", `refs/heads/${value.branch}`).split(/\s/u)[0],
    value.head,
  );
  assert.equal(io.git("ls-remote", "origin", "refs/heads/main").split(/\s/u)[0], value.cut);
  assert.equal(
    io.git("ls-remote", "origin", `refs/tags/${value.tag}`),
    "",
    "a historical tag cannot bootstrap",
  );
  const pr = record(io.api(`repos/${value.repository}/pulls/${value.number}`));
  assert.equal(pr.number, value.number);
  assert.equal(pr.state, "open");
  assert.equal(pr.draft, true, "the source must remain a Draft");
  assert.equal(pr.merged, false);
  assert.equal(record(pr.head).sha, value.head);
  assert.equal(record(pr.head).ref, value.branch);
  assert.equal(record(record(pr.head).repo).full_name, value.repository);
  assert.equal(record(pr.base).ref, "main");
  assert.equal(record(pr.base).sha, value.cut);
  assert.equal(record(record(pr.base).repo).full_name, value.repository);
  assert.equal(typeof pr.body, "string");
  if (requirePause)
    assert.ok(hasPause(String(pr.body), value), "source has no qualified bootstrap pause");
  const inventory = record(
    io.api(
      `repos/${value.repository}/actions/workflows/release.yml/runs?event=workflow_dispatch&head_sha=${value.head}&per_page=100`,
    ),
  );
  assert.deepEqual(inventory.workflow_runs, [], "an existing R must use its original resume path");
  const cargo = io.read("Cargo.toml");
  const version = cargo.split("[workspace.package]")[1]?.match(/^version\s*=\s*"([^"]+)"/mu)?.[1];
  assert.equal(`v${version}`, value.tag);
  return pr;
}

export function assertQualification(row: unknown, value: Candidate) {
  const run = record(row);
  positive(run.id, "full Check ID");
  assert.equal(run.head_sha, value.head);
  assert.equal(run.head_branch, value.branch);
  assert.equal(run.path, ".github/workflows/check.yml");
  assert.equal(run.event, "workflow_dispatch");
  assert.equal(run.status, "completed");
  assert.equal(run.conclusion, "success");
  assert.equal(run.display_title, `Check full @ ${value.head}`);
  positive(run.run_attempt, "full Check attempt");
  assert.equal(record(run.repository).full_name, value.repository);
  return run.id;
}

/** Reuse the ordinary full-Check job laws; a green lean dispatch cannot qualify. */
export function assertFullQualification(row: unknown, value: Candidate, io: Commands) {
  const run = structuredClone(record(row));
  const id = assertQualification(run, value);
  const response = record(
    io.api(`repos/${value.repository}/actions/runs/${id}/jobs?filter=latest&per_page=100`),
  );
  assert.ok(
    Array.isArray(response.jobs) &&
      response.jobs.length === response.total_count &&
      Number(response.total_count) <= 100,
  );
  for (const item of response.jobs) {
    const job = record(item);
    assert.equal(job.run_id, id);
    assert.equal(job.head_sha, value.head);
    positive(job.run_attempt, "full Check job attempt");
    assert.ok(job.run_attempt <= Number(run.run_attempt));
  }
  assertRequiredWorkflowJobs("Check", response.jobs, run);
  const inventory = record(
    io.api(
      `repos/${value.repository}/actions/workflows/check.yml/runs?event=workflow_dispatch&head_sha=${value.head}&per_page=100`,
    ),
  );
  assert.ok(Array.isArray(inventory.workflow_runs) && Number(inventory.total_count) <= 100);
  const full = inventory.workflow_runs
    .map(record)
    .filter(
      (item) =>
        item.display_title === `Check full @ ${value.head}` && item.head_branch === value.branch,
    );
  const latest = latestRequiredWorkflowRun(full, value.head, "Check");
  assert.ok(latest, "latest full Check is missing");
  const same = (current: unknown) => {
    assertQualification(current, value);
    const observed = record(current);
    for (const key of [
      "id",
      "name",
      "run_attempt",
      "head_sha",
      "head_branch",
      "event",
      "path",
      "display_title",
      "status",
      "conclusion",
    ]) {
      assert.equal(observed[key], run[key], "full Check changed during qualification");
    }
  };
  same(latest);
  same(io.api(`repos/${value.repository}/actions/runs/${id}`));
  return id;
}

export function readCheckpoint(value: Candidate, io: Commands) {
  return authenticatedCheckpoint(value, io, assertFullQualification);
}
export function readHandoff(value: Candidate, io: Commands) {
  return authenticatedHandoff(value, io, assertFullQualification);
}

export async function preRelease(
  value: Candidate,
  root: string,
  deadline: number,
  dependencies: { io?: Commands; request?: Request; catalogue?: readonly string[] } = {},
) {
  const io = dependencies.io ?? commands(root, deadline);
  assert.deepEqual(
    (dependencies.catalogue ?? publishedCrates).filter((name) => name.startsWith("vize_corsa")),
    crates,
    "H must declare the exact six-package catalogue",
  );
  for (const name of crates) {
    const manifest = io.read(`vendor/${name.slice(5)}/Cargo.toml`);
    assert.ok(manifest.includes(`name = "${name}"`));
    assert.match(manifest, /^version\.workspace\s*=\s*true$/mu);
  }
  const pr = authenticate(value, io);
  const paused = hasPause(String(pr.body), value);
  const request = boundedRequest(deadline, dependencies.request);
  const states: Array<"missing" | "existing"> = [];
  for (const name of crates) states.push(await namespace(name, request));
  const receipt = (paused ? readHandoff(value, io) : undefined) ?? readCheckpoint(value, io);
  if (receipt) {
    await canonicalOwner(receipt.owner, receipt.ownerLogin, request);
    for (const name of crates)
      await canonicalOwner(receipt.owner, receipt.ownerLogin, request, name);
  }
  const result = decision(states, receipt !== undefined);
  if (result !== "pause") {
    console.log(JSON.stringify({ schema: "vize-corsa-pre-r-v1", candidate: value, result }));
    return;
  }
  // Keep the same original deadline. Full Q is required before a bootstrap
  // credential can become relevant; neither R nor the integration queue starts.
  io.run(
    "node",
    [
      "tools/support/release/qualify_check/main.ts",
      value.repository,
      value.branch,
      value.head,
      value.cut,
    ],
    undefined,
    true,
  );
  const current = authenticate(value, io);
  const body = withPause(String(current.body), value);
  if (body !== current.body) {
    io.api(`repos/${value.repository}/pulls/${value.number}`, { body });
    assert.ok(hasPause(String(authenticate(value, io).body), value));
  }
  const pause = {
    schema: "vize-corsa-bootstrap-pause-v1",
    candidate: value,
    missing: crates.filter((_name, index) => states[index] === "missing"),
    result: "paused-before-new-R",
  };
  const output = process.env.VIZE_RELEASE_CORSA_BOOTSTRAP_PAUSE;
  if (output) writeFileSync(output, `${JSON.stringify(pause, null, 2)}\n`);
  console.error(JSON.stringify(pause));
  throw new Error(
    "Corsa first publication is paused before NEW-R. Preserve H/C and the original operator/Q evidence; review and authorize its protected bootstrap carrier, then resume this same source. No Release was dispatched.",
  );
}

/** Prepare an ordinary reviewed source change, only after real official success. */
async function prepareCheckpoint(args: string[], root: string, deadline: number) {
  const [receiptPath, mainCommit, releaseRun, output, ...extra] = args;
  assert.equal(extra.length, 0);
  assert.ok(receiptPath && output);
  const raw = record(JSON.parse(readFileSync(receiptPath, "utf8")));
  const original = candidate(record(raw.candidate) as Candidate);
  const io = commands(root, deadline);
  const policy = Object.fromEntries(
    policyFiles.map((file) => [
      file,
      sha256(io.run("git", ["--no-replace-objects", "show", `${original.head}:${file}`])),
    ]),
  );
  const packet = {
    schema: "vize-corsa-completed-checkpoint-v1",
    receipt: raw,
    mainCommit,
    releaseRun: Number(releaseRun),
    policy,
  };
  const receipt = readCheckpoint(
    { ...original, cut: mainCommit },
    {
      ...io,
      exists: (file) => file === checkpointPath || io.exists(file),
      read: (file) => (file === checkpointPath ? JSON.stringify(packet) : io.read(file)),
    },
  );
  assert.ok(receipt);
  const request = boundedRequest(deadline);
  await canonicalOwner(receipt.owner, receipt.ownerLogin, request);
  for (const name of crates) await canonicalOwner(receipt.owner, receipt.ownerLogin, request, name);
  writeFileSync(output, `${JSON.stringify(packet, null, 2)}\n`, { flag: "wx" });
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const deadline = operatorDeadline(process.env);
  const execute = async () => {
    if (args[0] === "--prepare-checkpoint")
      return prepareCheckpoint(args.slice(1), process.cwd(), deadline);
    const [repository, number, branch, head, cut, tag, ...extra] = args;
    assert.equal(extra.length, 0);
    await preRelease(
      candidate({ repository, number: Number(number), branch, head, cut, tag }),
      process.cwd(),
      deadline,
    );
  };
  await execute().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  });
}
