import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { selectColdScaleBaseline } from "../../tools/benchmarks/scripts/cold-typechecker-scale-build.ts";

test("PR comparisons use the actual merge parent while explicit dispatch baselines remain explicit", () => {
  const root = mkdtempSync(join(tmpdir(), "cold-baseline-parents-"));
  try {
    const git = (...args: string[]) => {
      const result = spawnSync(
        "git",
        ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", ...args],
        { cwd: root, encoding: "utf8" },
      );
      assert.equal(result.status, 0, result.stderr);
      return result.stdout.trim();
    };
    git("init", "-b", "main", "--quiet");
    mkdirSync(join(root, "crates"));
    writeFileSync(join(root, "crates", "base.rs"), "pub fn value() {}\n");
    git("add", ".");
    git("commit", "--quiet", "-m", "base");
    const staleEventBase = git("rev-parse", "HEAD");
    git("switch", "-c", "feature");
    writeFileSync(join(root, "crates", "feature.rs"), "pub fn feature() {}\n");
    git("add", ".");
    git("commit", "--quiet", "-m", "feature");
    const prHead = git("rev-parse", "HEAD");
    git("switch", "main");
    writeFileSync(join(root, "crates", "main.rs"), "pub fn main_change() {}\n");
    git("add", ".");
    git("commit", "--quiet", "-m", "main advanced");
    const actualBase = git("rev-parse", "HEAD");
    git("merge", "--no-edit", "feature");
    const mergeSha = git("rev-parse", "HEAD");
    const selected = selectColdScaleBaseline({
      repository: root,
      event: "pull_request",
      eventBase: staleEventBase,
      prHead,
    });
    assert.deepEqual(selected, {
      sourceSha: mergeSha,
      baselineSha: actualBase,
      strategy: "actual-test-merge-first-parent",
      eventBaseSha: staleEventBase,
      prHeadSha: prHead,
    });
    assert.notEqual(selected.baselineSha, staleEventBase);
    assert.equal(
      selectColdScaleBaseline({
        repository: root,
        event: "workflow_dispatch",
        requestedBase: staleEventBase,
      }).baselineSha,
      staleEventBase,
    );
    assert.throws(
      () =>
        selectColdScaleBaseline({
          repository: root,
          event: "pull_request",
          prHead: staleEventBase,
        }),
      /second parent/u,
    );
    assert.throws(
      () =>
        selectColdScaleBaseline({
          repository: root,
          event: "workflow_dispatch",
          requestedBase: mergeSha,
        }),
      /must differ/u,
    );
    assert.throws(
      () =>
        selectColdScaleBaseline({
          repository: root,
          event: "workflow_dispatch",
          requestedBase: "main",
        }),
      /immutable commit/u,
    );
    git("switch", "feature");
    assert.throws(
      () => selectColdScaleBaseline({ repository: root, event: "pull_request", prHead }),
      /two-parent test merge/u,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
