import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import {
  compareResults,
  testResults,
} from "../../tools/support/compat/github/compare-unused-bindings-profiles.ts";

const detectorNames = [
  "a_producer_reading_outside_its_depends_is_refused_too",
  "an_undeclared_read_is_refused_exactly_even_when_computed",
];
const output = [
  ...detectorNames.map((name) => `test ${name} ... ok`),
  "test ignored_witness ... ignored, requires external fixture",
  "test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 9 filtered out; finished in 1.23s",
  "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s",
].join("\n");

await test("profile comparison retains every selected status and every Cargo summary", () => {
  const before = testResults(output);
  const after = testResults(output.replace("1.23s", "0.02s"));
  compareResults(before, after, "fact_demand");
  assert.equal(before.cases.length, 3);
  assert.equal(before.summaries.length, 2);
  for (const changed of [
    output.replace("ignored_witness", "different_ignored_witness"),
    output.replace("9 filtered out", "8 filtered out"),
    output.replace("requires external fixture", "different reason"),
    output.replace(detectorNames[0], "different_case"),
  ])
    assert.throws(() => compareResults(before, testResults(changed), "fact_demand"));
});

await test("incomplete, failed and zero-test Cargo output cannot qualify", () => {
  for (const invalid of [
    "",
    output.replace(`test ${detectorNames[0]} ... ok\n`, ""),
    output.replace("test ignored_witness ... ignored, requires external fixture\n", ""),
    output.replace("2 passed; 0 failed", "1 passed; 1 failed"),
    output.replace("test result: ok", "test result: FAILED"),
    "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.00s",
  ])
    assert.throws(() => testResults(invalid));
});

await test("equal output still requires both debug detectors and the actual corpus test", () => {
  const missing = testResults(output.replace(detectorNames[0], "release_builds_carry_no_detector"));
  assert.throws(() => compareResults(missing, missing, "fact_demand"));
  const ignored = testResults(
    output
      .replace(`test ${detectorNames[0]} ... ok`, `test ${detectorNames[0]} ... ignored`)
      .replace("2 passed; 0 failed; 1 ignored", "1 passed; 0 failed; 2 ignored"),
  );
  assert.throws(() => compareResults(ignored, ignored, "fact_demand"));
  const result = testResults(output);
  assert.throws(() => compareResults(result, result, "fact_spec"));
  const corpus = testResults(
    output.replace(detectorNames[0], "the_corpus_shard_agrees_with_the_specs"),
  );
  compareResults(corpus, corpus, "fact_spec");
});

await test("the controlled comparison is manual-only and retains the real seeded CLI recall", () => {
  const workflow = parse(
    readFileSync(
      new URL("../../.github/workflows/davinci-unused-bindings.yml", import.meta.url),
      "utf8",
    ),
  );
  assert.equal(workflow.on.workflow_dispatch.inputs["compare-profiles"].default, false);
  const steps = workflow.jobs["unused-bindings"].steps;
  const compare = steps.find((step) => step.run?.includes("compare-unused-bindings-profiles.ts"));
  assert.equal(
    compare.if,
    "${{ github.event_name == 'workflow_dispatch' && inputs.compare-profiles }}",
  );
  assert.equal(
    compare.env.VIZE_DAVINCI_FACT_CORPUS,
    "tests/_fixtures/_git/splitpanes,tests/_fixtures/_git/layoutit-grid,tests/_fixtures/_git/cssgridgenerator",
  );
  assert.ok(steps.some((step) => step.run === "cargo build --profile ci -p vize"));
  const recall = steps.find((step) => step.run?.includes("davinci-unused-setup-bindings.test.ts"));
  assert.equal(recall.if, undefined);
  assert.match(recall.run, /target\/ci\/vize.*davinci-fpfn-pilots\.test\.ts/);
  assert.match(steps.at(-1).if, /always\(\)/);
});
