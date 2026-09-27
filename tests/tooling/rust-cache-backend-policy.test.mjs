import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { rustCachePolicy } from "../../.github/actions/setup-rust-sticky-cache/cache-policy.mjs";
import { cacheFixture, executeCacheAction } from "./support/rust-cache-fixture.mjs";

test("PR numbers, queue SHAs and nondefault dispatches do not create provider keys", () => {
  cacheFixture((fixture) => {
    for (let index = 1; index <= 256; index++) {
      for (const [event, ref] of [
        ["pull_request", `refs/pull/${index}/merge`],
        ["merge_group", `refs/heads/gh-readonly-queue/main/pr-${index}-${index.toString(16)}`],
        ["workflow_dispatch", `refs/heads/private-${index}`],
        ["push", `refs/heads/private-${index}`],
      ]) {
        const context = fixture.context(event, ref);
        context.sourceSha = index.toString(16).padStart(40, "0");
        assert.equal(rustCachePolicy(context, { cwd: fixture.cwd }).sticky, "false");
      }
    }
  });
});

test("a separate benchmark target follows its own lock and falls back to the root toolchain", () => {
  cacheFixture((fixture) => {
    const base = join(fixture.cwd, "base");
    mkdirSync(base);
    writeFileSync(join(base, "Cargo.lock"), "base lock\n");
    const context = {
      ...fixture.context(),
      secondaryRole: "benchmark-base",
      secondarySuffix: "Linux-X64",
      secondaryPath: "base/target",
    };
    const before = rustCachePolicy(context, { cwd: fixture.cwd });
    writeFileSync(join(base, "Cargo.lock"), "changed base lock\n");
    const changed = rustCachePolicy(context, { cwd: fixture.cwd });
    assert.equal(changed["target-key"], before["target-key"]);
    assert.notEqual(changed["secondary-key"], before["secondary-key"]);
    writeFileSync(join(fixture.cwd, "rust-toolchain.toml"), "changed root toolchain\n");
    assert.notEqual(
      rustCachePolicy(context, { cwd: fixture.cwd })["secondary-key"],
      changed["secondary-key"],
    );
  });
});

test("only the actual default-branch trusted checkout can write existing stable disks", () => {
  cacheFixture((fixture) => {
    for (const event of ["push", "schedule", "workflow_dispatch"]) {
      const context = fixture.context(event);
      const plan = rustCachePolicy(context, { cwd: fixture.cwd });
      assert.equal(plan.sticky, "true");
      assert.equal(plan["sticky-registry-key"], "example/repo-cargo-registry-Linux-X64");
      assert.equal(plan["sticky-target-key"], "example/repo-test-scripts-target-Linux-X64");
      context.event.repository.default_branch = "trunk";
      assert.equal(rustCachePolicy(context, { cwd: fixture.cwd }).sticky, "false");
      context.ref = "refs/heads/trunk";
      if (event === "push") context.event.ref = context.ref;
      assert.equal(rustCachePolicy(context, { cwd: fixture.cwd }).sticky, "true");
      context.checkoutSha = "f".repeat(40);
      assert.equal(rustCachePolicy(context, { cwd: fixture.cwd }).sticky, "false");
    }
    for (const event of ["pull_request_target", "workflow_run", "release", "repository_dispatch"]) {
      assert.equal(rustCachePolicy(fixture.context(event), { cwd: fixture.cwd }).sticky, "false");
    }
    const hosted = fixture.context();
    hosted.runnerEnvironment = "github-hosted";
    assert.equal(rustCachePolicy(hosted, { cwd: fixture.cwd }).sticky, "false");
  });
});

