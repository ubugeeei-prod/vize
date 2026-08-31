import assert from "node:assert/strict";
import { test } from "node:test";

import { resolveSemverChangeMarker } from "../../legacy-tools/github/semver-change-marker.mjs";
import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

test("push SemVer checks preserve pull-request markers after squash merge", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const job = workflowJobBody(workflow, "semver-checks");

  assert.match(job, /permissions:\n(?:[^\S\n]+\S.*\n)*[^\S\n]+contents:\s*read\n/);
  assert.match(job, /permissions:\n(?:[^\S\n]+\S.*\n)*[^\S\n]+pull-requests:\s*read\n/);
  assert.match(job, /- name:\s*Resolve SemVer change marker/);
  assert.match(
    job,
    /GITHUB_TOKEN:\s*\$\{\{\s*github\.event_name == 'push' && github\.token \|\| ''\s*\}\}/,
  );
  assert.match(
    job,
    /rust-script tools\/commands\/ci\/github\/semver-change-marker\.rs "\$RUNNER_TEMP\/semver-change-marker\.txt"/,
  );
  assert.match(job, /SEMVER_CHANGE_MARKER="\$\(cat "\$RUNNER_TEMP\/semver-change-marker\.txt"\)"/);
  assert.doesNotMatch(job, /git log -1 --format=%B/);
});

test("push marker uses the exact squash-merged pull request body", async () => {
  const sha = "f49c94e03ad957b1f6f51276a328acb533c21343";
  const calls = [];
  const marker = await resolveSemverChangeMarker({
    eventName: "push",
    event: {
      after: sha,
      head_commit: { message: "fix(atelier): keep custom element metadata internal (#3720)" },
      repository: { full_name: "ubugeeei-prod/vize" },
    },
    repository: "ubugeeei-prod/vize",
    sha,
    token: "test-token",
    fetchImpl: async (url, init) => {
      calls.push({ url, init });
      return new Response(
        JSON.stringify([
          {
            title: "fix(ci): unrelated pull request",
            body: "BREAKING CHANGE: unrelated marker.",
            merge_commit_sha: "0000000000000000000000000000000000000000",
            merged_at: "2026-08-02T08:39:00Z",
          },
          {
            title: "fix(atelier): keep custom element metadata internal",
            body: "Compatibility note\n\nBREAKING CHANGE: remove an unreleased field.",
            merge_commit_sha: sha,
            merged_at: "2026-08-02T08:40:36Z",
          },
        ]),
        { status: 200 },
      );
    },
  });

  assert.equal(
    marker,
    "fix(atelier): keep custom element metadata internal\nCompatibility note\n\nBREAKING CHANGE: remove an unreleased field.",
  );
  assert.equal(
    calls[0].url,
    `https://api.github.com/repos/ubugeeei-prod/vize/commits/${sha}/pulls`,
  );
  assert.equal(calls[0].init.headers.Authorization, "Bearer test-token");
});

test("push marker falls back to direct-push commit messages", async () => {
  const marker = await resolveSemverChangeMarker({
    eventName: "push",
    event: {
      after: "1234567890abcdef",
      commits: [
        { message: "fix(ci): prepare a direct push" },
        { message: "fix(ci)!: apply a direct breaking push" },
      ],
      head_commit: { message: "fix(ci)!: apply a direct breaking push" },
      repository: { full_name: "ubugeeei-prod/vize" },
    },
    token: "test-token",
    fetchImpl: async () => new Response("[]", { status: 200 }),
  });

  assert.equal(marker, "fix(ci): prepare a direct push\nfix(ci)!: apply a direct breaking push");
});

test("push marker fails closed when a commit has multiple exact merged pull requests", async () => {
  const sha = "f49c94e03ad957b1f6f51276a328acb533c21343";
  await assert.rejects(
    resolveSemverChangeMarker({
      eventName: "push",
      event: {
        after: sha,
        head_commit: { message: "fix(ci): ambiguous squash" },
        repository: { full_name: "ubugeeei-prod/vize" },
      },
      token: "test-token",
      fetchImpl: async () =>
        new Response(
          JSON.stringify([
            { title: "fix(ci): first", merge_commit_sha: sha, merged_at: "2026-08-02T08:40:36Z" },
            { title: "fix(ci): second", merge_commit_sha: sha, merged_at: "2026-08-02T08:41:12Z" },
          ]),
          { status: 200 },
        ),
    }),
    /multiple exact merged pull requests/,
  );
});

