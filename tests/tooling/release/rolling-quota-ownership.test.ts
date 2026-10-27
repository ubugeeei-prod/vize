import assert from "node:assert/strict";
import { test } from "node:test";
import { releaseRunning } from "../../../tools/support/release/release/cadence.ts";
import { readReleaseOwnership } from "../../../tools/support/release/release/quota_ownership.ts";

const candidate = "c".repeat(40),
  producer = "d".repeat(40),
  workflowHead = "b".repeat(40);
function facts() {
  return {
    run: {
      id: 42,
      repository: { full_name: "ubugeeei-prod/vize" },
      path: ".github/workflows/release.yml",
      event: "workflow_dispatch",
      head_branch: "main",
      head_sha: workflowHead,
    },
    ref: { object: { type: "commit", sha: candidate } },
    commit: { sha: candidate, parents: [{ sha: producer }] },
    manifest: {
      encoding: "base64",
      content: Buffer.from(JSON.stringify({ name: "@vizejs/native", version: "0.602.0" })).toString(
        "base64",
      ),
    },
    compare: { status: "ahead", base_commit: { sha: producer } },
  };
}
function reader(value: ReturnType<typeof facts>) {
  return async (path: string): Promise<unknown> => {
    if (path.startsWith("actions/runs/")) return value.run;
    if (path.startsWith("git/ref/")) return value.ref;
    if (path.startsWith("git/commits/")) return value.commit;
    if (path.startsWith("contents/")) return value.manifest;
    if (path.startsWith("compare/")) return value.compare;
    throw new Error("unexpected GitHub query");
  };
}
test("original workflow head, requested producer and prepared tag remain separate identities", async () => {
  const result = await readReleaseOwnership("0.602.0", candidate, "42", reader(facts()));
  assert.deepEqual(result, {
    version: "0.602.0",
    runId: "42",
    workflowHead,
    candidateSha: candidate,
    producerSource: producer,
  });
});
test("wrong workflow, repo, branch, run, tag and producer ancestry are refused", async () => {
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
      f.manifest.content = Buffer.from('{"name":"@vizejs/native","version":"0.601.0"}').toString(
        "base64",
      );
    },
  ];
  for (const mutate of mutations) {
    const value = facts();
    mutate(value);
    await assert.rejects(readReleaseOwnership("0.602.0", candidate, "42", reader(value)));
  }
});
test("missing or malformed authority and unavailable GitHub state never admit recovery", async () => {
  await assert.rejects(readReleaseOwnership("0.602.0", "bad", "42", reader(facts())));
  await assert.rejects(readReleaseOwnership("0.602.0", candidate, "not-run", reader(facts())));
  await assert.rejects(
    readReleaseOwnership("0.602.0", candidate, "42", async () => {
      throw new Error("HTTP 429");
    }),
  );
});

test("all active and pending publisher states serialize Release and JSR", async () => {
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
    seen.some((path) => path.includes("release-jsr.yml") && path.includes("status=requested")),
    true,
  );
  assert.equal(
    await releaseRunning(async (path) => ({
      total_count: path.includes("release-jsr.yml") ? 1 : 0,
    })),
    true,
  );
});
test("unknown active publisher counts fail closed", async () => {
  for (const count of [undefined, "0", -1, NaN]) {
    await assert.rejects(releaseRunning(async () => ({ total_count: count })));
  }
});
