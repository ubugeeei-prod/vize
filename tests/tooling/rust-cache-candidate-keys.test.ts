import assert from "node:assert/strict";
import { test } from "node:test";
import { rustCachePolicy } from "../../.github/actions/setup-rust-sticky-cache/cache-policy.ts";
import { cacheFixture } from "./support/rust-cache-fixture.mjs";

type Plan = Record<string, string>;
type Context = { sourceSha: string; runnerEnvironment: string };
type Fixture = { cwd: string; context: (event?: string, ref?: string) => Context };
const policy = (context: Context, cwd: string) => rustCachePolicy(context, { cwd }) as Plan;

await test("PR numbers, queue SHAs and nondefault dispatches never create or commit provider keys", () => {
  cacheFixture((fixture: Fixture) => {
    const stable = policy(fixture.context(), fixture.cwd);
    const stableKeys = ["registry", "git", "target"].map(
      (name: string) => stable[`sticky-${name}-key`],
    );
    for (let index = 1; index <= 256; index++) {
      for (const [event, ref] of [
        ["pull_request", `refs/pull/${index}/merge`],
        ["merge_group", `refs/heads/gh-readonly-queue/main/pr-${index}-${index.toString(16)}`],
      ]) {
        const context = fixture.context(event, ref);
        context.sourceSha = index.toString(16).padStart(40, "0");
        const plan = policy(context, fixture.cwd);
        // Candidates read a clone of the trusted disks that is never committed.
        assert.equal(plan.sticky, "true");
        assert.equal(plan["sticky-commit"], "false");
        assert.deepEqual(
          ["registry", "git", "target"].map((name: string) => plan[`sticky-${name}-key`]),
          stableKeys,
        );
      }
      for (const [event, ref] of [
        ["workflow_dispatch", `refs/heads/private-${index}`],
        ["push", `refs/heads/private-${index}`],
      ]) {
        const context = fixture.context(event, ref);
        context.sourceSha = index.toString(16).padStart(40, "0");
        const plan = policy(context, fixture.cwd);
        assert.equal(plan.sticky, "false");
        assert.equal(plan["sticky-commit"], "false");
      }
    }
    const hosted = fixture.context("pull_request", "refs/pull/1/merge");
    hosted.runnerEnvironment = "github-hosted";
    assert.equal(policy(hosted, fixture.cwd).sticky, "false");
    assert.equal(stable["sticky-commit"], "true");
  });
});
