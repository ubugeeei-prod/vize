import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
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
