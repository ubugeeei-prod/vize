import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { rustCacheMountState } from "../../.github/actions/setup-rust-sticky-cache/cache-policy.ts";
import { cacheFixture, executeCacheAction } from "./support/rust-cache-fixture.mjs";
import {
  allMountFailures,
  assertNoCandidateCommit,
  assertUntrustedStickyRows,
  isCandidate,
} from "./support/rust-cache-candidates.ts";

const action = parse(
  readFileSync(
    new URL("../../.github/actions/setup-rust-sticky-cache/action.yml", import.meta.url),
    "utf8",
  ),
);
const original = structuredClone(action);
const primarySave = original.runs.steps.find(
  ({ name }) => name === "Cache Rust target or seed it from the trusted sticky clone",
);
const primaryRestore = original.runs.steps.find(
  ({ name }) => name === "Restore a Rust target without registering a save",
);
primarySave.if =
  "${{ steps.cache-policy.outputs.target-save == 'true' && (inputs.key != 'test-scripts' || steps.cache-mounts.outputs.target != 'true') }}";
primaryRestore.if =
  "${{ steps.cache-policy.outputs.target-save == 'false' || (inputs.key == 'test-scripts' && steps.cache-mounts.outputs.target == 'true') }}";
primaryRestore.with.key = "${{ steps.cache-policy.outputs.target-key }}";
const save = original.runs.steps.find(
  ({ name }) => name === "Cache the secondary Rust target or seed its trusted clone",
);
const restore = original.runs.steps.find(
  ({ name }) => name === "Restore a secondary target without registering a save",
);
save.if =
  "${{ steps.cache-policy.outputs.secondary-save == 'true' && steps.cache-policy.outputs.secondary == 'true' }}";
restore.if =
  "${{ steps.cache-policy.outputs.secondary-save == 'false' && steps.cache-policy.outputs.secondary == 'true' }}";
restore.with.key = "${{ steps.cache-policy.outputs.secondary-key }}";
const paths = {
  role: "docs-wasm",
  secondaryRole: "docs-example",
  secondaryPath: ".docs-example-target",
  secondarySuffix: "Linux-X64",
};
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

function secondary(result) {
  const caches = result.trace.filter(({ kind, name }) => kind === "cache" && name === "secondary");
  assert.equal(caches.length, 1);
  return caches[0];
}

function cacheIdentity(row) {
  const { pid, ...identity } = row;
  assert.ok(Number.isSafeInteger(pid) && pid > 0);
  return identity;
}

function currentWorkloads(fixture, result) {
  assert.equal(result.status, 0);
  const expected = hash(readFileSync(join(fixture.cwd, "source.txt")));
  const workloads = result.trace.filter(({ kind }) => kind === "workload");
  assert.deepEqual(
    workloads.map(({ name, artifact }) => ({ name, artifact })),
    [
      { name: "target", artifact: expected },
      { name: "secondary", artifact: expected },
    ],
  );
  for (const workload of workloads)
    assert.equal(
      readFileSync(join(workload.path, "cache-artifact.txt"), "utf8"),
      `compiled:${expected}`,
    );
  const posts = result.trace.filter(({ kind }) => kind === "sticky-post");
  for (const post of posts)
    assert.ok(
      workloads.every((workload) => result.trace.indexOf(workload) < result.trace.indexOf(post)),
    );
  return workloads;
}

await test("docs examples use separate primary and secondary target directories", () => {
  const docs = parse(
    readFileSync(new URL("../../.github/workflows/build-docs.yml", import.meta.url), "utf8"),
  );
  const caller = docs.jobs["build-playground"].steps.filter(
    ({ uses }) => uses === "./.github/actions/setup-rust-sticky-cache",
  );
  assert.deepEqual(
    caller.map(({ with: inputs }) => inputs),
    [
      {
        key: "docs-wasm",
        "cache-key-suffix": "${{ runner.os }}-${{ runner.arch }}",
        "secondary-key": "docs-example",
        "secondary-target-path": ".docs-example-target",
        "secondary-cache-key-suffix": "${{ runner.os }}-${{ runner.arch }}",
      },
    ],
  );
  const scripts = JSON.parse(
    readFileSync(new URL("../../examples/vite-musea/package.json", import.meta.url)),
  ).scripts;
  assert.equal(scripts.build, "pnpm check && vp build --config vite.app.config.ts");
  assert.equal(
    scripts.check,
    "cargo run --quiet --target-dir ../../.docs-example-target --manifest-path ../../Cargo.toml -p vize -- check && vp check src vite.config.ts vite.app.config.ts vite.ui-gallery.config.ts vize.config.ts vite.globals.config.ts musea.preview.ts playwright.config.ts",
  );
});

