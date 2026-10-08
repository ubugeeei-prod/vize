import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { cacheFixture, executeCacheAction } from "./support/rust-cache-fixture.mjs";

const action = parse(
  readFileSync(
    new URL("../../.github/actions/setup-rust-sticky-cache/action.yml", import.meta.url),
    "utf8",
  ),
);
const largeRoles = ["coverage-source", "clippy-test"];

await test("oversized trusted targets build normally without a cache save post", () => {
  for (const role of largeRoles) {
    for (const scenario of ["mounted", "failed-mount", "github-hosted"]) {
      cacheFixture((fixture) => {
        const context = {
          ...fixture.context("workflow_dispatch"),
          role,
          runnerEnvironment: scenario === "github-hosted" ? "github-hosted" : "self-hosted",
        };
        const result = executeCacheAction(fixture, context, {
          nestedPost: true,
          mountFailures: scenario === "failed-mount" ? ["target"] : [],
        });
        assert.equal(result.status, 0);
        const caches = result.trace.filter((row) => row.kind === "cache");
        assert.equal(caches.length, 3);
        const target = caches.find((row) => row.name === "target");
        assert.equal(target.restoreOnly, true);
        assert.equal(target.lookup, scenario === "mounted");
        assert.equal(target.hit, false);
        const workload = result.trace.filter((row) => row.kind === "workload");
        assert.equal(workload.length, 1);
        assert.equal(readFileSync(join(fixture.cwd, "artifact.txt"), "utf8"), workload[0].artifact);
        assert.equal(
          result.trace.filter((row) => row.kind === "cache-post" && row.name === "target").length,
          0,
        );
        assert.deepEqual(
          result.trace.filter((row) => row.kind === "cache-post").map((row) => row.name),
          ["git", "registry"],
        );
        const unmounts = result.trace.filter((row) => row.kind === "sticky-post");
        assert.equal(unmounts.length, scenario === "github-hosted" ? 0 : 3);
        if (unmounts.length) {
          const firstUnmount = result.trace.indexOf(unmounts[0]);
          assert.ok(result.trace.indexOf(workload[0]) < firstUnmount);
          assert.ok(
            result.trace
              .filter((row) => row.kind === "cache-post")
              .every((row) => result.trace.indexOf(row) < firstUnmount),
          );
        }
      });
    }
  }
});

await test("a prior compatible target can still restore, then current input is rebuilt", () => {
  for (const role of largeRoles) {
    cacheFixture((fixture) => {
      const context = { ...fixture.context(), role, runnerEnvironment: "github-hosted" };
      const previousSteps = structuredClone(action.runs.steps);
      for (const step of previousSteps) {
        if (step.name === "Cache Rust target or seed it from the trusted sticky clone")
          step.if = "${{ steps.cache-policy.outputs.trusted == 'true' }}";
        if (step.name === "Restore a Rust target without registering a save")
          step.if = "${{ steps.cache-policy.outputs.trusted == 'false' }}";
      }
      const previous = executeCacheAction(fixture, context, {
        steps: previousSteps,
        storedCache: true,
      });
      assert.equal(previous.status, 0);
      const previousSave = previous.trace.find(
        (row) => row.kind === "cache-post" && row.name === "target",
      );
      assert.ok(previousSave.saved.startsWith("compiled:"));
      const storedBefore = readFileSync(join(fixture.cwd, "stored-cache.json"), "utf8");
      const source = "new source after the earlier compatible cache\n";
      writeFileSync(join(fixture.cwd, "source.txt"), source);
      const result = executeCacheAction(fixture, context, { storedCache: true });
      assert.equal(result.status, 0);
      const restored = result.trace.find((row) => row.kind === "cache" && row.name === "target");
      assert.equal(restored.restoreOnly, true);
      assert.equal(restored.hit, true);
      assert.equal(restored.restoredArtifact, previousSave.saved);
      const workload = result.trace.find((row) => row.kind === "workload");
      assert.equal(workload.artifact, createHash("sha256").update(source).digest("hex"));
      assert.notEqual(`compiled:${workload.artifact}`, previousSave.saved);
      assert.equal(
        result.trace.filter((row) => row.kind === "cache-post" && row.name === "target").length,
        0,
      );
      assert.equal(readFileSync(join(fixture.cwd, "stored-cache.json"), "utf8"), storedBefore);
    });
  }
});

await test("primary and secondary role eligibility remain independent", () => {
  for (const [primary, secondary] of [
    ["coverage-source", "docs-example"],
    ["docs-example", "clippy-test"],
  ]) {
    cacheFixture((fixture) => {
      const result = executeCacheAction(fixture, {
        ...fixture.context(),
        role: primary,
        secondaryRole: secondary,
        secondaryPath: "secondary",
        secondarySuffix: "Linux-X64",
      });
      assert.equal(result.status, 0);
      for (const [name, role] of [
        ["target", primary],
        ["secondary", secondary],
      ]) {
        const target = result.trace.find((row) => row.kind === "cache" && row.name === name);
        assert.equal(target.restoreOnly, largeRoles.includes(role));
        assert.equal(target.lookup, true);
        const posts = result.trace.filter((row) => row.kind === "cache-post" && row.name === name);
        assert.equal(posts.length, largeRoles.includes(role) ? 0 : 1);
        if (posts.length) assert.ok(posts[0].saved !== null);
      }
      assert.equal(result.trace.filter((row) => row.kind === "sticky-post").length, 4);
    });
  }
});

