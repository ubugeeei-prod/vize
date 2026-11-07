import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import {
  cargoTestPackages,
  jobTestShell,
} from "../../tools/support/compat/github/job-test-cases.ts";
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

test("the Node job shim preserves PID and transport through filtered environments", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-job-test-shim-"));
  const shim = path.join(directory, "node");
  try {
    const values = {
      VIZE_JOB_TEST_CASE_PARTIALS: path.join(directory, "partials"),
      VIZE_JOB_TEST_CASE_TOOL: path.join(directory, "missing-tool.ts"),
      VIZE_JOB_TEST_CASE_NODE: process.execPath,
      VIZE_JOB_TEST_CASE_VP: path.join(directory, "vp"),
      VIZE_JOB_TEST_CASE_CARGO: path.join(directory, "cargo"),
      VIZE_TEST_CASES_ROOT: directory,
    };
    fs.writeFileSync(shim, jobTestShell("node", values), { mode: 0o755 });
    const result = spawnSync(shim, ["-e", "process.stdout.write(String(process.ppid))"], {
      encoding: "utf8",
      env: { PATH: process.env.PATH ?? "" },
    });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, String(process.pid));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("every selected native runtime lane uploads one merged case packet", () => {
  const job = workflow("pr-source-checks.yml").jobs["pr-native-runtime"];
  assert.equal(job.env?.TEST_CASES_JOB, "Native runtime (${{ matrix.lane }})");
  assert.equal(job.env?.VIZE_TEST_CASES, "test-cases-native-${{ matrix.lane }}.json");
  const prepare = job.steps.find(
    (step) => step.name === "Prepare native runtime test case capture",
  );
  const collect = job.steps.find(
    (step) => step.name === "Collect and upload native runtime test cases",
  );
  assert.match(prepare?.run ?? "", /job-test-cases\.ts setup/u);
  assert.equal(collect?.if, "${{ !cancelled() && matrix.lane != 'none' }}");
  assert.equal(collect?.uses, "./.github/actions/collect-job-test-cases");
  assert.deepEqual(collect?.with, {
    artifact: "test-cases-native-${{ matrix.lane }}-${{ github.run_attempt }}",
  });
  assert.ok(job.steps.indexOf(prepare ?? {}) < job.steps.indexOf(collect ?? {}));
});

test("UI conformance reports its node and package-runner cases", () => {
  const job = workflow("pr-js-checks.yml").jobs["pr-js-package-conformance"];
  assert.equal(job.env?.TEST_CASES_JOB, "PR package conformance (${{ matrix.surface }})");
  const prepare = job.steps.find(
    (step) => step.name === "Prepare UI conformance test case capture",
  );
  const collect = job.steps.find(
    (step) => step.name === "Collect and upload UI conformance test cases",
  );
  assert.match(prepare?.if ?? "", /matrix\.surface == 'ui'/u);
  assert.equal(collect?.uses, "./.github/actions/collect-job-test-cases");
  assert.equal(
    collect?.with?.artifact,
    "test-cases-js-conformance-${{ matrix.surface }}-${{ github.run_attempt }}",
  );
});
