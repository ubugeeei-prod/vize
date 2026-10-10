import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  publicationDecision,
  validateBuild,
  type Comparison,
} from "../../tools/support/compat/github/docs-deployment-policy.ts";
import {
  build,
  manifest,
  repositoryId,
  run,
  sourceArtifacts,
  sourceSha,
} from "./support/docs-deployment.ts";

test("complete native source custody accepts every main trigger and refuses foreign or incomplete inputs", () => {
  for (const event of ["push", "schedule", "workflow_dispatch"])
    assert.equal(build(run({ event })).sourceSha, sourceSha);
  for (const mutation of [
    { event: "pull_request" },
    { head_branch: "feature" },
    { head_repository: { id: 999 } },
    { repository: { id: 999 } },
    { path: ".github/workflows/other.yml" },
    { run_attempt: 2 },
    { id: 99 },
    { conclusion: "failure" },
    { status: "in_progress" },
    { head_sha: "abc" },
  ])
    assert.throws(() => build(run(mutation)));
  const artifacts = sourceArtifacts();
  for (const mutation of [
    { expired: true },
    { digest: "sha256:abc" },
    { id: 0 },
    { created_at: "2026-10-09T00:00:00Z" },
    { workflow_run: { ...artifacts[0].workflow_run, id: 999 } },
    { workflow_run: { ...artifacts[0].workflow_run, head_sha: "d".repeat(40) } },
    { workflow_run: { ...artifacts[0].workflow_run, head_repository_id: 999 } },
    { workflow_run: { ...artifacts[0].workflow_run, head_branch: "feature" } },
  ])
    assert.throws(() => build(run(), [{ ...artifacts[0], ...mutation }, ...artifacts.slice(1)]));
  assert.throws(() => build(run(), artifacts.slice(1)), /Exactly one/);
  assert.throws(() => build(run(), [...artifacts, artifacts[0]]), /Exactly one/);
  assert.throws(
    () =>
      validateBuild(run(), repositoryId, { runId: run().id, attempt: 1, sourceSha }, artifacts, {
        ...manifest,
        sourceSha: "e".repeat(40),
      }),
    /manifest source/,
  );
  assert.throws(
    () =>
      validateBuild(run(), repositoryId, { runId: run().id, attempt: 1, sourceSha }, artifacts, {
        ...manifest,
        assetFingerprint: "",
      }),
    /fingerprint/,
  );
});

test("real Git ancestry permits completed ancestors without rollback, and refuses detached sources", () => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-docs-lineage-"));
  const git = (...args: string[]) =>
    execFileSync("git", ["-C", directory, ...args], { encoding: "utf8" }).trim();
  try {
    git("init", "--initial-branch=main");
    git("config", "user.name", "Fixture");
    git("config", "user.email", "fixture@example.invalid");
    const commit = (message: string) => {
      git("commit", "--allow-empty", "-m", message);
      return git("rev-parse", "HEAD");
    };
    const old = commit("old published source");
    const incoming = commit("completed build");
    const main = commit("main advanced during build");
    git("checkout", "-b", "detached", old);
    const detached = commit("unmerged source");
    function comparison(base: string, head: string): Comparison {
      const merge = git("merge-base", base, head);
      const ahead = Number(git("rev-list", "--count", base + ".." + head));
      const behind = Number(git("rev-list", "--count", head + ".." + base));
      return {
        base_commit: { sha: base },
        merge_base_commit: { sha: merge },
        status:
          base === head
            ? "identical"
            : merge === base
              ? "ahead"
              : merge === head
                ? "behind"
                : "diverged",
        ahead_by: ahead,
        behind_by: behind,
      };
    }
    assert.equal(
      spawnSync("git", ["-C", directory, "merge-base", "--is-ancestor", incoming, main]).status,
      0,
    );
    assert.deepEqual(
      publicationDecision(
        incoming,
        main,
        comparison(incoming, main),
        old,
        comparison(old, incoming),
        true,
      ),
      { eligible: true, reason: "Newer completed main build" },
    );
    assert.equal(
      publicationDecision(
        old,
        main,
        comparison(old, main),
        incoming,
        comparison(incoming, old),
        true,
      ).eligible,
      false,
    );
    assert.equal(
      publicationDecision(
        incoming,
        main,
        comparison(incoming, main),
        incoming,
        comparison(incoming, incoming),
        true,
      ).eligible,
      false,
    );
    assert.equal(
      publicationDecision(incoming, main, comparison(incoming, main), null, null, false).eligible,
      true,
    );
    assert.throws(
      () => publicationDecision(incoming, main, comparison(incoming, main), null, null, true),
      /authoritative publication floor/,
    );
    assert.throws(
      () =>
        publicationDecision(
          detached,
          main,
          comparison(detached, main),
          old,
          comparison(old, detached),
          true,
        ),
      /main ancestor/,
    );
    assert.throws(
      () =>
        publicationDecision(
          incoming,
          main,
          comparison(incoming, main),
          detached,
          comparison(detached, incoming),
          true,
        ),
      /protected main lineage/,
    );
    const wrongBase = { ...comparison(incoming, main), base_commit: { sha: old } };
    assert.throws(
      () => publicationDecision(incoming, main, wrongBase, null, null, false),
      /comparison base/,
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
