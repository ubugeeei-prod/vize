import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { toolingTestCommand } from "../../tools/support/compat/github/run-tooling-tests.mjs";
import { parseToolingTimingLog } from "../../tools/support/compat/github/update-tooling-test-timings.ts";
import {
  planToolingTests,
  toolingTestFiles,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";
import {
  MAX_TOOLING_SHARDS,
  TOOLING_SHARD_TARGET_MS,
  TOOLING_LEAD_STEPS,
  TOOLING_TIMINGS_LOG_MARKER,
  estimateToolingTestMs,
  selectToolingShard,
  toolingLeadSteps,
  toolingShardMatrix,
  toolingShards,
} from "../../tools/support/compat/github/tooling-test-shards.ts";

const plan = (tests, tier = "pr", leadSteps) => ({
  version: 1,
  tier,
  tests,
  ...(leadSteps ? { leadSteps } : {}),
});
const allLeadSteps = TOOLING_LEAD_STEPS.map(({ id }) => id);

for (const tier of ["pr", "merge"]) {
  void test(`${tier} shards preserve every selected file exactly once and remain serial`, () => {
    for (const size of [...Array.from({ length: 33 }, (_, size) => size), 100, 170, 400]) {
      for (const leadSteps of [undefined, ["historical-sfc-documents"], allLeadSteps]) {
        const files = Array.from({ length: size }, (_, index) => `tests/tooling/${index}.test.ts`);
        const input = plan(files, tier, leadSteps);
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
        assert.ok(shards.length >= 1 && shards.length <= Math.min(MAX_TOOLING_SHARDS, size || 1));
        // Runner 1 may be left to its captures; every other runner receives files.
        assert.ok(
          shards.every(
            ({ index, total }) =>
              size === 0 ||
              (index === 1 && leadSteps) ||
              selectToolingShard(input, `${index}/${total}`).length > 0,
          ),
        );
      }
    }
  });
}

void test("timed packing uses the fewest runners within the target and bounds the longest", () => {
  // Unmeasured files share one conservative estimate, so 10 of them fit one runner.
  const unmeasured = (size) =>
    Array.from({ length: size }, (_, index) => `tests/tooling/new-${index}.test.ts`);
  const estimate = estimateToolingTestMs("pr", "tests/tooling/new-0.test.ts");
  const perRunner = Math.floor(TOOLING_SHARD_TARGET_MS.pr / estimate);
  for (const size of [1, perRunner, perRunner + 1, perRunner * 5, perRunner * 50]) {
    assert.equal(
      toolingShards(plan(unmeasured(size))).length,
      Math.min(MAX_TOOLING_SHARDS, Math.ceil(size / perRunner)),
    );
  }
  for (const tier of ["pr", "merge"]) {
    const files = planToolingTests(["Cargo.toml"], { tier: "merge" }).tests;
    const input = plan(tier === "pr" ? files.slice(0, 900) : files, tier);
    const shards = toolingShards(input);
    const estimates = input.tests.map((file) => estimateToolingTestMs(tier, file));
    const sum = estimates.reduce((total, ms) => total + ms, 0);
    const longest = Math.max(...shards.map(({ testMs }) => testMs));
    // Longest-first packing never exceeds the average plus the heaviest single file.
    assert.ok(longest <= sum / shards.length + Math.max(...estimates), tier);
    assert.ok(
      shards.length === MAX_TOOLING_SHARDS ||
        longest <= Math.max(TOOLING_SHARD_TARGET_MS[tier], ...estimates),
      tier,
    );
    for (const [index, shard] of shards.entries()) {
      assert.deepEqual(shard.files, selectToolingShard(input, `${index + 1}/${shards.length}`));
      assert.deepEqual(
        shard.files,
        input.tests.filter((file) => shard.files.includes(file)),
      );
    }
  }
});

void test("runner 1 reserves exactly the captures its matrix entry runs", () => {
  assert.deepEqual(toolingLeadSteps("pr", {}), ["historical-sfc-documents"]);
  assert.deepEqual(toolingLeadSteps("merge", {}), allLeadSteps);
  assert.deepEqual(toolingLeadSteps("pr", { "native-ssr-capture": true }), [
    "native-ssr",
    "native-scoped-css",
    "historical-sfc-documents",
  ]);
  assert.deepEqual(
    toolingLeadSteps("pr", {
      "native-ssr-capture": true,
      "native-setup-capture": true,
      "native-vapor-capture": true,
      "native-attribute-values-7502-capture": true,
      "merge-only": true,
    }),
    allLeadSteps.filter((id) => id !== "native-selected-template"),
  );
  const files = planToolingTests(["Cargo.toml"], { tier: "merge" }).tests;
  const plain = toolingShards(plan(files, "merge"));
  const reserved = toolingShards(plan(files, "merge", allLeadSteps));
  assert.equal(reserved.length, plain.length);
  assert.ok(reserved[0].testMs < plain[0].testMs);
  assert.deepEqual(reserved.flatMap(({ files }) => files).sort(), files);
  for (const leadSteps of [["unknown"], ["native-ssr", "native-ssr"], "native-ssr"]) {
    assert.throws(() => toolingShardMatrix(plan(files, "merge", leadSteps)), /invalid/);
    assert.throws(() => toolingTestCommand(plan(files, "merge", leadSteps), files, "1/16"));
  }
});

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
  const input = { ...planToolingTests(["Cargo.toml"], { tier: "merge" }), leadSteps: allLeadSteps };
  const shards = toolingShardMatrix(input).include;
  assert.ok(shards.length > 4 && shards.length <= MAX_TOOLING_SHARDS);
  assert.equal(new Set(shards.map(({ name }) => name)).size, shards.length);
  for (const { name, index, total } of shards) {
    const selected = selectToolingShard(input, `${index}/${total}`);
    if (selected.length === 0) {
      assert.equal(name, "runtime captures");
    } else {
      assert.ok(
        selected.some((file) =>
          name.startsWith(file.replace(/^tests\/tooling\//, "").replace(/\.test\.(?:ts|mjs)$/, "")),
        ),
        `worker label must identify one of its actual test files: ${name}`,
      );
    }
  }
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
        env: { ...process.env, VIZE_TOOLING_TEST_TIER: "pr", VIZE_TOOLING_TEST_SHARD: "1/1" },
      },
    );
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /No tooling tests selected/);
    assert.doesNotMatch(result.stdout, /TAP version|tests \d+/);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