await test("eligible ordinary roles still save while every untrusted role remains restore-only", () => {
  for (const role of ["test-scripts", "nextest-ci", "coverage", "new-role", ...largeRoles]) {
    cacheFixture((fixture) => {
      const context = { ...fixture.context(), role };
      const trusted = executeCacheAction(fixture, context);
      assert.equal(trusted.status, 0);
      assert.equal(
        trusted.trace.filter((row) => row.kind === "cache-post" && row.name === "target").length,
        largeRoles.includes(role) || role === "test-scripts" ? 0 : 1,
      );
      const untrusted = executeCacheAction(fixture, {
        ...fixture.context("merge_group", "refs/heads/gh-readonly-queue/main/pr-1-a"),
        role,
      });
      assert.equal(untrusted.status, 0);
      assert.equal(
        untrusted.trace.filter((row) => row.kind === "sticky" || row.kind === "cache-post").length,
        0,
      );
      assert.ok(
        untrusted.trace
          .filter((row) => row.kind === "cache")
          .every((row) => row.restoreOnly && !row.lookup),
      );
      assert.equal(untrusted.trace.filter((row) => row.kind === "workload").length, 1);
    });
  }
});

await test("mounted tooling omits its archive post while the original conditions reproduce it", () => {
  const original = structuredClone(action.runs.steps);
  for (const step of original) {
    if (step.name === "Cache Rust target or seed it from the trusted sticky clone")
      step.if = "${{ steps.cache-policy.outputs.target-save == 'true' }}";
    if (step.name === "Restore a Rust target without registering a save")
      step.if = "${{ steps.cache-policy.outputs.target-save == 'false' }}";
  }
  for (const event of ["push", "schedule", "workflow_dispatch"])
    for (const hit of [false, "partial", true])
      for (const role of ["test-scripts", "Test-Scripts"])
        cacheFixture((fixture) => {
          const context = {
            ...fixture.context(event),
            role,
            secondaryRole: "docs-example",
            secondarySuffix: "Linux-X64",
            secondaryPath: "secondary",
          };
          const verify = (result) => {
            assert.equal(result.status, 0);
            const target = result.trace.filter(
              (row) => row.kind === "cache" && row.name === "target",
            );
            assert.equal(target.length, 1);
            assert.equal(target[0].restoreOnly, true);
            assert.equal(target[0].lookup, true);
            const posts = result.trace.filter((row) => row.kind === "cache-post");
            assert.deepEqual(
              posts.map((row) => row.name),
              ["secondary", "git", "registry"],
            );
            const workload = result.trace.filter((row) => row.kind === "workload");
            assert.equal(workload.length, 1);
            assert.equal(
              readFileSync(join(fixture.cwd, "target/cache-artifact.txt"), "utf8"),
              `compiled:${workload[0].artifact}`,
            );
            const unmount = result.trace.findIndex((row) => row.kind === "sticky-post");
            assert.equal(result.trace.filter((row) => row.kind === "sticky-post").length, 4);
            assert.ok(result.trace.indexOf(workload[0]) < unmount);
            assert.ok(posts.every((row) => result.trace.indexOf(row) < unmount));
          };
          const old = executeCacheAction(fixture, context, {
            hit,
            nestedPost: true,
            steps: original,
          });
          assert.equal(old.status, 0);
          assert.equal(
            old.trace.filter((row) => row.kind === "cache-post" && row.name === "target").length,
            1,
          );
          assert.throws(() => verify(old), assert.AssertionError);
          verify(executeCacheAction(fixture, context, { hit, nestedPost: true }));
        });
});

await test("tooling mount failures and hosted fallbacks retain saves and a secondary tooling role is independent", () => {
  for (const event of ["push", "schedule", "workflow_dispatch"])
    for (const mode of ["failed-mount", "github-hosted", "ordinary-primary"])
      for (const hit of [false, "partial", true])
        cacheFixture((fixture) => {
          const context = {
            ...fixture.context(event),
            role: mode === "ordinary-primary" ? "docs-example" : "test-scripts",
            runnerEnvironment: mode === "github-hosted" ? "github-hosted" : "self-hosted",
            secondaryRole: "test-scripts",
            secondarySuffix: "Linux-X64",
            secondaryPath: "secondary",
          };
          const result = executeCacheAction(fixture, context, {
            hit,
            nestedPost: true,
            mountFailures: mode === "failed-mount" ? ["target"] : [],
          });
          assert.equal(result.status, 0);
          for (const name of ["target", "secondary"]) {
            const cache = result.trace.find((row) => row.kind === "cache" && row.name === name);
            assert.equal(cache.restoreOnly, false);
            assert.equal(
              cache.lookup,
              mode !== "github-hosted" && (name === "secondary" || mode === "ordinary-primary"),
            );
            const posts = result.trace.filter(
              (row) => row.kind === "cache-post" && row.name === name,
            );
            assert.equal(posts.length, 1);
            const workload = result.trace.find((row) => row.kind === "workload");
            assert.ok(result.trace.indexOf(workload) < result.trace.indexOf(posts[0]));
            if (name === "target" && hit !== true)
              assert.equal(posts[0].saved, `compiled:${workload.artifact}`);
          }
          assert.equal(result.trace.filter((row) => row.kind === "cache").length, 4);
          assert.equal(result.trace.filter((row) => row.kind === "cache-post").length, 4);
        });
});

await test("save omission never bypasses malformed path or event validation", () => {
  for (const changes of [
    { targetPath: "../outside" },
    { eventName: "unknown" },
    { role: "coverage-source\nextra=true" },
  ]) {
    cacheFixture((fixture) => {
      const result = executeCacheAction(fixture, {
        ...fixture.context(),
        role: "coverage-source",
        ...changes,
      });
      assert.notEqual(result.status, 0);
      assert.deepEqual(result.trace, []);
    });
  }
});
