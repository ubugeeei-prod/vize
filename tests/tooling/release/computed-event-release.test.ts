import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  baseline,
  integrationPaths,
  releaseEvidence,
  requiredMerges,
  verifyComputedEventRelease,
  writeComputedEventRelease,
  type ReleaseRequest,
} from "../../../tools/support/release/computed_event_release.ts";
import { CRATE_INVENTORY_PATH } from "../../../tools/support/release/release/quota/inventory.ts";
import { mockProducerResponse } from "./quota-fixtures.ts";

const repository = "ubugeeei-prod/vize",
  originalHead = "f".repeat(40),
  integration = "e".repeat(40);
function gitFile(source: string, filename: string) {
  const bytes = Buffer.from(source);
  return {
    path: filename,
    type: "file",
    encoding: "base64",
    content: bytes.toString("base64"),
    sha: createHash("sha1")
      .update("blob " + bytes.length + "\0")
      .update(bytes)
      .digest("hex"),
  };
}
function comparison(base: string) {
  return {
    base_commit: { sha: base },
    merge_base_commit: { sha: base },
    behind_by: 0,
    status: "ahead",
  };
}
function fixture(before = false) {
  const request: ReleaseRequest = before
    ? { version: baseline.version, releaseRun: baseline.releaseRun, phase: "before" }
    : { version: "0.609.0", releaseRun: "42", phase: "after" };
  const candidate = before ? baseline.candidate : "c".repeat(40);
  const source = before ? baseline.source : "d".repeat(40),
    tag = "v" + request.version;
  const manifest = gitFile(
    JSON.stringify({ name: "@vizejs/native", version: request.version }),
    "npm/native/package.json",
  );
  const inventory = gitFile(
    'export const publishedCrates: readonly string[] = ["alpha"];',
    CRATE_INVENTORY_PATH,
  );
  const owner = {
    version: request.version,
    runId: request.releaseRun,
    runAttempt: 1,
    workflowHead: source,
    candidateSha: candidate,
    producerSource: source,
    nativeManifestGitBlob: manifest.sha,
    inventoryGitBlob: inventory.sha,
  };
  const records = new Map<string, unknown>([
    [
      `releases/tags/${tag}`,
      {
        tag_name: tag,
        draft: false,
        prerelease: false,
        published_at: "2026-10-30T00:30:00Z",
        retained: "whole-publication",
      },
    ],
    [
      `git/ref/tags/${tag}`,
      { ref: `refs/tags/${tag}`, object: { type: "commit", sha: candidate } },
    ],
    [
      `actions/runs/${request.releaseRun}`,
      {
        id: Number(request.releaseRun),
        run_attempt: 1,
        repository: { full_name: repository },
        path: ".github/workflows/release.yml",
        event: "workflow_dispatch",
        head_branch: "main",
        head_sha: source,
        status: "completed",
        conclusion: "success",
        created_at: "2026-10-30T00:00:00Z",
      },
    ],
    [`git/commits/${candidate}`, { sha: candidate, parents: [{ sha: source }] }],
    [`contents/npm/native/package.json?ref=${candidate}`, manifest],
    [`contents/${CRATE_INVENTORY_PATH}?ref=${candidate}`, inventory],
    [`compare/${source}...${source}`, { status: "identical", base_commit: { sha: source } }],
    [
      "pulls/11142",
      {
        number: 11142,
        merged: true,
        state: "closed",
        merged_at: "2026-10-29T23:00:00Z",
        merge_commit_sha: integration,
        base: { ref: "main", repo: { full_name: repository } },
        head: { ref: "chore/format-main", sha: originalHead, repo: { full_name: repository } },
      },
    ],
    [`compare/${integration}...${source}`, comparison(integration)],
  ]);
  for (const required of requiredMerges)
    records.set(`compare/${required}...${originalHead}`, comparison(required));
  for (const file of integrationPaths) {
    const blob = gitFile("original integration content: " + file, file);
    records.set(`contents/${file}?ref=${originalHead}`, blob);
    records.set(`contents/${file}?ref=${integration}`, structuredClone(blob));
  }
  const calls: string[] = [];
  const github = async (endpoint: string) => {
    calls.push(endpoint);
    assert.ok(records.has(endpoint), "unplanned API read: " + endpoint);
    return structuredClone(records.get(endpoint));
  };
  const fetchImpl = (async (input: string | URL | Request, init?: RequestInit) => {
    assert.ok(!init?.method || init.method === "GET", "attestation must never write remotely");
    const response = mockProducerResponse(owner, input, init);
    assert.ok(response, "only original producer artifact reads are allowed");
    return response;
  }) as typeof fetch;
  const record = (endpoint: string) => records.get(endpoint) as Record<string, unknown>;
  return { request, candidate, source, tag, owner, records, calls, github, fetchImpl, record };
}

