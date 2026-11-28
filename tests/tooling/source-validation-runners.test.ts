import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import {
  plannedSourceRunnerChanges,
  sourceRunnerChanges,
  sourceRunnersBefore,
} from "./support/source-validation-runners.ts";
import {
  TOOLING_NATIVE_SHARD_RUNNER,
  TOOLING_SHARD_RUNNER,
} from "../../tools/support/compat/github/tooling-test-shards.ts";

type Workflow = { jobs: Record<string, Record<string, unknown>> };
const read = (file: string) =>
  fs.readFileSync(new URL("../../" + file, import.meta.url), "utf8").replace(/\r\n?/g, "\n");

void test("live source checks use exactly the reviewed runner selectors and retain every other field", () => {
  assert.equal(sourceRunnerChanges.length, 22);
  const labels = (
    parse(read(".github/actionlint.yaml")) as {
      "self-hosted-runner": { labels: string[] };
    }
  )["self-hosted-runner"].labels;
  for (const row of sourceRunnerChanges) {
    for (const selector of [row.before, row.after])
      for (const runner of selector.match(/blacksmith-[a-z0-9-]+/g) ?? [])
        assert.ok(
          labels.includes(runner),
          "source runner selectors must pass workflow syntax checks",
        );
  }
  for (const file of new Set(sourceRunnerChanges.map((row) => row.workflow))) {
    const raw = read(file);
    const live = parse(raw) as Workflow;
    const historical = parse(sourceRunnersBefore(file, raw)) as Workflow;
    const expected = structuredClone(live);
    for (const row of sourceRunnerChanges.filter((row) => row.workflow === file)) {
      assert.equal(live.jobs[row.job]?.["runs-on"], row.after);
      expected.jobs[row.job]!["runs-on"] = row.before;
    }
    for (const row of plannedSourceRunnerChanges.filter((row) => row.workflow === file))
      expected.jobs[row.job]!["runs-on"] = row.before;
    assert.deepEqual(historical, expected);
    assert.equal(
      sourceRunnersBefore(file, sourceRunnersBefore(file, raw)),
      sourceRunnersBefore(file, raw),
    );
  }
});

void test("history restoration refuses missing or duplicate jobs and unknown runner changes", () => {
  for (const row of sourceRunnerChanges) {
    const raw = read(row.workflow);
    for (const changed of [
      raw.replace(`  ${row.job}:\n`, `  missing-${row.job}:\n`),
      raw + `\n  ${row.job}:\n    runs-on: ${row.after}\n`,
      raw.replace(row.after, "unreviewed-runner"),
    ])
      assert.throws(() => sourceRunnersBefore(row.workflow, changed));
    assert.equal(sourceRunnersBefore("unowned.yml", raw), raw);
  }
});

void test("planner-chosen runners restore the recorded runner and use only reviewed classes", () => {
  const labels = (
    parse(read(".github/actionlint.yaml")) as {
      "self-hosted-runner": { labels: string[] };
    }
  )["self-hosted-runner"].labels;
  for (const runner of [TOOLING_NATIVE_SHARD_RUNNER, TOOLING_SHARD_RUNNER])
    assert.ok(labels.includes(runner), runner);
  for (const row of plannedSourceRunnerChanges) {
    const raw = read(row.workflow);
    const live = parse(raw) as Workflow;
    assert.equal(live.jobs[row.job]?.["runs-on"], row.after);
    const historical = parse(sourceRunnersBefore(row.workflow, raw)) as Workflow;
    assert.equal(historical.jobs[row.job]?.["runs-on"], row.before);
    assert.equal(
      sourceRunnersBefore(row.workflow, raw.replace(row.after, row.before)),
      sourceRunnersBefore(row.workflow, raw),
    );
    for (const previous of row.previous)
      assert.equal(
        sourceRunnersBefore(row.workflow, raw.replace(row.after, previous)),
        sourceRunnersBefore(row.workflow, raw),
      );
    assert.throws(() =>
      sourceRunnersBefore(row.workflow, raw.replace(row.after, "unreviewed-runner")),
    );
  }
});
