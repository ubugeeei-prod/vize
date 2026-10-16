import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { rustCachePolicy } from "../../.github/actions/setup-rust-sticky-cache/cache-policy.ts";
import { cacheFixture, executeCacheAction } from "./support/rust-cache-fixture.mjs";
import { allMountFailures, assertNoCandidateCommit } from "./support/rust-cache-candidates.ts";

const steps = parse(
  readFileSync(
    new URL("../../.github/actions/setup-rust-sticky-cache/action.yml", import.meta.url),
    "utf8",
  ),
).runs.steps;
const paths = {
  role: "check-vize-apps",
  secondaryRole: "benchmark-base",
  secondarySuffix: "Linux-X64",
  secondaryPath: "base/target",
};

function advanceSource(fixture) {
  const source = "new source after the original trusted seed\n";
  writeFileSync(join(fixture.cwd, "source.txt"), source);
  fixture.git("add", "source.txt");
  fixture.git(
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.test",
    "commit",
    "--quiet",
    "-m",
    "advance source",
  );
  const sha = fixture.git("rev-parse", "HEAD");
  return { source, sourceSha: sha, checkoutSha: sha };
}

function savedTargets(result) {
  return result.trace.filter(
    (row) =>
      row.kind === "cache-post" && ["target", "secondary"].includes(row.name) && row.saved !== null,
  );
}

function archivalSteps() {
  const original = structuredClone(steps);
  for (const step of original) {
    if (step.name === "Cache Rust target or seed it from the trusted sticky clone")
      step.if = "${{ steps.cache-policy.outputs.target-save == 'true' }}";
    if (step.name === "Cache the secondary Rust target or seed its trusted clone")
      step.if =
        "${{ steps.cache-policy.outputs.secondary-save == 'true' && steps.cache-policy.outputs.secondary == 'true' }}";
    if (step.name === "Restore a Rust target without registering a save")
      step.if = "${{ steps.cache-policy.outputs.target-save == 'false' }}";
    if (step.name === "Restore a secondary target without registering a save")
      step.if =
        "${{ steps.cache-policy.outputs.secondary-save == 'false' && steps.cache-policy.outputs.secondary == 'true' }}";
  }
  return original;
}

await test("mounted targets never archive across source commits while the original keys archive again", () => {
  const original = archivalSteps();
  for (const step of original) {
    if (step.name === "Cache Rust target or seed it from the trusted sticky clone")
      step.with.key = "${{ steps.cache-policy.outputs.target-key }}";
    if (step.name === "Cache the secondary Rust target or seed its trusted clone")
      step.with.key = "${{ steps.cache-policy.outputs.secondary-key }}";
  }
  for (const actionSteps of [original, steps])
    cacheFixture((fixture) => {
      const context = { ...fixture.context(), ...paths };
      const options = { nestedPost: true, storedCache: true, steps: actionSteps };
      const first = executeCacheAction(fixture, context, options);
      assert.equal(first.status, 0);
      assert.equal(savedTargets(first).length, actionSteps === original ? 2 : 0);
      const stored = readFileSync(join(fixture.cwd, "stored-cache.json"), "utf8");
      const changed = advanceSource(fixture);
      const second = executeCacheAction(fixture, { ...context, ...changed }, options);
      assert.equal(second.status, 0);
      const workload = second.trace.find((row) => row.kind === "workload");
      assert.equal(workload.artifact, createHash("sha256").update(changed.source).digest("hex"));
      assert.notEqual(
        workload.artifact,
        first.trace.find((row) => row.kind === "workload").artifact,
      );
      assert.equal(workload.before, "sticky clone");
      assert.ok(second.trace.filter((row) => row.kind === "cache").every((row) => row.lookup));
      const targets = second.trace.filter(
        (row) => row.kind === "cache" && ["target", "secondary"].includes(row.name),
      );
      if (actionSteps === original) {
        assert.equal(savedTargets(second).length, 2);
        assert.ok(targets.every((row) => row.hit === "partial"));
      } else {
        assert.equal(savedTargets(second).length, 0);
        assert.ok(targets.every((row) => row.restoreOnly && row.lookup));
        assert.equal(readFileSync(join(fixture.cwd, "stored-cache.json"), "utf8"), stored);
      }
      const unmount = second.trace.findIndex((row) => row.kind === "sticky-post");
      assert.ok(second.trace.indexOf(workload) < unmount);
      assert.equal(second.trace.filter((row) => row.kind === "sticky-post").length, 4);
    });
});

await test("a mounted stable seed remains a compatible restore for untrusted current input", () => {
  cacheFixture((fixture) => {
    const options = { nestedPost: true, storedCache: true };
    const seeded = executeCacheAction(
      fixture,
      { ...fixture.context(), ...paths },
      { ...options, steps: archivalSteps() },
    );
    const seeds = savedTargets(seeded);
    const changed = advanceSource(fixture);
    for (const path of ["target", paths.secondaryPath])
      rmSync(join(fixture.cwd, path), { recursive: true, force: true });
    const result = executeCacheAction(
      fixture,
      {
        ...fixture.context("pull_request", "refs/pull/37/merge"),
        ...paths,
        ...changed,
      },
      { ...options, mountFailures: allMountFailures },
    );
    assert.equal(result.status, 0);
    assert.equal(result.trace.filter((row) => row.kind === "cache-post").length, 0);
    assertNoCandidateCommit(result.trace);
    for (const seed of seeds) {
      const restored = result.trace.find((row) => row.kind === "cache" && row.name === seed.name);
      assert.equal(restored.matchedKey, seed.key);
      assert.equal(restored.restoredArtifact, seed.saved);
      assert.equal(restored.version, seed.version);
      assert.equal(restored.restoreOnly, true);
      assert.equal(restored.lookup, false);
      assert.equal(restored.hit, "partial");
    }
    const workload = result.trace.find((row) => row.kind === "workload");
    assert.equal(workload.before, "Actions restore");
    assert.equal(workload.artifact, createHash("sha256").update(changed.source).digest("hex"));
  });
});

