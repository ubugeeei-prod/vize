import assert from "node:assert/strict";
import { test } from "node:test";

import {
  githubApiPages,
  githubApiRequest,
} from "../../../tools/support/compat/github/release-preflight-github.mjs";

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

test("GitHub API GETs back off across a minute-scale outage before giving up", async () => {
  let calls = 0;
  const waits: number[] = [];
  const flaky = (failures: number) => async () => {
    calls += 1;
    return calls <= failures
      ? new Response("bad gateway", { status: 502 })
      : new Response("{}", { status: 200 });
  };
  const sleep = async (milliseconds: number) => {
    waits.push(milliseconds);
  };
  const response = await githubApiRequest({ ...requestBase, fetchImpl: flaky(5), sleep });
  assert.equal(response.body, "{}");
  assert.deepEqual(waits, [1_000, 2_000, 4_000, 8_000, 16_000]);

  calls = 0;
  await assert.rejects(githubApiRequest({ ...requestBase, fetchImpl: flaky(6), sleep }), /502/);
  assert.equal(calls, 6);
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
      const requested = new URL(url instanceof Request ? url.url : url);
      urls.push(requested.href);
      const page = requested.searchParams.get("page");
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
