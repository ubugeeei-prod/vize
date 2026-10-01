import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { toolingTestCommand } from "../../tools/support/compat/github/run-tooling-tests.mjs";
import {
  selectToolingShard,
  toolingShardMatrix,
} from "../../tools/support/compat/github/tooling-test-shards.ts";

const plan = (tests, tier = "pr") => ({ version: 1, tier, tests });

void test("PR shards preserve every selected file exactly once and remain serial", () => {
  for (let size = 0; size < 33; size += 1) {
    const files = Array.from({ length: size }, (_, index) => `tests/tooling/${index}.test.ts`);
    const input = plan(files);
    const shards = toolingShardMatrix(input).include;
    const selected = shards.flatMap(({ index, total }) => {
      const shard = `${index}/${total}`;
      const command = toolingTestCommand(input, files, shard);
      assert.deepEqual(command.slice(0, 2), ["--test", "--test-concurrency=1"]);
      assert.deepEqual(command.slice(2), selectToolingShard(input, shard));
      return command.slice(2);
    });
    assert.deepEqual([...selected].sort(), [...files].sort());
    assert.equal(new Set(selected).size, files.length);
    assert.ok(shards.length <= 4);
    if (size > 0)
      assert.ok(
        shards.every(
          ({ index, total }) => selectToolingShard(input, `${index}/${total}`).length > 0,
        ),
      );
  }
});

void test("invalid or incomplete shard coordinates fail closed", () => {
  const files = Array.from({ length: 8 }, (_, index) => `tests/tooling/${index}.test.ts`);
  for (const shard of [null, 1, "0/4", "5/4", "1/3", "1/5", "1.5/4", "1/4/2", "1/0"]) {
    assert.throws(() => toolingTestCommand(plan(files), files, shard), /complete PR tooling shard/);
  }
  assert.throws(() => toolingTestCommand(plan([files[0], files[0]]), files, "1/2"), /unrecognized/);
  assert.throws(() => toolingShardMatrix(plan([], "unknown")), /invalid/);
});

void test("merge keeps the full suite in one runner and refuses PR partition coordinates", () => {
  const files = ["tests/tooling/a.test.ts", "tests/tooling/b.test.ts"];
  const input = plan(files, "merge");
  assert.deepEqual(toolingShardMatrix(input), { include: [{ index: 1, total: 1 }] });
  assert.deepEqual(toolingTestCommand(input, files).slice(2), files);
  assert.throws(() => toolingTestCommand(input, files, "1/1"), /complete PR tooling shard/);
  assert.throws(() => toolingTestCommand(plan([], "merge"), files), /retain every test/);
});

void test("an empty selection does not invoke Node's implicit test discovery", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-tooling-empty-shard-"));
  try {
    const file = join(cwd, "plan.json");
    writeFileSync(file, JSON.stringify(plan([])));
    const result = spawnSync(
      process.execPath,
      ["tools/support/compat/github/run-tooling-tests.mjs", file],
      {
        encoding: "utf8",
        env: { ...process.env, VIZE_TOOLING_TEST_SHARD: "1/1" },
      },
    );
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /No tooling tests selected/);
    assert.doesNotMatch(result.stdout, /TAP version|tests \d+/);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});
