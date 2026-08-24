import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  assertRollbackContext,
  remoteTagState,
  rollbackUnpublishedTag,
} from "../../tools/github/release-tag-rollback.mjs";
import { readRepoFile } from "./support/github-workflows.ts";

const tag = "v0.348.0";
const commitSha = "a".repeat(40);
const tagObjectSha = "b".repeat(40);

function releaseEnv(overrides: Record<string, string> = {}) {
  return {
    GITHUB_API_URL: "https://api.github.test",
    GITHUB_REF_NAME: tag,
    GITHUB_REF_TYPE: "tag",
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_SHA: commitSha,
    GITHUB_TOKEN: "test-token",
    RELEASE_PREFLIGHT_RESULT: "failure",
    ...overrides,
  };
}

function fakeGit({
  localCommit = commitSha,
  localObject = tagObjectSha,
  remoteCommit = commitSha,
  remoteObject = tagObjectSha,
}: {
  localCommit?: string;
  localObject?: string;
  remoteCommit?: string;
  remoteObject?: string;
} = {}) {
  const calls: string[][] = [];
  const git = (args: string[]) => {
    calls.push(args);
    if (args[0] === "ls-remote") {
      const stdout = remoteObject
        ? `${remoteObject}\trefs/tags/${tag}\n${remoteCommit}\trefs/tags/${tag}^{}\n`
        : "";
      return { status: 0, stderr: "", stdout };
    }
    if (args[0] === "fetch" || args[0] === "push") {
      return { status: 0, stderr: "", stdout: "" };
    }
    if (args[0] === "rev-parse") {
      return {
        status: 0,
        stderr: "",
        stdout: `${args[1].endsWith("^{}") ? localCommit : localObject}\n`,
      };
    }
    throw new Error(`Unexpected git call: ${args.join(" ")}`);
  };
  return { calls, git };
}

function githubResponse(status: number, body = "") {
  return async () => ({ status, text: async () => body });
}

test("rollback context accepts only failed tag-event preflights", () => {
  assert.equal(assertRollbackContext(releaseEnv()).tag, tag);
  assert.equal(
    assertRollbackContext(releaseEnv({ RELEASE_PREFLIGHT_RESULT: "cancelled" })).tag,
    tag,
  );
  for (const [overrides, message] of [
    [{ RELEASE_PREFLIGHT_RESULT: "success" }, /concluded success/],
    [{ RELEASE_PREFLIGHT_RESULT: "skipped" }, /concluded skipped/],
    [{ GITHUB_REF_TYPE: "branch" }, /requires a tag event/],
    [{ GITHUB_REF_NAME: "0.348.0" }, /v-prefixed/],
    [{ GITHUB_SHA: "abc" }, /full event SHA/],
    [{ GITHUB_REPOSITORY: "vize" }, /owner\/repository/],
    [{ GITHUB_TOKEN: "" }, /requires GITHUB_TOKEN/],
  ] as const) {
    assert.throws(() => assertRollbackContext(releaseEnv(overrides)), message);
  }
});

test("remote tag parsing preserves annotated tag object and peeled commit identities", () => {
  assert.deepEqual(
    remoteTagState(`${tagObjectSha}\trefs/tags/${tag}\n${commitSha}\trefs/tags/${tag}^{}\n`, tag),
    { commitSha, objectSha: tagObjectSha },
  );
  assert.deepEqual(remoteTagState(`${commitSha}\trefs/tags/${tag}\n`, tag), {
    commitSha,
    objectSha: commitSha,
  });
  assert.equal(remoteTagState("", tag), undefined);
});

test("rollback deletes the audited tag with an exact force-with-lease", async () => {
  const { calls, git } = fakeGit();
  let releaseChecks = 0;
  const result = await rollbackUnpublishedTag({
    env: releaseEnv(),
    fetchImpl: async () => {
      releaseChecks += 1;
      return { status: 404, text: async () => "" };
    },
    git,
  });

  assert.deepEqual(result, { deleted: true, tag });
  assert.equal(releaseChecks, 2);
  assert.deepEqual(calls.at(-1), [
    "push",
    `--force-with-lease=refs/tags/${tag}:${tagObjectSha}`,
    "origin",
    `:refs/tags/${tag}`,
  ]);
});

