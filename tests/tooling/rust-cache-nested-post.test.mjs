import assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { cacheFixture, executeCacheAction } from "./support/rust-cache-fixture.mjs";

await test("nested cache posts save the validated target inputs after step outputs disappear", () => {
  for (const targetPath of ["target", "head/target", "tests/fuzz/target"])
    cacheFixture((fixture) => {
      const context = {
        ...fixture.context(),
        targetPath,
        secondaryRole: "benchmark-base",
        secondarySuffix: "Linux-X64",
        secondaryPath: "base/target",
      };
      const result = executeCacheAction(fixture, context, { nestedPost: true });
      assert.equal(result.status, 0);
      const workload = result.trace.find((row) => row.kind === "workload");
      const seeds = result.trace.filter((row) => row.kind === "cache-post");
      assert.equal(seeds.length, 4);
      const target = seeds.find((row) => row.name === "target");
      const secondary = seeds.find((row) => row.name === "secondary");
      assert.equal(resolve(fixture.cwd, target.path), workload.path);
      assert.equal(target.saved, `compiled:${workload.artifact}`);
      assert.equal(target.warning, undefined);
      assert.equal(resolve(fixture.cwd, secondary.path), resolve(fixture.cwd, "base/target"));
      assert.equal(secondary.saved, "sticky clone");
      const firstUnmount = result.trace.findIndex((row) => row.kind === "sticky-post");
      assert.ok(seeds.every((row) => result.trace.indexOf(row) < firstUnmount));
      assert.equal(target.key, result.outputs["cache-policy"]["target-key"]);
    });
});

await test("the old output-based path loses the seed while its warning lets cleanup finish", () => {
  const action = parse(
    readFileSync(
      new URL("../../.github/actions/setup-rust-sticky-cache/action.yml", import.meta.url),
      "utf8",
    ),
  );
  for (const step of action.runs.steps) {
    if (step.uses?.startsWith("actions/cache@") && step.with.path === "${{ inputs.target-path }}")
      step.with.path = "${{ steps.cache-policy.outputs.target-path }}";
  }
  cacheFixture((fixture) => {
    const { status, trace } = executeCacheAction(fixture, fixture.context(), {
      nestedPost: true,
      steps: action.runs.steps,
    });
    assert.equal(status, 0);
    const post = trace.find((row) => row.kind === "cache-post" && row.name === "target");
    assert.equal(post.warning, "Input required and not supplied: path");
    assert.equal(post.saved, null);
    assert.equal(trace.filter((row) => row.kind === "workload").length, 1);
    assert.equal(trace.filter((row) => row.kind === "sticky-post").length, 3);
  });
});

await test("nested untrusted cache hits retain fresh validation and never register a save", () => {
  cacheFixture((fixture) => {
    const result = executeCacheAction(
      fixture,
      fixture.context("pull_request", "refs/pull/17/merge"),
      { nestedPost: true, hit: true },
    );
    assert.equal(result.status, 0);
    assert.equal(result.trace.filter((row) => row.kind === "cache-post").length, 0);
    assert.equal(result.trace.filter((row) => row.kind === "sticky").length, 0);
    assert.equal(result.trace.filter((row) => row.kind === "workload").length, 1);
  });
});