test("lock, toolchain, runner and role separate compatible Actions seeds", () => {
  cacheFixture((fixture) => {
    const context = fixture.context();
    const first = rustCachePolicy(context, { cwd: fixture.cwd });
    for (const [key, changed] of [
      ["role", "nextest-ci"],
      ["suffix", "Linux-ARM64"],
      ["runnerOs", "Windows"],
      ["runnerArch", "ARM64"],
    ]) {
      const plan = rustCachePolicy({ ...context, [key]: changed }, { cwd: fixture.cwd });
      assert.notEqual(plan["target-restore"], first["target-restore"]);
    }
    for (const file of ["Cargo.lock", "rust-toolchain.toml"]) {
      writeFileSync(join(fixture.cwd, file), "different inputs\n");
      assert.notEqual(
        rustCachePolicy(context, { cwd: fixture.cwd })["registry-key"],
        first["registry-key"],
      );
    }
    const advanced = rustCachePolicy(
      { ...context, checkoutSha: "a".repeat(40) },
      { cwd: fixture.cwd },
    );
    const current = rustCachePolicy(context, { cwd: fixture.cwd });
    assert.equal(advanced["target-restore"], current["target-restore"]);
    assert.notEqual(advanced["target-key"], current["target-key"]);
  });
});

test("actual backend children miss safely for many PRs, queues and nondefault dispatches", () => {
  for (let index = 1; index <= 8; index++) {
    for (const [event, ref] of [
      ["pull_request", `refs/pull/${6900 + index}/merge`],
      ["merge_group", `refs/heads/gh-readonly-queue/main/pr-${index}-deadbeef`],
      ["workflow_dispatch", `refs/heads/private-${index}`],
      ["push", `refs/heads/private-${index}`],
    ])
      cacheFixture((fixture) => {
        const result = executeCacheAction(fixture, fixture.context(event, ref));
        assert.equal(result.status, 0);
        assert.equal(result.trace.filter((row) => row.kind === "sticky").length, 0);
        const caches = result.trace.filter((row) => row.kind === "cache");
        assert.equal(caches.length, 3);
        assert.ok(caches.every((row) => row.hit === false && row.lookup === false));
        const workload = result.trace.filter((row) => row.kind === "workload");
        assert.equal(workload.length, 1);
        assert.equal(workload[0].before, null);
        assert.equal(readFileSync(join(fixture.cwd, "artifact.txt"), "utf8"), workload[0].artifact);
        assert.equal(
          new Set(result.trace.filter((row) => row.kind === "cache").map((row) => row.pid)).size,
          3,
        );
      });
  }
});

test("a trusted seed includes the fresh artifact and is saved before unmount", () => {
  cacheFixture((fixture) => {
    const result = executeCacheAction(fixture, fixture.context());
    assert.equal(result.status, 0);
    const seeds = result.trace.filter((row) => row.kind === "cache-post");
    assert.equal(seeds.length, 3);
    const artifact = readFileSync(join(fixture.cwd, "artifact.txt"), "utf8");
    assert.equal(seeds.find((row) => row.name === "target").saved, `compiled:${artifact}`);
    assert.ok(
      seeds.filter((row) => row.name !== "target").every((row) => row.saved === "sticky clone"),
    );
    const lastValidation = result.trace.findLastIndex((row) => row.kind === "workload");
    const firstSave = result.trace.findIndex((row) => row.kind === "cache-post");
    const firstUnmount = result.trace.findIndex((row) => row.kind === "sticky-post");
    assert.ok(lastValidation < firstSave && firstSave < firstUnmount);
    assert.ok(seeds.every((row) => result.trace.indexOf(row) < firstUnmount));
  });
});

test("a compatible prior target lookup saves the new exact seed without restoring over its clone", () => {
  cacheFixture((fixture) => {
    const result = executeCacheAction(fixture, fixture.context(), { hit: "partial" });
    assert.equal(result.status, 0);
    assert.ok(
      result.trace.filter((row) => row.kind === "cache").every((row) => row.lookup === true),
    );
    const artifact = readFileSync(join(fixture.cwd, "artifact.txt"), "utf8");
    assert.equal(
      result.trace.find((row) => row.kind === "cache-post" && row.name === "target").saved,
      `compiled:${artifact}`,
    );
  });
});

