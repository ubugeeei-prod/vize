import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { planPrChecks } from "../../tools/support/compat/github/plan-pr-checks.ts";
import { aggregateCheckNeedsResults } from "../../tools/support/compat/github/require-needs-success.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const metadata = {
  workspace_root: "/repo",
  workspace_members: ["vize_armature", "vize_vitrine", "vize_curator", "vize_atelier_ssr"],
  packages: ["vize_armature", "vize_vitrine", "vize_curator", "vize_atelier_ssr"].map((name) => ({
    id: name,
    name,
    manifest_path: `/repo/crates/${name}/Cargo.toml`,
    dependencies: [],
  })),
};
const workspace = [
  { name: "@vizejs/native", path: "npm/native", dependencies: [] },
  { name: "@vizejs/cli", path: "npm/cli", dependencies: ["@vizejs/native"] },
  { name: "@vizejs/oxlint", path: "npm/oxlint", dependencies: ["@vizejs/native"] },
  { name: "@vizejs/ui", path: "npm/ui", dependencies: ["@vizejs/native"] },
];

test("explicit manual source qualification selects all Rust and genuine product families", () => {
  const request = {
    paths: ["docs/guide.md"],
    eventName: "workflow_dispatch",
    labels: [],
    workspace,
    cargoMetadata: metadata,
  };
  const selected = planPrChecks({ ...request, fullProfile: true });
  assert.deepEqual(selected.rust.packages, [...metadata.workspace_members].sort());
  assert.equal(selected.tier, "heavy");
  assert.equal(selected.source.playground, true);
  assert.equal(selected.sourceContracts, true);
  assert.ok(selected.js.groups.length > 0);
  assert.equal(selected.native.matrix.lane.includes("none"), false);
  assert.equal(request.eventName, "workflow_dispatch");
  assert.throws(() => planPrChecks(request));
  assert.throws(() => planPrChecks({ ...request, eventName: "unknown", fullProfile: true }));
});

test("full source reports reject skipped queue qualifications and preserve every failure", () => {
  const needs = {
    "nuxt-module-qualification": { result: "success" },
    other: { result: "success" },
  };
  assert.equal(aggregateCheckNeedsResults(needs, "workflow_dispatch", true).exitCode, 0);
  for (const result of ["skipped", "failure", "cancelled"]) {
    assert.equal(
      aggregateCheckNeedsResults(
        { ...needs, "nuxt-module-qualification": { result } },
        "workflow_dispatch",
        true,
      ).exitCode,
      1,
    );
  }
  assert.equal(
    aggregateCheckNeedsResults(
      { ...needs, "nuxt-module-qualification": { result: "skipped" } },
      "workflow_dispatch",
    ).exitCode,
    0,
  );
});

test("actual Rust envelope permits only explicitly requested manual full execution", () => {
  const workflow = parse(readRepoFile(".github/workflows/pr-rust-products.yml")) as {
    jobs: Record<string, { steps: Array<{ name?: string; run?: string }> }>;
  };
  const script = workflow.jobs["pr-rust-shard"].steps.find(
    (step) => step.name === "Select Rust runtime envelope",
  )?.run;
  assert.ok(script);
  const directory = mkdtempSync(join(tmpdir(), "source-full-envelope-"));
  try {
    for (const full of ["true", "false"]) {
      const output = join(directory, `${full}.env`);
      const environment: Record<string, string | undefined> = {
        ...process.env,
        GITHUB_EVENT_NAME: "workflow_dispatch",
        SOURCE_FULL_PROFILE: full,
        COHORT: "cli-and-integrations",
        GITHUB_ENV: output,
      };
      delete environment.VIZE_TEST_DISABLE_TSGO;
      delete environment.VIZE_TEST_REQUIRE_TSGO;
      const result = spawnSync("/bin/bash", ["--noprofile", "--norc", "-e", "-c", script], {
        env: environment,
        encoding: "utf8",
      });
      if (full === "true") {
        assert.equal(result.status, 0, result.stderr);
        assert.equal(readFileSync(output, "utf8"), "VIZE_TEST_REQUIRE_TSGO=1\n");
      } else assert.notEqual(result.status, 0);
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
