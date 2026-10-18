import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { spawnSync } from "node:child_process";
import { isEntrypoint } from "../../command-runtime.ts";
import { fullCheckProfile } from "../../compat/github/full-check-policy.ts";
import { assertRequiredWorkflowJobs } from "../../compat/github/release-preflight-evidence.mjs";
import { assertRun, type Row } from "./check-state.ts";
import { createIntegrationReadApi } from "./integration-http.ts";
import {
  qualifyIntegrationRelease,
  releaseJobDigest,
  type IntegrationReleaseOptions,
} from "./integration-release.ts";

export type IntegrationProductReceipt = {
  kind: "vize.release.integration-product-v1";
  qualificationSha256: string;
  profile: "full-qualification-v2";
  checkId: number;
  checkAttempt: number;
  head: string;
  candidate: string;
  headTree: string;
  candidateTree: string;
};
export type IntegrationProductOptions = Omit<IntegrationReleaseOptions, "mode"> & {
  mode: "qualify" | "verify";
  readCommit?: (sha: string) => string | Uint8Array | Promise<string | Uint8Array>;
};
export type IntegrationProducts = { reuse: boolean; product: IntegrationProductReceipt | null };

const sha = (value: unknown): string => {
  assert.ok(typeof value === "string" && /^[a-f0-9]{40}$/u.test(value), "full Git SHA required");
  return value;
};
const object = (value: unknown): Row => {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Row;
};
function readCommit(id: string, head: string, repository: string): Buffer {
  const inventory = spawnSync("git", ["--no-replace-objects", "cat-file", "--batch-check"], {
    input: `${sha(id)}\n`,
    encoding: "utf8",
    maxBuffer: 4096,
  });
  assert.ifError(inventory.error);
  assert.equal(inventory.status, 0, "raw Git object inventory failed");
  if (inventory.stdout === `${id} missing\n`) {
    assert.equal(id, head, "candidate must already exist locally");
    const origin = spawnSync("git", ["config", "--get", "remote.origin.url"], {
      encoding: "utf8",
      maxBuffer: 4096,
    });
    assert.ifError(origin.error);
    assert.equal(origin.status, 0);
    assert.ok(
      [
        `https://github.com/${repository}`,
        `https://github.com/${repository}.git`,
        `git@github.com:${repository}.git`,
        `ssh://git@github.com/${repository}.git`,
      ].includes(origin.stdout.trim()),
      "authenticated repository Git origin required",
    );
    const fetched = spawnSync(
      "git",
      [
        "--no-replace-objects",
        "fetch",
        "--no-tags",
        "--no-write-fetch-head",
        "--depth=1",
        "origin",
        id,
      ],
      { maxBuffer: 1024 * 1024 },
    );
    assert.ifError(fetched.error);
    assert.equal(fetched.status, 0, "authenticated H object unavailable");
  } else {
    assert.match(
      inventory.stdout,
      new RegExp(`^${id} commit [0-9]+\\n$`, "u"),
      "raw commit required",
    );
  }
  const result = spawnSync("git", ["--no-replace-objects", "cat-file", "commit", id], {
    maxBuffer: 4 * 1024 * 1024,
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, "raw Git commit is unavailable");
  return result.stdout;
}
function commitIdentity(id: string, value: string | Uint8Array, parent: string) {
  const bytes = typeof value === "string" ? Buffer.from(value) : Buffer.from(value);
  assert.equal(
    createHash("sha1").update(`commit ${bytes.length}\0`).update(bytes).digest("hex"),
    id,
    "raw Git commit object identity changed",
  );
  const separator = bytes.indexOf("\n\n");
  assert.ok(separator >= 0, "raw Git commit header terminator required");
  const headers = bytes.subarray(0, separator).toString("utf8").split("\n");
  const trees = headers
    .filter((line) => line.startsWith("tree "))
    .map((line) => sha(line.slice(5)));
  const parents = headers
    .filter((line) => line.startsWith("parent "))
    .map((line) => sha(line.slice(7)));
  assert.equal(trees.length, 1, "exactly one raw tree required");
  assert.deepEqual(parents, [parent], "raw Git parent vector changed");
  return trees[0];
}

/** Adjacent proof only: the original Q/R/pin/lease verifier remains mandatory. */
export async function qualifyIntegrationProducts(
  input: unknown,
  options: IntegrationProductOptions,
): Promise<IntegrationProducts> {
  assert.ok(options.mode === "qualify" || options.mode === "verify");
  const packet = object(input);
  assert.deepEqual(
    Object.keys(packet).sort(),
    options.mode === "verify" ? ["product", "qualification"] : ["qualification"],
    "exact product qualification input required",
  );
  const verifyOriginal = () =>
    qualifyIntegrationRelease(packet.qualification, { ...options, mode: "verify" });
  const qualification = await verifyOriginal();
  const q = qualification.qualification;
  const context = q.context;
  const run = await options.api.read(`actions/runs/${q.checkId}`);
  assertRun(run);
  for (const [key, value] of Object.entries({
    id: q.checkId,
    run_attempt: q.checkAttempt,
    head_sha: context.H,
    head_branch: `release/${context.tag}`,
    path: ".github/workflows/check.yml",
    event: "workflow_dispatch",
    display_title: `Check full @ ${context.H}`,
    created_at: q.checkCreatedAt,
    status: "completed",
    conclusion: "success",
  }))
    assert.equal(run[key], value, `current Q ${key} changed`);
  for (const field of ["repository", "head_repository"])
    assert.equal(object(run[field]).full_name, q.repository);
  const profile = fullCheckProfile(run);
  const jobs = await options.api.pages(`actions/runs/${q.checkId}/jobs`, "jobs", {
    filter: "latest",
  });
  assertRequiredWorkflowJobs("Check", jobs, run);
  const read = options.readCommit ?? ((id) => readCommit(id, context.H, q.repository));
  const rawHead = await read(context.H);
  const rawCandidate = await read(context.candidate);
  const headTree = commitIdentity(context.H, rawHead, context.C);
  const candidateTree = commitIdentity(context.candidate, rawCandidate, context.base);
  for (const [id, value] of [
    [context.H, rawHead],
    [context.candidate, rawCandidate],
  ] as const)
    options.observe?.({
      kind: "raw-git-commit",
      sha: id,
      base64:
        typeof value === "string"
          ? Buffer.from(value).toString("base64")
          : Buffer.from(value).toString("base64"),
    });
  // Raw headers authenticate trees independently of the GitHub/traversal view.
  for (const [id, tree, parent] of [
    [context.H, headTree, context.C],
    [context.candidate, candidateTree, context.base],
  ]) {
    const commit = await options.api.read(`git/commits/${id}`);
    assert.equal(commit.sha, id);
    assert.equal(object(commit.tree).sha, tree, "raw/API Git tree changed");
    assert.ok(Array.isArray(commit.parents));
    assert.deepEqual(
      commit.parents.map((value) => object(value).sha),
      [parent],
    );
  }
  await verifyOriginal();
  assert.deepEqual(
    await options.api.read(`actions/runs/${q.checkId}`),
    run,
    "whole current Q metadata changed",
  );
  const product: IntegrationProductReceipt | null =
    profile === "full-qualification-v2" && headTree === candidateTree
      ? {
          kind: "vize.release.integration-product-v1",
          qualificationSha256: releaseJobDigest(qualification),
          profile,
          checkId: q.checkId,
          checkAttempt: q.checkAttempt,
          head: context.H,
          candidate: context.candidate,
          headTree,
          candidateTree,
        }
      : null;
  if (options.mode === "verify") {
    assert.ok(product !== null, "product reuse authority required for verification");
    assert.deepEqual(packet.product, product, "product qualification changed");
  }
  return { reuse: product !== null, product };
}

export function integrationProductOptionsFromEnvironment(
  mode: IntegrationProductOptions["mode"],
): IntegrationProductOptions {
  assert.equal(process.env.GITHUB_EVENT_NAME, "merge_group");
  const repository = process.env.GITHUB_REPOSITORY;
  const candidateSha = process.env.GITHUB_SHA;
  const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN;
  assert.ok(repository && candidateSha && token, "current read-only GitHub authority required");
  const transport = createIntegrationReadApi({
    repository,
    token,
    apiUrl: process.env.GITHUB_API_URL,
  });
  const evidence = process.env.VIZE_RELEASE_INTEGRATION_EVIDENCE;
  return {
    repository,
    candidateSha,
    api: transport.api,
    mode,
    log: console.error,
    onPendingDeadline: transport.setPendingDeadline,
    observe: evidence
      ? (snapshot) => fs.appendFileSync(evidence, `${JSON.stringify(snapshot)}\n`)
      : undefined,
  };
}

if (isEntrypoint(import.meta.url)) {
  const [mode, ...extra] = process.argv.slice(2);
  assert.ok((mode === "--qualify" || mode === "--verify") && extra.length === 0);
  const bytes = fs.readFileSync(0);
  assert.ok(bytes.length <= 24_576, "bounded adjacent qualification input required");
  const result = await qualifyIntegrationProducts(
    JSON.parse(bytes.toString("utf8")),
    integrationProductOptionsFromEnvironment(mode === "--qualify" ? "qualify" : "verify"),
  );
  process.stdout.write(`${JSON.stringify(result.product)}\n`);
}
