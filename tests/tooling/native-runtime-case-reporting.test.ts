import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { cargoTestPackages } from "../../tools/support/compat/github/job-test-cases.ts";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  id?: string;
  if?: string;
  run?: string;
  uses?: string;
  with?: Record<string, string | number>;
};
type Job = { name?: string; env?: Record<string, string>; steps: Step[] };

const workflow = (file: string) =>
  parse(readRepoFile(".github", "workflows", file)) as { jobs: Record<string, Job> };

test("cargo test package owners are retained for the stable output bridge", () => {
  assert.deepEqual(
    cargoTestPackages(["test", "--locked", "-p", "vize_l4", "--package=vize_atelier_sfc"]),
    ["vize_l4", "vize_atelier_sfc"],
  );
  assert.deepEqual(cargoTestPackages(["test", "-pvize_canon", "--lib"]), ["vize_canon"]);
  assert.deepEqual(cargoTestPackages(["test", "--workspace"]), []);
  assert.equal(cargoTestPackages(["clippy", "-p", "vize"]), null);
  assert.throws(() => cargoTestPackages(["test", "-p"]), /requires a package/u);
});

test("every selected native runtime lane uploads one merged case packet", () => {
  const job = workflow("pr-source-checks.yml").jobs["pr-native-runtime"];
  assert.equal(job.env?.TEST_CASES_JOB, "Native runtime (${{ matrix.lane }})");
  assert.equal(
    job.env?.VIZE_TEST_CASES,
    "${{ runner.temp }}/test-cases-native-${{ matrix.lane }}.json",
  );
  const prepare = job.steps.find(
    (step) => step.name === "Prepare native runtime test case capture",
  );
  const collect = job.steps.find((step) => step.name === "Collect native runtime test cases");
  const upload = job.steps.find((step) => step.name === "Upload native runtime test cases");
  assert.match(prepare?.run ?? "", /job-test-cases\.ts setup/u);
  assert.equal(collect?.id, "native-cases");
  assert.equal(collect?.if, "${{ !cancelled() && matrix.lane != 'none' }}");
  assert.equal(collect?.run, "node tools/support/compat/github/job-test-cases.ts finalize");
  assert.equal(upload?.if, "${{ !cancelled() && steps.native-cases.outcome == 'success' }}");
  assert.deepEqual(upload?.with, {
    name: "test-cases-native-${{ matrix.lane }}-${{ github.run_attempt }}",
    path: "${{ env.VIZE_TEST_CASES }}",
    "if-no-files-found": "error",
    "retention-days": 7,
  });
  assert.ok(job.steps.indexOf(prepare ?? {}) < job.steps.indexOf(collect ?? {}));
});

test("UI conformance reports its node and package-runner cases", () => {
  const job = workflow("pr-js-checks.yml").jobs["pr-js-package-conformance"];
  assert.equal(job.env?.TEST_CASES_JOB, "PR package conformance (${{ matrix.surface }})");
  const prepare = job.steps.find(
    (step) => step.name === "Prepare UI conformance test case capture",
  );
  const collect = job.steps.find((step) => step.name === "Collect UI conformance test cases");
  const upload = job.steps.find((step) => step.name === "Upload UI conformance test cases");
  assert.match(prepare?.if ?? "", /matrix\.surface == 'ui'/u);
  assert.equal(collect?.id, "conformance-cases");
  assert.match(collect?.run ?? "", /job-test-cases\.ts finalize/u);
  assert.equal(
    upload?.with?.name,
    "test-cases-js-conformance-${{ matrix.surface }}-${{ github.run_attempt }}",
  );
  assert.equal(upload?.with?.path, "${{ env.VIZE_TEST_CASES }}");
});
