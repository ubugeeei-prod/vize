import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { toolingTestCommand } from "../../tools/support/compat/github/run-tooling-tests.mjs";
import {
  planToolingTests,
  toolingTestFiles,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";
import {
  selectToolingShard,
  toolingShardMatrix,
} from "../../tools/support/compat/github/tooling-test-shards.ts";

const plan = (tests, tier = "pr") => ({ version: 1, tier, tests });

for (const tier of ["pr", "merge"]) {
  void test(`${tier} shards preserve every selected file exactly once and remain serial`, () => {
    for (let size = 0; size < 33; size += 1) {
      const files = Array.from({ length: size }, (_, index) => `tests/tooling/${index}.test.ts`);
      const input = plan(files, tier);
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
}

void test("invalid or incomplete shard coordinates fail closed", () => {
  const files = Array.from({ length: 8 }, (_, index) => `tests/tooling/${index}.test.ts`);
  for (const tier of ["pr", "merge"]) {
    for (const shard of [null, 1, "0/4", "5/4", "1/3", "1/5", "1.5/4", "1/4/2", "1/0"]) {
      assert.throws(
        () => toolingTestCommand(plan(files, tier), files, shard),
        /complete tooling shard/,
      );
    }
  }
  assert.throws(() => toolingTestCommand(plan([files[0], files[0]]), files, "1/2"), /unrecognized/);
  assert.throws(() => toolingShardMatrix(plan([], "unknown")), /invalid/);
});

void test("real merge shards retain every discovered test, including all deferred runtime files", () => {
  const files = toolingTestFiles();
  const input = planToolingTests(["README.md"], { tier: "merge" });
  const shards = toolingShardMatrix(input).include;
  assert.equal(shards.length, 4);
  const selected = shards.flatMap(({ index, total }) =>
    toolingTestCommand(input, files, `${index}/${total}`).slice(2),
  );
  assert.deepEqual([...selected].sort(), files);
  assert.equal(new Set(selected).size, files.length);
  for (const tests of [[], files.slice(1), [...files, files[0]]]) {
    assert.throws(
      () => toolingTestCommand(plan(tests, "merge"), files, "1/4"),
      /retain every test|unrecognized/,
    );
  }
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

void test("the real merge runner rejects a partial PR plan before starting any tests", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-tooling-wrong-tier-"));
  try {
    const file = join(cwd, "plan.json");
    writeFileSync(file, JSON.stringify(plan([])));
    for (const tier of ["merge", "unknown"]) {
      const result = spawnSync(
        process.execPath,
        ["tools/support/compat/github/run-tooling-tests.mjs", file],
        {
          encoding: "utf8",
          env: { ...process.env, VIZE_TOOLING_TEST_TIER: tier, VIZE_TOOLING_TEST_SHARD: "1/1" },
        },
      );
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /required execution tier/);
      assert.doesNotMatch(result.stdout, /No tooling tests selected|TAP version/);
    }
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});
