import assert from "node:assert/strict";
import { test } from "node:test";

import {
  githubApiPages,
  githubApiRequest,
} from "../../legacy-tools/github/release-preflight-github.mjs";

const requestBase = {
  apiUrl: "https://api.github.test",
  repository: "owner/repository",
  token: "secret",
  resource: "actions/runs",
};

test("GitHub API GETs retry transient responses but mutations do not", async () => {
  let calls = 0;
  const waits: number[] = [];
  const response = await githubApiRequest({
    ...requestBase,
    fetchImpl: async () => {
      calls += 1;
      return calls === 1
        ? new Response("temporarily unavailable", { status: 503 })
        : new Response("{}", { status: 200 });
    },
    sleep: async (milliseconds) => waits.push(milliseconds),
  });
  assert.equal(response.body, "{}");
  assert.equal(calls, 2);
  assert.deepEqual(waits, [1_000]);

  calls = 0;
  await assert.rejects(
    githubApiRequest({
      ...requestBase,
      method: "POST",
      fetchImpl: async () => {
        calls += 1;
        return new Response("temporarily unavailable", { status: 503 });
      },
    }),
    /503/,
  );
  assert.equal(calls, 1);
});

test("GitHub API request timeout covers both headers and response body", async () => {
  await assert.rejects(
    githubApiRequest({
      ...requestBase,
      method: "POST",
      requestTimeoutMs: 5,
      fetchImpl: async (_url, init) =>
        new Promise((_resolve, reject) => {
          init?.signal?.addEventListener("abort", () => reject(new Error("aborted")), {
            once: true,
          });
        }),
    }),
    /timed out after 5ms/,
  );
});

test("GitHub API pagination keeps exact query filters and reads every page", async () => {
  const urls: string[] = [];
  const values = await githubApiPages({
    ...requestBase,
    collection: "workflow_runs",
    query: { head_sha: "a".repeat(40) },
    fetchImpl: async (url) => {
      urls.push(String(url));
      const page = new URL(String(url)).searchParams.get("page");
      const workflowRuns =
        page === "1" ? Array.from({ length: 100 }, (_, id) => ({ id })) : [{ id: 100 }];
      return new Response(JSON.stringify({ workflow_runs: workflowRuns }), { status: 200 });
    },
  });

  assert.equal(values.length, 101);
  assert.equal(urls.length, 2);
  for (const url of urls) {
    assert.equal(new URL(url).searchParams.get("head_sha"), "a".repeat(40));
    assert.equal(new URL(url).searchParams.get("per_page"), "100");
  }
});