await test("mounted trusted docs examples keep the same lookup and both current workloads without an archive post", () => {
  for (const event of ["push", "schedule", "workflow_dispatch"])
    for (const hit of [false, "partial", true])
      cacheFixture((fixture) => {
        const context = { ...fixture.context(event), ...paths };
        const options = { hit, nestedPost: true, secondaryWorkload: true };
        const old = executeCacheAction(fixture, context, {
          ...options,
          steps: original.runs.steps,
        });
        currentWorkloads(fixture, old);
        assert.equal(
          old.trace.filter(({ kind, name }) => kind === "cache-post" && name === "secondary")
            .length,
          1,
        );
        const result = executeCacheAction(fixture, context, options);
        currentWorkloads(fixture, result);
        const before = secondary(old),
          after = secondary(result);
        assert.equal(before.restoreOnly, false);
        assert.equal(after.restoreOnly, true);
        for (const key of ["key", "path", "lookup", "version", "restorePrefix", "hit"])
          assert.equal(after[key], before[key], key);
        assert.equal(after.lookup, true);
        assert.equal(after.key, result.outputs["cache-policy"]["secondary-seed-key"]);
        assert.deepEqual(
          result.trace.filter(({ kind }) => kind === "cache-post").map(({ name }) => name),
          ["git", "registry"],
        );
        assert.equal(result.trace.filter(({ kind }) => kind === "sticky-post").length, 4);
        assert.throws(
          () =>
            assert.equal(
              old.trace.filter(({ kind, name }) => kind === "cache-post" && name === "secondary")
                .length,
              0,
            ),
          assert.AssertionError,
        );
      });
});

await test("failed secondary mounts and GitHub-hosted docs retain original restores and saves", () => {
  for (const event of ["push", "schedule", "workflow_dispatch"])
    for (const mode of ["failed-mount", "github-hosted"])
      cacheFixture((fixture) => {
        const context = {
          ...fixture.context(event),
          ...paths,
          runnerEnvironment: mode === "github-hosted" ? "github-hosted" : "self-hosted",
        };
        const options = {
          hit: "partial",
          nestedPost: true,
          secondaryWorkload: true,
          mountFailures: mode === "failed-mount" ? ["secondary"] : [],
        };
        const old = executeCacheAction(fixture, context, {
          ...options,
          steps: original.runs.steps,
        });
        const result = executeCacheAction(fixture, context, options);
        currentWorkloads(fixture, result);
        assert.deepEqual(cacheIdentity(secondary(result)), cacheIdentity(secondary(old)));
        assert.equal(secondary(result).lookup, false);
        assert.equal(secondary(result).restoreOnly, false);
        assert.equal(secondary(result).key, result.outputs["cache-policy"]["secondary-key"]);
        const saved = result.trace.filter(
          ({ kind, name }) => kind === "cache-post" && name === "secondary",
        );
        assert.equal(saved.length, 1);
        assert.equal(
          saved[0].saved,
          `compiled:${hash(readFileSync(join(fixture.cwd, "source.txt")))}`,
        );
      });
});

await test("PRs, merge groups and untrusted pushes retain exact restore-only behavior", () => {
  for (const [event, ref] of [
    ["pull_request", "refs/pull/37/merge"],
    ["merge_group", "refs/heads/gh-readonly-queue/main/pr-1-a"],
    ["push", "refs/heads/feature"],
  ])
    cacheFixture((fixture) => {
      const context = { ...fixture.context(event, ref), ...paths };
      // Candidates fall back to these restores only when their read-only mounts fail.
      const mountFailures = isCandidate(event) ? allMountFailures : [];
      const options = { hit: "partial", nestedPost: true, secondaryWorkload: true, mountFailures };
      const old = executeCacheAction(fixture, context, { ...options, steps: original.runs.steps });
      const result = executeCacheAction(fixture, context, options);
      currentWorkloads(fixture, result);
      assert.deepEqual(cacheIdentity(secondary(result)), cacheIdentity(secondary(old)));
      assert.equal(secondary(result).restoreOnly, true);
      assert.equal(secondary(result).lookup, false);
      assert.equal(result.trace.filter(({ kind }) => kind === "cache-post").length, 0);
      assertUntrustedStickyRows(result.trace, event);
    });
});