test("actual mount probes preserve sticky clones and restore only failed paths", () => {
  for (const failures of [[], ["target"], ["registry", "git", "target", "secondary"]]) {
    cacheFixture((fixture) => {
      const context = {
        ...fixture.context(),
        secondaryRole: "docs-example",
        secondarySuffix: "Linux-X64",
        secondaryPath: "secondary",
      };
      const result = executeCacheAction(fixture, context, { mountFailures: failures, hit: true });
      assert.equal(result.status, 0);
      assert.equal(result.trace.filter((row) => row.kind === "sticky").length, 4);
      for (const row of result.trace.filter((row) => row.kind === "cache")) {
        assert.equal(row.lookup, !failures.includes(row.name), row.name);
      }
      assert.equal(result.trace.filter((row) => row.kind === "workload").length, 1);
    });
  }
});

test("Actions hits still compile current input instead of accepting the restored artifact", () => {
  cacheFixture((fixture) => {
    const source = "changed input after cached compilation\n";
    writeFileSync(join(fixture.cwd, "source.txt"), source);
    const result = executeCacheAction(
      fixture,
      fixture.context("pull_request", "refs/pull/9/merge"),
      { hit: true },
    );
    assert.equal(result.status, 0);
    const workload = result.trace.find((row) => row.kind === "workload");
    assert.equal(workload.before, "Actions restore");
    assert.equal(workload.artifact, createHash("sha256").update(source).digest("hex"));
    assert.equal(readFileSync(join(fixture.cwd, "artifact.txt"), "utf8"), workload.artifact);
  });
});

test("manual dispatch validates the checked-out commit and actual default branch", () => {
  cacheFixture((fixture) => {
    const context = fixture.context("workflow_dispatch", "refs/heads/main");
    context.event.repository.default_branch = "trunk";
    let result = executeCacheAction(fixture, context);
    assert.equal(result.status, 0);
    assert.equal(result.outputs["cache-policy"].sticky, "false");
  });
  cacheFixture((fixture) => {
    const context = fixture.context("workflow_dispatch");
    writeFileSync(join(fixture.cwd, "Cargo.lock"), "another checkout\n");
    fixture.git("add", "Cargo.lock");
    fixture.git(
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.test",
      "commit",
      "--quiet",
      "-m",
      "other checkout",
    );
    const result = executeCacheAction(fixture, context);
    assert.equal(result.status, 0);
    assert.equal(result.trace.filter((row) => row.kind === "sticky").length, 0);
  });
});

test("unknown or malformed events stop before any provider or cache child", () => {
  for (const change of [
    { eventName: "invented" },
    { event: null },
    { event: {} },
    { sourceSha: "short" },
    { ref: "main" },
    { runnerEnvironment: "" },
    { runnerOs: "" },
    { runnerArch: "" },
    { suffix: "bad\ncache=true" },
    { role: "x".repeat(129) },
    { repository: "wrong/repo" },
    { role: "target\nmalicious=true" },
    {
      event: {
        repository: { full_name: "example/repo", default_branch: "main" },
        ref: "refs/heads/topic",
        deleted: false,
      },
    },
  ])
    cacheFixture((fixture) => {
      const result = executeCacheAction(fixture, { ...fixture.context(), ...change });
      assert.notEqual(result.status, 0);
      assert.deepEqual(result.trace, []);
    });
});

test("target metadata cannot alias or escape the checkout", () => {
  cacheFixture((fixture) => {
    for (const change of [
      { targetPath: "../outside" },
      { targetPath: "." },
      { secondaryPath: "target", secondaryRole: "second", secondarySuffix: "Linux-X64" },
      { secondaryPath: "target/child", secondaryRole: "second", secondarySuffix: "Linux-X64" },
      {
        targetPath: "target/child",
        secondaryPath: "target",
        secondaryRole: "second",
        secondarySuffix: "Linux-X64",
      },
      { secondaryRole: "second" },
      { secondaryPath: "second" },
    ])
      assert.throws(() =>
        rustCachePolicy({ ...fixture.context(), ...change }, { cwd: fixture.cwd }),
      );
  });
});
