import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import {
  readReleaseIdentity,
  readReleaseOwnership,
} from "../../../tools/support/release/release/quota_ownership.ts";
import {
  readProducedCandidate,
  MissingProducerAttestation,
} from "../../../tools/support/release/release/quota/producer.ts";
import { CRATE_INVENTORY_PATH } from "../../../tools/support/release/release/quota/inventory.ts";
import { releaseRunning } from "../../../tools/support/release/release/cadence.ts";
import { mockProducerResponse } from "./quota-fixtures.ts";

const candidate = "c".repeat(40),
  producer = "d".repeat(40),
  workflowHead = "b".repeat(40);
function gitFile(source: string, path: string) {
  const bytes = Buffer.from(source),
    sha = createHash("sha1")
      .update("blob " + bytes.length + "\0")
      .update(bytes)
      .digest("hex");
  return { path, type: "file", encoding: "base64", content: bytes.toString("base64"), sha };
}
function facts() {
  return {
    run: {
      id: 42,
      run_attempt: 1,
      repository: { full_name: "ubugeeei-prod/vize" },
      path: ".github/workflows/release.yml",
      event: "workflow_dispatch",
      head_branch: "main",
      head_sha: workflowHead,
    },
    ref: { object: { type: "commit", sha: candidate } },
    commit: { sha: candidate, parents: [{ sha: producer }] },
    manifest: gitFile(
      JSON.stringify({ name: "@vizejs/native", version: "0.602.0" }),
      "npm/native/package.json",
    ),
    inventory: gitFile(
      'export const publishedCrates: readonly string[] = ["alpha"];',
      CRATE_INVENTORY_PATH,
    ),
    compare: { status: "ahead", base_commit: { sha: producer } },
  };
}
function reader(value: ReturnType<typeof facts>) {
  return async (path: string): Promise<unknown> => {
    if (path.startsWith("actions/runs/")) return value.run;
    if (path.startsWith("git/ref/")) return value.ref;
    if (path.startsWith("git/commits/")) return value.commit;
    if (path.includes("contents/npm/native/")) return value.manifest;
    if (path.startsWith("contents/")) return value.inventory;
    if (path.startsWith("compare/")) return value.compare;
    throw new Error("unexpected GitHub query");
  };
}
function completeOwner(value = facts()) {
  return {
    version: "0.602.0",
    runId: "42",
    runAttempt: 1,
    workflowHead,
    candidateSha: candidate,
    producerSource: producer,
    nativeManifestGitBlob: value.manifest.sha,
    inventoryGitBlob: value.inventory.sha,
  };
}
test("workflow head, requested producer and prepared tag remain separate basic identities", async () => {
  const result = await readReleaseIdentity("0.602.0", candidate, "42", reader(facts()));
  assert.equal(result.candidateSha, candidate);
  assert.equal(result.producerSource, producer);
  assert.equal(result.workflowHead, workflowHead);
  assert.equal(result.nativeManifestGitBlob, facts().manifest.sha);
});
test("wrong workflow/repo/branch/run/tag/manifest and producer ancestry are refused", async () => {
  const mutations: Array<(f: ReturnType<typeof facts>) => void> = [
    (f) => {
      f.run.repository.full_name = "foreign/repo";
    },
    (f) => {
      f.run.path = ".github/workflows/check.yml";
    },
    (f) => {
      f.run.head_branch = "feature";
    },
    (f) => {
      f.run.id = 43;
    },
    (f) => {
      f.ref.object.sha = "e".repeat(40);
    },
    (f) => {
      f.ref.object.type = "tag";
    },
    (f) => {
      f.commit.parents = [];
    },
    (f) => {
      f.compare.status = "diverged";
    },
    (f) => {
      f.compare.base_commit.sha = "f".repeat(40);
    },
    (f) => {
      f.manifest = gitFile(
        '{"name":"@vizejs/native","version":"0.601.0"}',
        "npm/native/package.json",
      );
    },
  ];
  for (const mutate of mutations) {
    const value = facts();
    mutate(value);
    await assert.rejects(readReleaseIdentity("0.602.0", candidate, "42", reader(value)));
  }
});
test("same run/head/version with another ancestor S/C is refused by the genuine producer artifact", async () => {
  const real = facts(),
    value = facts(),
    owner = completeOwner(real);
  const wrongC = "9".repeat(40),
    wrongS = "8".repeat(40);
  value.ref.object.sha = wrongC;
  value.commit.sha = wrongC;
  value.commit.parents = [{ sha: wrongS }];
  value.compare.base_commit.sha = wrongS;
  assert.equal(
    (await readReleaseIdentity("0.602.0", wrongC, "42", reader(value))).candidateSha,
    wrongC,
  );
  const fetchImpl = (async (input: string | URL | Request, init?: RequestInit) => {
    const response = mockProducerResponse(owner, input, init);
    assert.ok(response);
    return response;
  }) as typeof fetch;
  await assert.rejects(
    readReleaseOwnership("0.602.0", wrongC, "42", reader(value), fetchImpl),
    /matching intended candidate/,
  );
});
test("actual producer artifact validates the whole ZIP and never forwards GitHub authorization", async () => {
  const owner = completeOwner(),
    saved = process.env.GITHUB_TOKEN;
  process.env.GITHUB_TOKEN = "inert-protocol-fixture-token";
  try {
    const fetchImpl = (async (input: string | URL | Request, init?: RequestInit) => {
      const response = mockProducerResponse(owner, input, init);
      assert.ok(response);
      return response;
    }) as typeof fetch;
    const result = await readReleaseOwnership(
      "0.602.0",
      candidate,
      "42",
      reader(facts()),
      fetchImpl,
    );
    assert.equal(result.produced.record.candidateSha, candidate);
    assert.equal(result.produced.artifactName, "release-producer-source-1");
    assert.match(result.produced.archiveSha256, /^[a-f0-9]{64}$/);
  } finally {
    if (saved === undefined) delete process.env.GITHUB_TOKEN;
    else process.env.GITHUB_TOKEN = saved;
  }
});
test("missing legacy artifact is an explicit refusal, not a version-only waiver", async () => {
  const result = readProducedCandidate(
    completeOwner(),
    (async () => new Response(JSON.stringify({ total_count: 0, artifacts: [] }))) as typeof fetch,
  );
  await assert.rejects(
    result,
    (error: unknown) =>
      error instanceof MissingProducerAttestation &&
      error.code === "VIZE_PRODUCER_ATTESTATION_UNAVAILABLE" &&
      error.witness.total === 0,
  );
});
test("all active/pending publisher states serialize Release and JSR; unknown counts refuse", async () => {
  const seen: string[] = [];
  assert.equal(
    await releaseRunning(async (path) => {
      seen.push(path);
      return { total_count: 0 };
    }),
    false,
  );
  assert.equal(seen.length, 10);
  assert.equal(
    await releaseRunning(async (path) => ({
      total_count: path.includes("release-jsr.yml") ? 1 : 0,
    })),
    true,
  );
  for (const count of [undefined, "0", -1, NaN])
    await assert.rejects(releaseRunning(async () => ({ total_count: count })));
});