test("rollback is idempotent when the remote tag is already absent", async () => {
  const { calls, git } = fakeGit({ remoteObject: "" });
  const result = await rollbackUnpublishedTag({
    env: releaseEnv(),
    fetchImpl: () => {
      throw new Error("the API should not be queried for an absent tag");
    },
    git,
  });

  assert.deepEqual(result, { deleted: false, reason: "already-absent", tag });
  assert.equal(calls.length, 1);
});

test("rollback refuses a published GitHub Release or inconclusive API response", async () => {
  for (const [status, body, message] of [
    [200, "", /GitHub Release already exists/],
    [503, "unavailable", /Could not prove.*503: unavailable/],
  ] as const) {
    const { calls, git } = fakeGit();
    await assert.rejects(
      rollbackUnpublishedTag({ env: releaseEnv(), fetchImpl: githubResponse(status, body), git }),
      message,
    );
    assert.equal(
      calls.some((args) => args[0] === "push"),
      false,
    );
  }
});

test("rollback refuses a Release published during the pre-deletion recheck", async () => {
  const { calls, git } = fakeGit();
  let releaseChecks = 0;
  const fetchImpl = async () => {
    releaseChecks += 1;
    if (releaseChecks === 1) return { status: 404, text: async () => "" };
    assert.equal(
      calls.some((args) => args[0] === "fetch"),
      true,
    );
    return { status: 200, text: async () => "" };
  };

  await assert.rejects(
    rollbackUnpublishedTag({ env: releaseEnv(), fetchImpl, git }),
    /GitHub Release already exists/,
  );

  assert.equal(releaseChecks, 2);
  assert.equal(
    calls.some((args) => args[0] === "push"),
    false,
  );
});

test("rollback refuses changed remote, event, or fetched tag identities", async () => {
  for (const [options, message] of [
    [{ remoteCommit: "c".repeat(40) }, /not event SHA/],
    [{ localObject: "c".repeat(40) }, /fetched tag identity/],
    [{ localCommit: "c".repeat(40) }, /fetched tag identity/],
  ] as const) {
    const { calls, git } = fakeGit(options);
    await assert.rejects(
      rollbackUnpublishedTag({ env: releaseEnv(), fetchImpl: githubResponse(404), git }),
      message,
    );
    assert.equal(
      calls.some((args) => args[0] === "push"),
      false,
    );
  }
});

test("release calls a credential-minimal hosted rollback workflow after preflight failure", () => {
  const release = parse(readRepoFile(".github", "workflows", "release.yml")) as {
    jobs: Record<string, Record<string, unknown>>;
  };
  const caller = release.jobs["rollback-unpublished-tag"];
  assert.equal(caller.needs, "release-preflight");
  assert.equal(
    caller.if,
    "${{ github.event_name == 'push' && always() && needs.release-preflight.result != 'success' }}",
  );
  assert.deepEqual(caller.permissions, { contents: "write" });
  assert.equal(caller.uses, "./.github/workflows/release-tag-rollback.yml");
  assert.deepEqual(caller.with, {
    preflight_result: "${{ needs.release-preflight.result }}",
  });

  const called = parse(readRepoFile(".github", "workflows", "release-tag-rollback.yml")) as {
    jobs: Record<string, Record<string, unknown>>;
  };
  const rollback = called.jobs.rollback;
  assert.equal(rollback["runs-on"], "ubuntu-24.04");
  assert.equal(rollback["timeout-minutes"], 5);
  assert.deepEqual(rollback.permissions, { contents: "write" });
  assert.doesNotMatch(JSON.stringify(rollback), /environment|id-token|secrets\./);
  assert.match(JSON.stringify(rollback), /release-tag-rollback\.mjs/);
});

test("release stabilizes apt before installing ARM64 cross-compilation tools", () => {
  const release = parse(readRepoFile(".github", "workflows", "release.yml")) as {
    jobs: Record<string, { steps: Array<{ run?: string; uses?: string }> }>;
  };
  const steps = release.jobs["build-cli"].steps;
  const archiveSetup = steps.findIndex(
    (step) => step.uses === "./.github/actions/setup-ubuntu-archive",
  );
  const aptInstall = steps.findIndex((step) =>
    step.run?.includes("install_cross_compile_tools --"),
  );
  assert.ok(archiveSetup >= 0);
  assert.ok(aptInstall > archiveSetup);
});