await test("a compatible prior example seed still restores and never replaces current source checks", () => {
  cacheFixture((fixture) => {
    const context = { ...fixture.context(), ...paths };
    const options = { nestedPost: true, secondaryWorkload: true, storedCache: true };
    const seeded = executeCacheAction(fixture, context, { ...options, steps: original.runs.steps });
    const saved = seeded.trace.find(
      ({ kind, name }) => kind === "cache-post" && name === "secondary",
    );
    writeFileSync(
      join(fixture.cwd, "source.txt"),
      "current source differs from the compatible old example seed\n",
    );
    const result = executeCacheAction(
      fixture,
      { ...fixture.context("pull_request", "refs/pull/37/merge"), ...paths },
      { ...options, mountFailures: allMountFailures },
    );
    const workloads = currentWorkloads(fixture, result);
    assert.equal(secondary(result).matchedKey, saved.key);
    assert.equal(secondary(result).restoredArtifact, saved.saved);
    assert.equal(workloads[1].before, "Actions restore");
    assert.notEqual(`compiled:${workloads[1].artifact}`, saved.saved);
    assert.equal(result.trace.filter(({ kind }) => kind === "cache-post").length, 0);
    assertNoCandidateCommit(result.trace);
  });
});

await test("all mounted secondary roles omit their post independently of a failed primary", () => {
  for (const [role, secondaryRole, failures] of [
    ["docs-wasm-extra", "docs-example", []],
    ["docs-wasm", "docs-example-extra", []],
    ["check-vize-apps", "docs-example", []],
    ["docs-wasm", "docs-example", ["target"]],
  ])
    cacheFixture((fixture) => {
      const context = { ...fixture.context(), ...paths, role, secondaryRole };
      const options = { nestedPost: true, secondaryWorkload: true, mountFailures: failures };
      const result = executeCacheAction(fixture, context, options);
      currentWorkloads(fixture, result);
      assert.equal(secondary(result).restoreOnly, true);
      assert.equal(
        result.trace.filter(({ kind, name }) => kind === "cache-post" && name === "secondary")
          .length,
        0,
      );
      assert.equal(
        result.trace.filter(({ kind, name }) => kind === "cache-post" && name === "target").length,
        failures.includes("target") ? 1 : 0,
      );
    });
});

await test("docs save omission retains malformed source and path refusals before any workload", () => {
  for (const changes of [
    { secondaryPath: "../outside" },
    { eventName: "unknown" },
    { secondaryRole: "docs-example\nforged=true" },
  ])
    cacheFixture((fixture) => {
      const result = executeCacheAction(
        fixture,
        { ...fixture.context(), ...paths, ...changes },
        { secondaryWorkload: true },
      );
      assert.notEqual(result.status, 0);
      assert.deepEqual(result.trace, []);
    });
});

await test("a genuine secondary subprocess failure remains a failure with either cache policy", () => {
  for (const steps of [original.runs.steps, action.runs.steps])
    cacheFixture((fixture) => {
      const backend = join(fixture.cwd, "backend.mjs");
      const source = readFileSync(backend, "utf8");
      const branch = 'else if (request.kind === "workload") {';
      assert.equal(source.split(branch).length, 2);
      writeFileSync(
        backend,
        source.replace(branch, `${branch}\nif(request.name === "secondary") process.exit(23);`),
      );
      assert.throws(
        () =>
          executeCacheAction(
            fixture,
            { ...fixture.context(), ...paths },
            {
              steps,
              secondaryWorkload: true,
              nestedPost: true,
            },
          ),
        (error) => {
          assert.equal(error.actual, 23);
          return true;
        },
      );
      const trace = readFileSync(join(fixture.cwd, "trace.jsonl"), "utf8")
        .trim()
        .split("\n")
        .map((row) => JSON.parse(row));
      assert.deepEqual(
        trace.filter(({ kind }) => kind === "workload").map(({ name }) => name),
        ["target"],
      );
      assert.equal(trace.filter(({ kind }) => kind === "cache-post").length, 0);
    });
});

await test(
  "Linux mount admission distinguishes a real mounted filesystem from an ordinary directory",
  {
    skip: process.platform !== "linux" && "actual mountpoint command runs on Linux Actions",
  },
  () => {
    cacheFixture((fixture) => {
      const states = rustCacheMountState(
        { target: "/proc", secondary: fixture.cwd },
        { home: fixture.cwd },
      );
      assert.equal(states.target, "true");
      assert.equal(states.secondary, "false");
    });
  },
);
