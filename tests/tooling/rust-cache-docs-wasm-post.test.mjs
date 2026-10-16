import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
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
// Keep the original secondary policy as the before-frame for these primary laws.
const secondarySave = original.runs.steps.find(
  ({ name }) => name === "Cache the secondary Rust target or seed its trusted clone",
);
const secondaryRestore = original.runs.steps.find(
  ({ name }) => name === "Restore a secondary target without registering a save",
);
secondarySave.if =
  "${{ steps.cache-policy.outputs.secondary-save == 'true' && steps.cache-policy.outputs.secondary == 'true' && (inputs.key != 'docs-wasm' || inputs.secondary-key != 'docs-example' || steps.cache-mounts.outputs.secondary != 'true') }}";
secondaryRestore.if =
  "${{ (steps.cache-policy.outputs.secondary-save == 'false' || (inputs.key == 'docs-wasm' && inputs.secondary-key == 'docs-example' && steps.cache-mounts.outputs.secondary == 'true')) && steps.cache-policy.outputs.secondary == 'true' }}";
secondaryRestore.with.key =
  "${{ inputs.key == 'docs-wasm' && inputs.secondary-key == 'docs-example' && steps.cache-mounts.outputs.secondary == 'true' && steps.cache-policy.outputs.secondary-seed-key || steps.cache-policy.outputs.secondary-key }}";
const save = original.runs.steps.find(
  ({ name }) => name === "Cache Rust target or seed it from the trusted sticky clone",
);
const restore = original.runs.steps.find(
  ({ name }) => name === "Restore a Rust target without registering a save",
);
save.if =
  "${{ steps.cache-policy.outputs.target-save == 'true' && (inputs.key != 'test-scripts' || steps.cache-mounts.outputs.target != 'true') }}";
restore.if =
  "${{ steps.cache-policy.outputs.target-save == 'false' || (inputs.key == 'test-scripts' && steps.cache-mounts.outputs.target == 'true') }}";
restore.with.key = "${{ steps.cache-policy.outputs.target-key }}";
const paths = {
  role: "docs-wasm",
  secondaryRole: "docs-example",
  secondaryPath: ".docs-example-target",
  secondarySuffix: "Linux-X64",
};
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const posts = (result, name) =>
  result.trace.filter((row) => row.kind === "cache-post" && row.name === name);

function target(result) {
  const rows = result.trace.filter(({ kind, name }) => kind === "cache" && name === "target");
  assert.equal(rows.length, 1);
  return rows[0];
}

function identity({ pid, ...row }) {
  assert.ok(Number.isSafeInteger(pid) && pid > 0);
  return row;
}

function workloads(fixture, result) {
  assert.equal(result.status, 0);
  const expected = hash(readFileSync(join(fixture.cwd, "source.txt")));
  const rows = result.trace.filter(({ kind }) => kind === "workload");
  assert.deepEqual(
    rows.map(({ name, artifact }) => ({ name, artifact })),
    [
      { name: "target", artifact: expected },
      { name: "secondary", artifact: expected },
    ],
  );
  for (const row of rows)
    assert.equal(
      readFileSync(join(row.path, "cache-artifact.txt"), "utf8"),
      `compiled:${expected}`,
    );
  for (const post of result.trace.filter(({ kind }) => kind === "sticky-post"))
    assert.ok(rows.every((row) => result.trace.indexOf(row) < result.trace.indexOf(post)));
  return rows;
}

await test("mounted docs-wasm avoids a duplicate primary archive and builds current source", () => {
  for (const event of ["push", "schedule", "workflow_dispatch"])
    for (const hit of [false, "partial", true])
      for (const role of ["docs-wasm", "Docs-Wasm"])
        cacheFixture((fixture) => {
          const context = { ...fixture.context(event), ...paths, role };
          const options = { hit, nestedPost: true, secondaryWorkload: true };
          const before = executeCacheAction(fixture, context, {
            ...options,
            steps: original.runs.steps,
          });
          workloads(fixture, before);
          assert.equal(posts(before, "target").length, 1);
          const after = executeCacheAction(fixture, context, options);
          workloads(fixture, after);
          assert.equal(target(after).restoreOnly, true);
          assert.equal(posts(after, "target").length, 0);
          for (const key of ["key", "path", "lookup", "version", "restorePrefix", "hit"])
            assert.equal(target(after)[key], target(before)[key], key);
          assert.equal(target(after).key, after.outputs["cache-policy"]["target-seed-key"]);
          assert.deepEqual(
            after.trace.filter(({ kind }) => kind === "cache-post").map(({ name }) => name),
            ["git", "registry"],
          );
          assert.equal(after.trace.filter(({ kind }) => kind === "sticky-post").length, 4);
        });
});

