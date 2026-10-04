import assert from "node:assert/strict";
import { test } from "node:test";
import { selectRustWorkerArtifacts } from "../differential/rust-worker-artifacts.ts";
import { requireRustTier } from "../../tools/support/compat/github/require-rust-tier.mjs";

const runId = "37182670251";
const names = (attempt: number) =>
  [1, 2, 3, 4].map((shard) => `rust-test-shard-${shard}-${runId}-${attempt}`);
const run = { runId, runAttempt: 2 };

test("partial reruns retain the latest artifact per shard without relabelling inherited attempts", () => {
  const selected = selectRustWorkerArtifacts([...names(1).reverse(), names(2)[0]], run);
  assert.deepEqual(
    selected.map(({ shard, attempt }) => [shard, attempt]),
    [
      [1, 2],
      [2, 1],
      [3, 1],
      [4, 1],
    ],
  );
  assert.deepEqual(
    selectRustWorkerArtifacts([...names(2), ...names(1)], run).map(({ directory }) => directory),
    names(2),
  );
  assert.deepEqual(
    selectRustWorkerArtifacts(names(1), { runId, runAttempt: 1 }).map(({ directory }) => directory),
    names(1),
  );
});

test("missing, flat, duplicate, foreign-run and future artifact identities refuse before packet reads", () => {
  for (const input of [
    names(1).slice(1),
    ["junit.xml", "timing.json", "typechecker-fixtures"],
    [...names(1), names(1)[0]],
    [...names(1), "rust-test-shard-1-123-2"],
    [...names(1), names(3)[0]],
    [...names(1), `rust-test-shard-5-${runId}-2`],
    [...names(1), `rust-test-shard-1-${runId}-0`],
    [...names(1), `rust-test-shard-1-${runId}-02`],
    [...names(1), `rust-test-shard-1-${runId}-9007199254740992`],
  ]) {
    assert.throws(() => selectRustWorkerArtifacts(input, run));
  }
  for (const context of [
    { runId: "", runAttempt: 2 },
    { runId: "001", runAttempt: 2 },
    { runId, runAttempt: 0 },
    { runId, runAttempt: Number.NaN },
    { runId, runAttempt: 1.5 },
  ]) {
    assert.throws(() => selectRustWorkerArtifacts(names(1), context));
  }
});

test("complete retained artifacts do not override failed current Rust dependencies", () => {
  assert.equal(selectRustWorkerArtifacts(names(1), run).length, 4);
  for (const job of ["merge-rust-source", "pr-rust-shard"]) {
    for (const result of ["failure", "cancelled", "skipped", "in_progress"]) {
      const needs = {
        "merge-rust-source": { result: "success" },
        "pr-rust-shard": { result: "success" },
        [job]: { result },
      };
      assert.equal(requireRustTier("merge_group", "true", needs).exitCode, 1);
    }
    const missing = {
      "merge-rust-source": { result: "success" },
      "pr-rust-shard": { result: "success" },
    };
    Reflect.deleteProperty(missing, job);
    assert.throws(() => requireRustTier("merge_group", "true", missing));
  }
});