await test("a versioned nested seed restores both targets in a PR and a failed-mount fallback", () => {
  for (const targetPath of ["target", "head/target", "tests/fuzz/target"])
    cacheFixture((fixture) => {
      const paths = {
        targetPath,
        secondaryPath: "base/target",
        secondaryRole: "benchmark-base",
        secondarySuffix: "Linux-X64",
      };
      const seeded = executeCacheAction(
        fixture,
        { ...fixture.context(), ...paths },
        { nestedPost: true, storedCache: true },
      );
      const prior = seeded.trace.filter((row) => row.kind === "cache-post");
      const original = seeded.trace.find((row) => row.kind === "workload");
      writeFileSync(resolve(fixture.cwd, "source.txt"), "current input after the seed\n");
      fixture.git("add", "source.txt");
      fixture.git(
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.test",
        "commit",
        "--quiet",
        "-m",
        "current source",
      );
      const sha = fixture.git("rev-parse", "HEAD");
      for (const event of ["pull_request", "push"]) {
        for (const path of [targetPath, paths.secondaryPath])
          rmSync(resolve(fixture.cwd, path), { recursive: true, force: true });
        const context = {
          ...fixture.context(event, event === "push" ? "refs/heads/main" : "refs/pull/23/merge"),
          ...paths,
          sourceSha: sha,
          checkoutSha: sha,
        };
        const result = executeCacheAction(fixture, context, {
          nestedPost: true,
          storedCache: true,
          mountFailures: event === "push" ? ["target", "secondary"] : [],
        });
        assert.equal(result.status, 0);
        for (const name of ["target", "secondary"]) {
          const restored = result.trace.find((row) => row.kind === "cache" && row.name === name);
          const seed = prior.find((row) => row.name === name);
          assert.equal(restored.version, seed.version);
          assert.equal(restored.matchedKey, seed.key);
          assert.equal(restored.restoredArtifact, seed.saved);
          assert.equal(restored.hit, "partial");
        }
        const current = result.trace.find((row) => row.kind === "workload");
        assert.equal(current.before, "Actions restore");
        assert.notEqual(current.artifact, original.artifact);
        const posts = result.trace.filter((row) => row.kind === "cache-post");
        if (event === "pull_request") assert.equal(posts.length, 0);
        else
          assert.equal(
            posts.find((row) => row.name === "target").saved,
            `compiled:${current.artifact}`,
          );
      }
    });
});

await test("an absolute-path reader cannot use a relative-path seed despite an identical key", () => {
  const action = parse(
    readFileSync(
      new URL("../../.github/actions/setup-rust-sticky-cache/action.yml", import.meta.url),
      "utf8",
    ),
  );
  for (const step of action.runs.steps)
    if (step.uses?.startsWith("actions/cache/restore@")) {
      if (step.with.path === "${{ inputs.target-path }}")
        step.with.path = "${{ steps.cache-policy.outputs.target-path }}";
      if (step.with.path === "${{ inputs.secondary-target-path }}")
        step.with.path = "${{ steps.cache-policy.outputs.secondary-path }}";
    }
  cacheFixture((fixture) => {
    const paths = {
      secondaryPath: "secondary",
      secondaryRole: "second",
      secondarySuffix: "Linux-X64",
    };
    const seed = executeCacheAction(
      fixture,
      { ...fixture.context(), ...paths },
      { storedCache: true, nestedPost: true },
    );
    for (const path of ["target", "secondary"])
      rmSync(resolve(fixture.cwd, path), { recursive: true, force: true });
    const result = executeCacheAction(
      fixture,
      { ...fixture.context("pull_request", "refs/pull/24/merge"), ...paths },
      { storedCache: true, nestedPost: true, steps: action.runs.steps },
    );
    for (const name of ["target", "secondary"]) {
      const writer = seed.trace.find((row) => row.kind === "cache-post" && row.name === name);
      const reader = result.trace.find((row) => row.kind === "cache" && row.name === name);
      assert.equal(reader.key, writer.key);
      assert.notEqual(reader.version, writer.version);
      assert.equal(reader.hit, false);
      assert.equal(reader.restoredArtifact, undefined);
    }
    const workload = result.trace.find((row) => row.kind === "workload");
    assert.equal(workload.before, null);
    assert.equal(
      readFileSync(resolve(fixture.cwd, "target/cache-artifact.txt"), "utf8"),
      `compiled:${workload.artifact}`,
    );
  });
});

await test("a stored seed cannot bypass nested path validation for either target", () => {
  for (const change of [
    { targetPath: "../outside" },
    { secondaryPath: "../outside", secondaryRole: "second", secondarySuffix: "Linux-X64" },
  ])
    cacheFixture((fixture) => {
      executeCacheAction(fixture, fixture.context(), { storedCache: true, nestedPost: true });
      const before = readFileSync(resolve(fixture.cwd, "stored-cache.json"), "utf8");
      const result = executeCacheAction(
        fixture,
        { ...fixture.context("pull_request", "refs/pull/25/merge"), ...change },
        { storedCache: true, nestedPost: true },
      );
      assert.notEqual(result.status, 0);
      assert.deepEqual(result.trace, []);
      assert.equal(readFileSync(resolve(fixture.cwd, "stored-cache.json"), "utf8"), before);
    });
});