await test("unmounted primary and hosted runners retain exact original primary restores and saves", () => {
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
          mountFailures: mode === "failed-mount" ? ["target"] : [],
        };
        const before = executeCacheAction(fixture, context, {
          ...options,
          steps: original.runs.steps,
        });
        const after = executeCacheAction(fixture, context, options);
        workloads(fixture, before);
        workloads(fixture, after);
        assert.deepEqual(identity(target(after)), identity(target(before)));
        assert.equal(target(after).lookup, false);
        assert.equal(target(after).restoreOnly, false);
        assert.equal(target(after).key, after.outputs["cache-policy"]["target-key"]);
        assert.equal(posts(after, "target").length, 1);
        assert.equal(
          posts(after, "target")[0].saved,
          `compiled:${hash(readFileSync(join(fixture.cwd, "source.txt")))}`,
        );
      });
});

await test("PR, merge-group, foreign source and other untrusted events retain original restores", () => {
  for (const [event, ref, deleted, foreignSource] of [
    ["pull_request", "refs/pull/37/merge", false],
    ["merge_group", "refs/heads/gh-readonly-queue/main/pr-1-a", false],
    ["push", "refs/heads/feature", false],
    ["workflow_dispatch", "refs/heads/feature", false],
    ["push", "refs/heads/main", true],
    ["push", "refs/heads/main", false, "0".repeat(40)],
  ])
    cacheFixture((fixture) => {
      const context = { ...fixture.context(event, ref), ...paths };
      if (foreignSource) context.sourceSha = foreignSource;
      context.event.deleted = deleted;
      // Candidates fall back to these restores only when their read-only mounts fail.
      const mountFailures = isCandidate(event) ? allMountFailures : [];
      const options = { hit: "partial", nestedPost: true, secondaryWorkload: true, mountFailures };
      const before = executeCacheAction(fixture, context, {
        ...options,
        steps: original.runs.steps,
      });
      const after = executeCacheAction(fixture, context, options);
      workloads(fixture, before);
      workloads(fixture, after);
      assert.deepEqual(identity(target(after)), identity(target(before)));
      assert.equal(target(after).restoreOnly, true);
      assert.equal(target(after).lookup, false);
      assert.equal(after.trace.filter(({ kind }) => kind === "cache-post").length, 0);
      assertUntrustedStickyRows(after.trace, event);
    });
});

await test("every other mounted role omits its primary post independently of the secondary", () => {
  for (const role of [
    "docs-napi",
    "docs-wasm-extra",
    "new-role",
    "check-vize-apps",
    "test-scripts",
  ])
    cacheFixture((fixture) => {
      const context = { ...fixture.context(), ...paths, role, secondaryRole: "docs-wasm" };
      const options = { nestedPost: true, secondaryWorkload: true };
      const before = executeCacheAction(fixture, context, {
        ...options,
        steps: original.runs.steps,
      });
      const after = executeCacheAction(fixture, context, options);
      workloads(fixture, before);
      workloads(fixture, after);
      for (const key of ["path", "lookup", "version", "restorePrefix", "hit"])
        assert.equal(target(after)[key], target(before)[key], key);
      assert.equal(target(after).restoreOnly, true);
      assert.equal(target(after).key, after.outputs["cache-policy"]["target-seed-key"]);
      assert.equal(posts(after, "target").length, 0);
    });
  cacheFixture((fixture) => {
    const result = executeCacheAction(
      fixture,
      { ...fixture.context(), ...paths },
      {
        nestedPost: true,
        secondaryWorkload: true,
        mountFailures: ["secondary"],
      },
    );
    workloads(fixture, result);
    assert.equal(target(result).restoreOnly, true);
    assert.equal(posts(result, "target").length, 0);
    assert.equal(posts(result, "secondary").length, 1);
  });
});