test("push marker fails closed without repository, commit, or token metadata", async () => {
  const event = {
    head_commit: { message: "fix(ci): direct push" },
    repository: { full_name: "ubugeeei-prod/vize" },
  };
  const fetchImpl = async () => {
    throw new Error("incomplete push metadata must not reach the associated-pulls API");
  };

  await assert.rejects(
    resolveSemverChangeMarker({ eventName: "push", event, sha: "abc123", fetchImpl }),
    /GITHUB_REPOSITORY, GITHUB_SHA, and GITHUB_TOKEN are required for push events/,
  );
  await assert.rejects(
    resolveSemverChangeMarker({ eventName: "push", event, token: "test-token", fetchImpl }),
    /GITHUB_REPOSITORY, GITHUB_SHA, and GITHUB_TOKEN are required for push events/,
  );
  await assert.rejects(
    resolveSemverChangeMarker({
      eventName: "push",
      event: { head_commit: event.head_commit },
      sha: "abc123",
      token: "test-token",
      fetchImpl,
    }),
    /GITHUB_REPOSITORY, GITHUB_SHA, and GITHUB_TOKEN are required for push events/,
  );
});

test("branch-deletion pushes skip the associated-pulls request", async () => {
  const marker = await resolveSemverChangeMarker({
    eventName: "push",
    event: {
      after: "0000000000000000000000000000000000000000",
      commits: [],
      deleted: true,
      repository: { full_name: "ubugeeei-prod/vize" },
    },
    token: "test-token",
    fetchImpl: async () => {
      throw new Error("branch deletions must not call the associated-pulls API");
    },
  });

  assert.equal(marker, "");
});

test("associated-pulls requests retry network errors and transient server failures", async () => {
  let attempts = 0;
  const delays = [];
  const marker = await resolveSemverChangeMarker({
    eventName: "push",
    event: {
      after: "1234567890abcdef",
      commits: [{ message: "fix(ci): retry transient associated-pulls failures" }],
      repository: { full_name: "ubugeeei-prod/vize" },
    },
    token: "test-token",
    fetchImpl: async () => {
      attempts += 1;
      if (attempts === 1) throw new Error("temporary network failure");
      if (attempts === 2) return new Response("temporary server failure", { status: 503 });
      return new Response("[]", { status: 200 });
    },
    sleepImpl: async (delay) => {
      delays.push(delay);
    },
  });

  assert.equal(marker, "fix(ci): retry transient associated-pulls failures");
  assert.equal(attempts, 3);
  assert.deepEqual(delays, [100, 200]);
});

test("associated-pulls requests do not retry non-transient failures", async () => {
  let attempts = 0;

  await assert.rejects(
    resolveSemverChangeMarker({
      eventName: "push",
      event: {
        after: "1234567890abcdef",
        commits: [{ message: "fix(ci): no fallback" }],
        repository: { full_name: "ubugeeei-prod/vize" },
      },
      token: "test-token",
      fetchImpl: async () => {
        attempts += 1;
        return new Response("not found", { status: 404 });
      },
      sleepImpl: async () => {
        throw new Error("non-transient failures must not sleep");
      },
    }),
    /HTTP 404/,
  );

  assert.equal(attempts, 1);
});

test("pull-request marker uses event metadata without an API request", async () => {
  const marker = await resolveSemverChangeMarker({
    eventName: "pull_request",
    event: {
      pull_request: {
        title: "fix(relief): remove an unreleased field",
        body: "BREAKING CHANGE: remove the field before release.",
      },
    },
    fetchImpl: async () => {
      throw new Error("pull-request events must not call the associated-pulls API");
    },
  });

  assert.equal(
    marker,
    "fix(relief): remove an unreleased field\nBREAKING CHANGE: remove the field before release.",
  );
});
