import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  appendFileSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { githubApiPages, githubApiRequest } from "./release-preflight-github.mjs";
import { selectRustWorkers, verifyRustWorkerDirectories } from "./rust-worker-selection.mjs";

export async function readRustWorkerSelection(context, options) {
  const resource = `actions/runs/${context.runId}`;
  const readRun = async () => JSON.parse((await githubApiRequest({ ...options, resource })).body);
  const before = await readRun();
  const [jobs, artifacts] = await Promise.all([
    githubApiPages({
      ...options,
      resource: `${resource}/jobs`,
      collection: "jobs",
      query: { filter: "all" },
    }),
    githubApiPages({ ...options, resource: `${resource}/artifacts`, collection: "artifacts" }),
  ]);
  const first = selectRustWorkers(context, before, jobs, artifacts);
  const after = selectRustWorkers(context, await readRun(), jobs, artifacts);
  assert.deepEqual(after, first, "Rust workflow identity changed during selection");
  return after;
}

export async function runRustWorkerSelection(argv, env = process.env, options = {}) {
  const [mode, receiptPath, artifactRoot] = argv;
  assert(
    ["select", "verify"].includes(mode) && receiptPath,
    "Expected select|verify RECEIPT [ARTIFACT_ROOT]",
  );
  for (const name of ["report.json", "acceptance.json"]) {
    rmSync(join(dirname(receiptPath), name), { force: true });
  }
  if (mode === "select") rmSync(receiptPath, { force: true });
  assert.equal(env.GITHUB_WORKFLOW, "Check", "Unexpected Rust workflow context");
  assert.equal(env.GITHUB_EVENT_NAME, "merge_group", "Unexpected Rust event context");
  const context = {
    runId: Number(env.GITHUB_RUN_ID),
    attempt: Number(env.GITHUB_RUN_ATTEMPT),
    sha: env.GITHUB_SHA,
    repository: env.GITHUB_REPOSITORY,
  };
  const head = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  assert.equal(head, context.sha, "Rust report checkout differs from the workflow source");
  assert(env.GH_TOKEN, "Rust selection requires its read-only Actions token");
  const selection = await readRustWorkerSelection(context, {
    apiUrl: env.GITHUB_API_URL,
    repository: context.repository,
    token: env.GH_TOKEN,
    ...options,
  });
  if (mode === "verify") {
    assert(artifactRoot, "Missing Rust artifact root");
    assert.deepEqual(
      selection,
      JSON.parse(readFileSync(receiptPath, "utf8")),
      "Rust workers changed after download",
    );
    verifyRustWorkerDirectories(selection, readdirSync(artifactRoot, { withFileTypes: true }));
  } else {
    assert(env.GITHUB_OUTPUT, "Missing GitHub step output");
    mkdirSync(dirname(receiptPath), { recursive: true });
    writeFileSync(receiptPath, `${JSON.stringify(selection, null, 2)}\n`);
    appendFileSync(
      env.GITHUB_OUTPUT,
      `artifact-ids=${selection.workers.map((worker) => worker.artifactId).join(",")}\n`,
    );
  }
  process.stdout.write(`${JSON.stringify(selection)}\n`);
  return selection;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await runRustWorkerSelection(process.argv.slice(2));
}