await test("old compatible primary seeds restore without accepting stale source or overwriting mounted clones", () => {
  for (const mode of ["pull_request", "read-only", "failed-mount", "mounted"])
    cacheFixture((fixture) => {
      const options = { nestedPost: true, secondaryWorkload: true, storedCache: true };
      const seeded = executeCacheAction(
        fixture,
        { ...fixture.context(), ...paths },
        {
          ...options,
          steps: original.runs.steps,
        },
      );
      const saved = posts(seeded, "target")[0];
      assert.equal(saved.key, seeded.outputs["cache-policy"]["target-seed-key"]);
      const stored = readFileSync(join(fixture.cwd, "stored-cache.json"));
      writeFileSync(
        join(fixture.cwd, "source.txt"),
        "genuine new source after the original primary seed\n",
      );
      fixture.git("add", "source.txt");
      fixture.git(
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.test",
        "commit",
        "--quiet",
        "-m",
        "new source",
      );
      const sha = fixture.git("--no-replace-objects", "rev-parse", "HEAD");
      const candidate = mode === "pull_request" || mode === "read-only";
      const base = fixture.context(
        candidate ? "pull_request" : "push",
        candidate ? "refs/pull/37/merge" : "refs/heads/main",
      );
      const context = { ...base, ...paths, sourceSha: sha, checkoutSha: sha };
      const result = executeCacheAction(fixture, context, {
        ...options,
        mountFailures:
          mode === "failed-mount" ? ["target"] : mode === "pull_request" ? allMountFailures : [],
      });
      const rows = workloads(fixture, result);
      assert.equal(target(result).matchedKey, saved.key);
      assert.notEqual(`compiled:${rows[0].artifact}`, saved.saved);
      if (candidate) assertNoCandidateCommit(result.trace);
      if (mode === "mounted" || mode === "read-only") {
        assert.equal(target(result).restoredArtifact, undefined);
        assert.equal(rows[0].before, "sticky clone");
        assert.equal(posts(result, "target").length, 0);
        assert.deepEqual(readFileSync(join(fixture.cwd, "stored-cache.json")), stored);
      } else {
        assert.equal(target(result).restoredArtifact, saved.saved);
        assert.equal(rows[0].before, "Actions restore");
        assert.equal(posts(result, "target").length, mode === "failed-mount" ? 1 : 0);
      }
    });
});

await test("malformed metadata and input refuse before any workload with both policies", () => {
  for (const changes of [
    { targetPath: "../outside" },
    { targetPath: "." },
    { targetPath: ".docs-example-target" },
    { role: "docs-wasm\nforged=true" },
    { suffix: "bad/suffix" },
    { eventName: "unknown" },
    { sourceSha: "bad" },
  ])
    cacheFixture((fixture) => {
      for (const steps of [original.runs.steps, action.runs.steps]) {
        const result = executeCacheAction(
          fixture,
          { ...fixture.context(), ...paths, ...changes },
          { steps, secondaryWorkload: true },
        );
        assert.notEqual(result.status, 0, JSON.stringify(changes));
        assert.deepEqual(result.trace, []);
      }
    });
});

await test("a real failed primary source subprocess retains exit23 and registers no successful post", () => {
  for (const steps of [original.runs.steps, action.runs.steps])
    cacheFixture((fixture) => {
      const backend = join(fixture.cwd, "backend.mjs");
      const body = readFileSync(backend, "utf8");
      const branch = 'else if (request.kind === "workload") {';
      assert.equal(body.split(branch).length, 2);
      writeFileSync(
        backend,
        body.replace(branch, `${branch}\nif(request.name === "target") process.exit(23);`),
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
      assert.equal(
        trace.filter(({ kind }) => kind === "workload" || kind === "cache-post").length,
        0,
      );
    });
});