test("after binds the genuine producer to the main squash and all original integration blobs", async () => {
  const f = fixture();
  const result = await verifyComputedEventRelease(f.request, f.github, f.fetchImpl);
  assert.equal(result.verified, true);
  const proof = result.proof as {
    ownership: { produced: { artifactId: number } };
    integration: { blobs: unknown[] };
  };
  assert.equal(
    proof.ownership.produced.artifactId,
    7,
    "real ownership reader authenticated the inert artifact",
  );
  assert.equal(proof.integration.blobs.length, 13);
  assert.deepEqual(
    result.observations.find((x) => x.endpoint === `releases/tags/${f.tag}`)?.value,
    f.record(`releases/tags/${f.tag}`),
  );
  for (const required of requiredMerges) {
    assert.ok(f.calls.includes(`compare/${required}...${originalHead}`));
    assert.ok(
      !f.calls.includes(`compare/${required}...${f.source}`),
      "squash does not invent direct ancestry",
    );
  }
});

test("before is the frozen public 0.608 release with its genuine original producer", async () => {
  const f = fixture(true);
  const result = await verifyComputedEventRelease(f.request, f.github, f.fetchImpl);
  assert.equal(result.verified, true);
  assert.equal((result.proof as { integration: unknown }).integration, null);
  assert.ok(!f.calls.includes("pulls/11142"));
});

test("qualified source and newer main workflow tooling retain separate authentic identities", async () => {
  const f = fixture();
  const workflowHead = "a".repeat(40);
  f.record("actions/runs/42").head_sha = workflowHead;
  f.owner.workflowHead = workflowHead;
  f.records.set(`compare/${f.source}...${workflowHead}`, comparison(f.source));
  const result = await verifyComputedEventRelease(f.request, f.github, f.fetchImpl);
  assert.equal(result.verified, true);
  const ownership = (
    result.proof as { ownership: { workflowHead: string; producerSource: string } }
  ).ownership;
  assert.equal(ownership.workflowHead, workflowHead);
  assert.equal(ownership.producerSource, f.source);

  f.record(`compare/${f.source}...${workflowHead}`).status = "diverged";
  await assert.rejects(verifyComputedEventRelease(f.request, f.github, f.fetchImpl));
});

test("malformed, historical and substituted before requests fail before any API read", async () => {
  for (const request of [
    { version: "0.608.0", releaseRun: "42", phase: "before" },
    { version: "0.607.0", releaseRun: baseline.releaseRun, phase: "before" },
    { version: "0.608.0", releaseRun: "42", phase: "after" },
    { version: "0.609.0/x", releaseRun: "42", phase: "after" },
    { version: "0.609.0", releaseRun: "42/foo", phase: "after" },
    { version: "0.609.0", releaseRun: "42", phase: "other" },
  ]) {
    const f = fixture();
    await assert.rejects(verifyComputedEventRelease(request, f.github, f.fetchImpl));
    assert.deepEqual(f.calls, []);
  }
});

