import assert from "node:assert/strict";
import { test } from "node:test";
import { rustCachePolicy } from "../../.github/actions/setup-rust-sticky-cache/cache-policy.ts";
import type { RustCacheContext } from "../../.github/actions/setup-rust-sticky-cache/cache-policy.ts";
import { cacheFixture, executeCacheAction } from "./support/rust-cache-fixture.mjs";

test("release dispatches can opt into trusted cache reads without gaining writes", () => {
  cacheFixture(
    (fixture: { cwd: string; context: (event?: string, ref?: string) => RustCacheContext }) => {
      const main = { ...fixture.context(), role: "release-cli-x86_64-unknown-linux-gnu" };
      // The checkout is the stamped candidate, a child of the dispatched commit.
      const release = {
        ...fixture.context("workflow_dispatch"),
        sourceSha: "a".repeat(40),
        role: main.role,
        restoreTrusted: true,
      };
      const trusted = rustCachePolicy(main, { cwd: fixture.cwd });
      const plan = rustCachePolicy(release, { cwd: fixture.cwd });
      assert.equal(plan.sticky, "true");
      assert.equal(plan.trusted, "false");
      assert.equal(plan["sticky-commit"], "false");
      assert.equal(plan["target-save"], "false");
      assert.equal(plan["sticky-target-key"], trusted["sticky-target-key"]);
      const result = executeCacheAction(fixture, release);
      const trace = result.trace as Array<{ kind: string; commit?: string }>;
      assert.equal(result.status, 0);
      assert.equal(trace.filter((row) => row.kind === "workload").length, 1);
      assert.equal(trace.filter((row) => row.kind === "cache-post").length, 0);
      assert.ok(
        trace.filter((row) => row.kind === "sticky-post").every((row) => row.commit === "false"),
      );
      assert.equal(
        rustCachePolicy({ ...release, sourceSha: release.checkoutSha }, { cwd: fixture.cwd })
          .sticky,
        "true",
      );
      for (const altered of [
        { restoreTrusted: false },
        { role: "nextest-ci" },
        { ref: "refs/heads/release-v1.2.3" },
        { eventName: "push" },
      ]) {
        assert.equal(
          rustCachePolicy({ ...release, ...altered }, { cwd: fixture.cwd }).sticky,
          "false",
        );
      }
    },
  );
});

test("manual Full coverage reads the trusted instrumented target without gaining writes", () => {
  cacheFixture(
    (fixture: { cwd: string; context: (event?: string, ref?: string) => RustCacheContext }) => {
      const main = { ...fixture.context("schedule"), role: "coverage-source" };
      const manual = {
        ...fixture.context("workflow_dispatch", "refs/heads/feature"),
        role: main.role,
        restoreTrusted: true,
      };
      const trusted = rustCachePolicy(main, { cwd: fixture.cwd });
      const plan = rustCachePolicy(manual, { cwd: fixture.cwd });
      assert.equal(trusted["sticky-commit"], "true");
      assert.equal(plan.sticky, "true");
      assert.equal(plan.trusted, "false");
      assert.equal(plan["sticky-commit"], "false");
      assert.equal(plan["target-save"], "false");
      assert.equal(plan["sticky-target-key"], trusted["sticky-target-key"]);
      for (const altered of [
        { restoreTrusted: false },
        { role: "coverage-branch" },
        { role: "nextest-ci" },
        { checkoutSha: "f".repeat(40) },
      ]) {
        assert.equal(
          rustCachePolicy({ ...manual, ...altered }, { cwd: fixture.cwd }).sticky,
          "false",
        );
      }
    },
  );
});