await test("an old per-SHA seed remains lookup-only without migration archives", () => {
  cacheFixture((fixture) => {
    const original = archivalSteps();
    for (const step of original) {
      if (step.name === "Cache Rust target or seed it from the trusted sticky clone")
        step.with.key = "${{ steps.cache-policy.outputs.target-key }}";
      if (step.name === "Cache the secondary Rust target or seed its trusted clone")
        step.with.key = "${{ steps.cache-policy.outputs.secondary-key }}";
    }
    const context = { ...fixture.context(), ...paths };
    const options = { nestedPost: true, storedCache: true };
    const old = executeCacheAction(fixture, context, { ...options, steps: original });
    assert.equal(savedTargets(old).length, 2);
    const migrated = executeCacheAction(fixture, context, options);
    assert.equal(migrated.status, 0);
    assert.equal(savedTargets(migrated).length, 0);
    const stable = executeCacheAction(fixture, context, options);
    assert.equal(stable.status, 0);
    assert.equal(savedTargets(stable).length, 0);
    assert.ok(
      stable.trace
        .filter((row) => row.kind === "cache" && ["target", "secondary"].includes(row.name))
        .every((row) => row.restoreOnly && row.lookup && row.hit === "partial"),
    );
    assert.equal(stable.trace.filter((row) => row.kind === "workload").length, 1);
  });
});

await test("nonmounted trusted targets retain exact-source saves independently of a mounted sibling", () => {
  for (const mode of ["github-hosted", "primary-failed", "secondary-failed"])
    cacheFixture((fixture) => {
      const context = {
        ...fixture.context(),
        ...paths,
        runnerEnvironment: mode === "github-hosted" ? "github-hosted" : "self-hosted",
      };
      const options = {
        nestedPost: true,
        storedCache: true,
        mountFailures:
          mode === "primary-failed" ? ["target"] : mode === "secondary-failed" ? ["secondary"] : [],
      };
      mkdirSync(join(fixture.cwd, paths.secondaryPath), { recursive: true });
      writeFileSync(
        join(fixture.cwd, paths.secondaryPath, "cache-artifact.txt"),
        "compiled:secondary workload\n",
      );
      const first = executeCacheAction(fixture, context, options);
      const changed = advanceSource(fixture);
      const second = executeCacheAction(fixture, { ...context, ...changed }, options);
      assert.equal(first.status, 0);
      assert.equal(second.status, 0);
      for (const name of ["target", "secondary"]) {
        const before = first.trace.find((row) => row.kind === "cache" && row.name === name);
        const after = second.trace.find((row) => row.kind === "cache" && row.name === name);
        const post = second.trace.find((row) => row.kind === "cache-post" && row.name === name);
        const mounted = mode !== "github-hosted" && !options.mountFailures.includes(name);
        assert.equal(after.lookup, mounted);
        if (mounted) {
          assert.equal(after.key, before.key);
          assert.equal(after.restoreOnly, true);
          assert.equal(post, undefined);
        } else {
          assert.notEqual(after.key, before.key);
          assert.ok(after.key.endsWith(changed.checkoutSha));
          assert.equal(after.hit, "partial");
          assert.ok(post.saved !== null);
        }
      }
      assert.equal(second.trace.filter((row) => row.kind === "workload").length, 1);
    });
});

await test("stable seed identities retain lock, toolchain, runner and primary/secondary role isolation", () => {
  cacheFixture((fixture) => {
    mkdirSync(join(fixture.cwd, "base"));
    writeFileSync(join(fixture.cwd, "base/Cargo.lock"), "independent benchmark lock\n");
    const context = { ...fixture.context(), ...paths };
    const first = rustCachePolicy(context, { cwd: fixture.cwd });
    const advanced = rustCachePolicy(
      { ...context, ...advanceSource(fixture) },
      { cwd: fixture.cwd },
    );
    for (const name of ["target", "secondary"]) {
      assert.equal(advanced[`${name}-seed-key`], first[`${name}-seed-key`]);
      assert.ok(first[`${name}-seed-key`].startsWith(first[`${name}-restore`]));
    }
    for (const [field, value] of [
      ["role", "build-js-packages"],
      ["suffix", "Linux-ARM64"],
      ["runnerOs", "Windows"],
      ["runnerArch", "ARM64"],
    ])
      assert.notEqual(
        rustCachePolicy({ ...context, [field]: value }, { cwd: fixture.cwd })["target-seed-key"],
        first["target-seed-key"],
      );
    assert.notEqual(
      rustCachePolicy({ ...context, secondaryRole: "other" }, { cwd: fixture.cwd })[
        "secondary-seed-key"
      ],
      first["secondary-seed-key"],
    );
    writeFileSync(join(fixture.cwd, "base/Cargo.lock"), "changed benchmark lock\n");
    const secondaryChanged = rustCachePolicy(context, { cwd: fixture.cwd });
    assert.equal(secondaryChanged["target-seed-key"], first["target-seed-key"]);
    assert.notEqual(secondaryChanged["secondary-seed-key"], first["secondary-seed-key"]);
    for (const file of ["Cargo.lock", "rust-toolchain.toml"]) {
      writeFileSync(join(fixture.cwd, file), "changed compatible inputs\n");
      assert.notEqual(
        rustCachePolicy(context, { cwd: fixture.cwd })["target-seed-key"],
        first["target-seed-key"],
      );
    }
  });
});
