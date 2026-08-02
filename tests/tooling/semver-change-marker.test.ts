import assert from "node:assert/strict";
import { test } from "node:test";

import {
  pullRequestMarkerText,
  releaseTypeForMarker,
  resolveSemverChangeMarker,
  selectSquashedPullRequest,
} from "../../tools/github/semver-change-marker.mjs";
import {
  breakingPullRequest,
  type MergedPullRequest,
  previousSha,
  squashCommitMessage,
  squashSha,
} from "./support/semver-change-marker.ts";

test("the squash simulation reproduces GitHub's squash commit message", () => {
  assert.equal(
    squashCommitMessage(breakingPullRequest, []),
    "fix(atelier): keep custom element metadata internal (#3720)\n",
  );
  assert.equal(
    squashCommitMessage(breakingPullRequest, ["Refs #3391"]),
    "fix(atelier): keep custom element metadata internal (#3720)\n\nRefs #3391\n",
  );
});

test("a squash merge destroys a marker that only lives in the commit message", async () => {
  const squashed = squashCommitMessage(breakingPullRequest, ["Refs #3391"]);

  assert.deepEqual(
    await resolveSemverChangeMarker({
      commitMessage: squashed,
      eventName: "push",
      sha: squashSha,
    }),
    {
      marker: "fix(atelier): keep custom element metadata internal (#3720)\n\nRefs #3391\n",
      releaseType: "none",
      source: "commit_message",
    },
  );
});

test("push SemVer checks recover the marker from the squashed pull request", async () => {
  const requestedShas: string[] = [];

  assert.deepEqual(
    await resolveSemverChangeMarker({
      commitMessage: squashCommitMessage(breakingPullRequest, ["Refs #3391"]),
      eventName: "push",
      listPullRequestsForCommit: async (sha: string) => {
        requestedShas.push(sha);
        return [breakingPullRequest];
      },
      sha: squashSha,
    }),
    {
      marker: `${breakingPullRequest.title}\n${breakingPullRequest.body}\n`,
      releaseType: "major",
      source: "squashed_pull_request",
    },
  );
  assert.deepEqual(requestedShas, [squashSha]);
});

test("direct pushes keep deciding from their own commit message", async () => {
  assert.deepEqual(
    await resolveSemverChangeMarker({
      commitMessage: "refactor(relief)!: drop the legacy node builder\n",
      eventName: "push",
      listPullRequestsForCommit: async () => [],
      sha: squashSha,
    }),
    {
      marker: "refactor(relief)!: drop the legacy node builder\n",
      releaseType: "major",
      source: "commit_message",
    },
  );

  assert.deepEqual(
    await resolveSemverChangeMarker({
      commitMessage: "fix(relief): keep the legacy node builder\n",
      eventName: "push",
      listPullRequestsForCommit: async () => [],
      sha: squashSha,
    }),
    {
      marker: "fix(relief): keep the legacy node builder\n",
      releaseType: "none",
      source: "commit_message",
    },
  );
});

test("pull-request events keep reading the event payload", async () => {
  assert.deepEqual(
    await resolveSemverChangeMarker({
      commitMessage: "",
      eventName: "pull_request",
      pullRequestBody: breakingPullRequest.body,
      pullRequestTitle: breakingPullRequest.title,
      sha: squashSha,
    }),
    {
      marker: `${breakingPullRequest.title}\n${breakingPullRequest.body}\n`,
      releaseType: "major",
      source: "pull_request_event",
    },
  );

  assert.deepEqual(
    await resolveSemverChangeMarker({
      commitMessage: "",
      eventName: "pull_request",
      pullRequestBody: "## Summary\n\n- keep the public field\n",
      pullRequestTitle: "fix(relief): keep the public field",
      sha: squashSha,
    }),
    {
      marker: "fix(relief): keep the public field\n## Summary\n\n- keep the public field\n\n",
      releaseType: "none",
      source: "pull_request_event",
    },
  );
});

test("only the pull request this push is the merge of supplies the marker", () => {
  const unrelatedOpen = {
    body: "BREAKING CHANGE: not merged yet.\n",
    merge_commit_sha: squashSha,
    merged_at: null,
    number: 3722,
    title: "fix(relief): unrelated open pull request",
  };
  const earlierMerge: MergedPullRequest = {
    body: "BREAKING CHANGE: merged by an earlier push.\n",
    merge_commit_sha: previousSha,
    merged_at: "2026-08-02T07:10:00Z",
    number: 3718,
    title: "fix(atelier): honor Babel custom element predicates",
  };

  assert.equal(
    selectSquashedPullRequest({
      pullRequests: [unrelatedOpen, earlierMerge, breakingPullRequest],
      sha: squashSha,
    }),
    breakingPullRequest,
  );
  assert.equal(
    selectSquashedPullRequest({ pullRequests: [unrelatedOpen, earlierMerge], sha: squashSha }),
    null,
  );
  assert.equal(selectSquashedPullRequest({ pullRequests: [], sha: squashSha }), null);
  assert.equal(selectSquashedPullRequest({ sha: squashSha }), null);
});

test("marker text keeps the shell contract for titles, bodies, and CRLF", () => {
  assert.equal(pullRequestMarkerText({ body: "body", title: "title" }), "title\nbody\n");
  assert.equal(pullRequestMarkerText({ body: null, title: "title" }), "title\n\n");
  assert.equal(pullRequestMarkerText({}), "\n\n");
  assert.equal(
    pullRequestMarkerText({ body: "## Note\r\nBREAKING CHANGE: drop it.\r\n", title: "fix: x" }),
    "fix: x\n## Note\nBREAKING CHANGE: drop it.\n\n",
  );

  assert.equal(releaseTypeForMarker("feat(relief)!: drop it"), "major");
  assert.equal(releaseTypeForMarker("feat!: drop it"), "major");
  assert.equal(releaseTypeForMarker("chore: note\r\n\r\nBREAKING CHANGE: drop it"), "major");
  assert.equal(releaseTypeForMarker("chore: mention BREAKING CHANGE: inline"), "none");
  assert.equal(releaseTypeForMarker("feat(relief): keep it"), "none");
  assert.equal(releaseTypeForMarker(""), "none");
});