void test("a selected shard keeps its spec output and logs one per-file timing record", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-tooling-timed-shard-"));
  try {
    const file = join(cwd, "plan.json");
    const selected = ["tests/tooling/git-tracked-files.test.ts"];
    writeFileSync(file, JSON.stringify(plan(selected, "pr", ["historical-sfc-documents"])));
    const env = { ...process.env, VIZE_TOOLING_TEST_TIER: "pr", VIZE_TOOLING_TEST_SHARD: "1/1" };
    delete env.NODE_TEST_CONTEXT;
    const result = spawnSync(
      process.execPath,
      ["tools/support/compat/github/run-tooling-tests.mjs", file],
      { encoding: "utf8", env },
    );
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.match(result.stdout, /✔ a failed Git command remains a failed tracked-file inspection/);
    assert.match(result.stdout, /ℹ fail 0/);
    const records = parseToolingTimingLog(result.stdout);
    assert.equal(records.length, 1);
    assert.equal(result.stdout.split(TOOLING_TIMINGS_LOG_MARKER).length, 2);
    assert.equal(records[0].tier, "pr");
    assert.equal(records[0].shard, "1/1");
    assert.deepEqual(Object.keys(records[0].files), selected);
    assert.ok(records[0].files[selected[0]] > 0);
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