test("unpublished, foreign, failed, incomplete or mismatched release facts refuse", async () => {
  const mutations: Array<[string, (f: ReturnType<typeof fixture>) => void]> = [
    [
      "draft",
      (f) => {
        f.record(`releases/tags/${f.tag}`).draft = true;
      },
    ],
    [
      "prerelease",
      (f) => {
        f.record(`releases/tags/${f.tag}`).prerelease = true;
      },
    ],
    [
      "missing publication",
      (f) => {
        f.record(`releases/tags/${f.tag}`).published_at = null;
      },
    ],
    [
      "wrong publication tag",
      (f) => {
        f.record(`releases/tags/${f.tag}`).tag_name = "v0.607.0";
      },
    ],
    [
      "wrong tag ref",
      (f) => {
        f.record(`git/ref/tags/${f.tag}`).ref = "refs/tags/v0.607.0";
      },
    ],
    [
      "annotated tag",
      (f) => {
        (f.record(`git/ref/tags/${f.tag}`).object as Record<string, unknown>).type = "tag";
      },
    ],
    [
      "foreign repository",
      (f) => {
        f.record("actions/runs/42").repository = { full_name: "foreign/vize" };
      },
    ],
    [
      "wrong run",
      (f) => {
        f.record("actions/runs/42").id = 43;
      },
    ],
    [
      "wrong workflow",
      (f) => {
        f.record("actions/runs/42").path = ".github/workflows/check.yml";
      },
    ],
    [
      "wrong event",
      (f) => {
        f.record("actions/runs/42").event = "push";
      },
    ],
    [
      "feature branch",
      (f) => {
        f.record("actions/runs/42").head_branch = "feature";
      },
    ],
    [
      "incomplete",
      (f) => {
        f.record("actions/runs/42").status = "in_progress";
      },
    ],
    [
      "failed",
      (f) => {
        f.record("actions/runs/42").conclusion = "failure";
      },
    ],
    [
      "unrelated workflow head",
      (f) => {
        f.record("actions/runs/42").head_sha = "a".repeat(40);
      },
    ],
    [
      "publication before run",
      (f) => {
        f.record("actions/runs/42").created_at = "2027-01-01T00:00:00Z";
      },
    ],
    [
      "multiple candidate parents",
      (f) => {
        f.record(`git/commits/${f.candidate}`).parents = [{ sha: f.source }, { sha: originalHead }];
      },
    ],
    [
      "different committed version",
      (f) => {
        f.records.set(
          `contents/npm/native/package.json?ref=${f.candidate}`,
          gitFile('{"name":"@vizejs/native","version":"0.608.0"}', "npm/native/package.json"),
        );
      },
    ],
    [
      "foreign original producer",
      (f) => {
        f.owner.candidateSha = "9".repeat(40);
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const f = fixture();
    mutate(f);
    await assert.rejects(
      verifyComputedEventRelease(f.request, f.github, f.fetchImpl),
      undefined,
      name,
    );
  }
});

test("unmerged integration, missing fix ancestry, foreign paths and altered squash blobs refuse", async () => {
  const mutations: Array<[string, (f: ReturnType<typeof fixture>) => void]> = [
    [
      "unmerged",
      (f) => {
        f.record("pulls/11142").merged = false;
      },
    ],
    [
      "wrong target",
      (f) => {
        (f.record("pulls/11142").base as Record<string, unknown>).ref = "feature";
      },
    ],
    [
      "foreign original branch",
      (f) => {
        (f.record("pulls/11142").head as Record<string, unknown>).repo = {
          full_name: "foreign/vize",
        };
      },
    ],
    [
      "fix absent",
      (f) => {
        f.record(`compare/${requiredMerges[0]}...${originalHead}`).merge_base_commit = {
          sha: "0".repeat(40),
        };
      },
    ],
    [
      "second fix absent",
      (f) => {
        f.record(`compare/${requiredMerges[1]}...${originalHead}`).behind_by = 1;
      },
    ],
    [
      "squash absent from release",
      (f) => {
        f.record(`compare/${integration}...${f.source}`).status = "diverged";
      },
    ],
    [
      "wrong compare base",
      (f) => {
        f.record(`compare/${integration}...${f.source}`).base_commit = { sha: originalHead };
      },
    ],
    [
      "altered production blob",
      (f) => {
        f.record(`contents/${integrationPaths[3]}?ref=${integration}`).sha = "1".repeat(40);
      },
    ],
    [
      "altered fixture blob",
      (f) => {
        f.record(`contents/${integrationPaths[10]}?ref=${integration}`).sha = "2".repeat(40);
      },
    ],
    [
      "symlink",
      (f) => {
        f.record(`contents/${integrationPaths[0]}?ref=${integration}`).type = "symlink";
      },
    ],
    [
      "foreign path",
      (f) => {
        f.record(`contents/${integrationPaths[0]}?ref=${integration}`).path = "unrelated.rs";
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const f = fixture();
    mutate(f);
    await assert.rejects(
      verifyComputedEventRelease(f.request, f.github, f.fetchImpl),
      undefined,
      name,
    );
  }
});

test("refusal evidence retains actual observations and an existing output is never overwritten", async () => {
  const directory = mkdtempSync(path.join(os.tmpdir(), "computed-event-proof-"));
  try {
    const output = path.join(directory, "release.json"),
      f = fixture();
    f.record("actions/runs/42").conclusion = "failure";
    await assert.rejects(
      writeComputedEventRelease(f.request, output, f.github, f.fetchImpl),
      /Release must succeed/,
    );
    const saved = JSON.parse(readFileSync(output, "utf8"));
    assert.equal(saved.verified, false);
    assert.match(saved.error, /Release must succeed/);
    assert.equal(saved.observations.at(-1).value.conclusion, "failure");
    const bytes = readFileSync(output);
    await assert.rejects(writeComputedEventRelease(f.request, output, f.github, f.fetchImpl), {
      code: "EEXIST",
    });
    assert.deepEqual(readFileSync(output), bytes);
    const malformed = { ...f.request, phase: "unknown" };
    await assert.rejects(
      writeComputedEventRelease(
        malformed,
        path.join(directory, "early.json"),
        f.github,
        f.fetchImpl,
      ),
    );
    assert.deepEqual(
      JSON.parse(readFileSync(path.join(directory, "early.json"), "utf8")).observations,
      [],
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("an API refusal leaves verified false without inventing a successful packet", async () => {
  const f = fixture(),
    evidence = releaseEvidence(f.request);
  await assert.rejects(
    verifyComputedEventRelease(
      f.request,
      async () => {
        throw new Error("HTTP 403");
      },
      f.fetchImpl,
      evidence,
    ),
    /HTTP 403/,
  );
  assert.equal(evidence.verified, false);
  assert.deepEqual(evidence.observations, []);
});
