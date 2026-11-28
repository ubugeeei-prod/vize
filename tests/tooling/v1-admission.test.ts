import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { V1_EXACT_CANDIDATE_BUNDLE_SCHEMA } from "../../tools/support/compat/v1/exact-candidate.ts";
import { automationShellPath, readAutomationPrograms } from "./support/automation-source.ts";
import { readToolingProgram } from "./support/program-path.ts";
import { candidate, completeBundle, runIdentity } from "./support/v1-exact-candidate-receipts.ts";

const root = path.resolve(import.meta.dirname, "../..");
const cli = path.join(root, "tools/commands/release/v1-exact-candidate.ts");
const fiveCaseAction = "./.github/actions/v1-five-case-receipts";

interface Step {
  id?: string;
  name?: string;
  if?: string;
  run?: string;
  uses?: string;
  env?: Record<string, string>;
  with?: Record<string, unknown>;
}

interface Workflow {
  on: Record<string, { inputs?: Record<string, { default?: unknown; required?: boolean }> }>;
  jobs: Record<
    string,
    {
      env?: Record<string, string>;
      needs?: string;
      steps?: Step[];
      uses?: string;
      with?: Record<string, unknown>;
    }
  >;
}

function workflow(file: string): Workflow {
  return parse(
    readAutomationPrograms(path.join(root, ".github/workflows", file), "utf8"),
  ) as Workflow;
}

function admitRun(directory: string, receipts: string, enforce: boolean) {
  const output = path.join(directory, "manifest.json");
  const result = spawnSync(
    process.execPath,
    [
      cli,
      "admit-run",
      "--candidate",
      candidate,
      "--run",
      String(runIdentity.run),
      "--run-attempt",
      String(runIdentity.runAttempt),
      "--receipts",
      receipts,
      "--output",
      output,
      ...(enforce ? ["--enforce-complete"] : []),
    ],
    { cwd: root, encoding: "utf8" },
  );
  return { ...result, output };
}

function readManifest(file: string) {
  return JSON.parse(readFileSync(file, "utf8")) as {
    releaseReady: boolean;
    counts: { completeRequiredRows: number; requiredRows: number };
    knownGaps: Array<{ id: string; state: string }>;
  };
}

void test("the qualification manifest workflow calls admission for the same exact source", () => {
  const caller = workflow("v1-supported-matrix.yml").jobs.admission;
  assert.deepEqual(caller, {
    needs: "inventory-contract",
    uses: "./.github/workflows/02-v1-admission.yml",
    with: {
      candidate_sha: "${{ github.event.pull_request.head.sha || github.sha }}",
      enforce_complete: "${{ inputs.enforce_complete }}",
    },
  });

  const admission = workflow("02-v1-admission.yml");
  for (const trigger of ["workflow_call", "workflow_dispatch"]) {
    assert.equal(admission.on[trigger]?.inputs?.candidate_sha?.required, true, trigger);
    assert.equal(admission.on[trigger]?.inputs?.enforce_complete?.default, true, trigger);
  }
  const steps = admission.jobs.admission.steps!;
  const fiveCase = steps.filter(({ uses }) => uses === fiveCaseAction);
  assert.deepEqual(
    fiveCase.map((step) => step.with?.operation),
    ["plan", "admit"],
  );
  for (const step of fiveCase) {
    assert.equal(step.with?.["source-sha"], "${{ inputs.candidate_sha }}");
    assert.equal(step.with?.["candidate-sha"], "${{ inputs.candidate_sha }}");
    assert.equal(step.with?.mode, "release");
  }
  const fiveCaseAdmit = fiveCase[1];
  assert.equal(fiveCaseAdmit.with?.["enforce-complete"], "${{ inputs.enforce_complete }}");
  assert.equal(fiveCaseAdmit.with?.["bundles-directory"], "${{ env.FIVE_CASE_BUNDLES }}");
  // Enforcement never skips the admission, even when the run uploaded no bundles.
  assert.equal(
    fiveCaseAdmit.if,
    "${{ inputs.enforce_complete || steps.five-case-bundles.outputs.count != '0' }}",
  );
  const downloads = steps
    .filter(({ uses }) => uses?.startsWith("actions/download-artifact@"))
    .map((step) => [step.with?.pattern, step.with?.path]);
  assert.deepEqual(downloads, [
    ["v1-five-case-bundle-${{ inputs.candidate_sha }}-*", "${{ env.FIVE_CASE_BUNDLES }}"],
    ["v1-host-receipts-${{ inputs.candidate_sha }}-*", "${{ env.HOST_RECEIPTS }}"],
  ]);
});

void test("the host admission program passes enforcement through to the exact CLI", () => {
  const admission = workflow("02-v1-admission.yml").jobs.admission;
  const step = admission.steps!.find(({ name }) => name === "Admit every required host row")!;
  const temporary = mkdtempSync(path.join(os.tmpdir(), "vize-v1-admission-"));
  try {
    const binaries = path.join(temporary, "bin");
    const log = path.join(temporary, "node-arguments.txt");
    mkdirSync(binaries);
    writeFileSync(path.join(binaries, "node"), readToolingProgram("commands/v1-admission/node.sh"));
    chmodSync(path.join(binaries, "node"), 0o755);
    for (const enforce of ["true", "false"]) {
      const result = spawnSync("bash", ["-euo", "pipefail", automationShellPath(step.run!)], {
        cwd: temporary,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${binaries}:${process.env.PATH ?? ""}`,
          CANDIDATE_SHA: candidate,
          ENFORCE_COMPLETE: enforce,
          HOST_RECEIPTS: admission.env!.HOST_RECEIPTS,
          GITHUB_RUN_ID: String(runIdentity.run),
          GITHUB_RUN_ATTEMPT: String(runIdentity.runAttempt),
          NODE_ARGUMENTS_LOG: log,
        },
      });
      assert.equal(result.status, 0, result.stderr);
      assert.deepEqual(readFileSync(log, "utf8").trimEnd().split("\n"), [
        "tools/commands/release/v1-exact-candidate.ts",
        "admit-run",
        "--candidate",
        candidate,
        "--run",
        String(runIdentity.run),
        "--run-attempt",
        String(runIdentity.runAttempt),
        "--receipts",
        ".qualification/v1-host-receipts",
        "--output",
        ".qualification/v1-exact-candidate-manifest.json",
        ...(enforce === "true" ? ["--enforce-complete"] : []),
      ]);
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

void test("a run without host receipts admits nothing and fails closed when enforced", () => {
  const temporary = mkdtempSync(path.join(os.tmpdir(), "vize-v1-admission-"));
  try {
    const missing = path.join(temporary, "receipts");
    const recorded = admitRun(temporary, missing, false);
    assert.equal(recorded.status, 0, recorded.stderr);
    const manifest = readManifest(recorded.output);
    assert.equal(manifest.releaseReady, false);
    assert.equal(manifest.counts.completeRequiredRows, 0);
    assert.equal(
      manifest.knownGaps.find(({ id }) => id === "host-runtime-join")?.state,
      "blocking",
    );

    const enforced = admitRun(temporary, missing, true);
    assert.notEqual(enforced.status, 0);
    assert.match(
      enforced.stderr,
      new RegExp(`exact candidate admitted 0/${manifest.counts.requiredRows} required rows`, "u"),
    );
    // The failing run still keeps what it admitted.
    assert.equal(readManifest(enforced.output).counts.completeRequiredRows, 0);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

void test("receipt bundles from several lanes join into one complete admission", () => {
  const temporary = mkdtempSync(path.join(os.tmpdir(), "vize-v1-admission-"));
  try {
    const receipts = path.join(temporary, "receipts");
    mkdirSync(receipts);
    const all = completeBundle().receipts;
    const half = Math.floor(all.length / 2);
    for (const [name, part] of [
      ["lane-a.json", all.slice(0, half)],
      ["lane-b.json", all.slice(half)],
    ] as const)
      writeFileSync(
        path.join(receipts, name),
        JSON.stringify({ schema: V1_EXACT_CANDIDATE_BUNDLE_SCHEMA, receipts: part }),
      );
    const result = admitRun(temporary, receipts, true);
    assert.equal(result.status, 0, result.stderr);
    const manifest = readManifest(result.output);
    assert.equal(manifest.counts.completeRequiredRows, manifest.counts.requiredRows);
    assert.equal(
      manifest.knownGaps.find(({ id }) => id === "host-runtime-join")?.state,
      "complete",
    );
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

void test("foreign, duplicate and malformed receipts fail even without enforcement", () => {
  const temporary = mkdtempSync(path.join(os.tmpdir(), "vize-v1-admission-"));
  try {
    const [first, second] = completeBundle().receipts;
    const cases = [
      ["foreign run", [{ ...first, run: runIdentity.run + 1 }], /candidate run/u],
      ["duplicate receipt", [first, first], /duplicate receipt ID/u],
      ["unknown schema", null, /schema/u],
    ] as const;
    for (const [label, receipts, error] of cases) {
      const directory = path.join(temporary, label.replaceAll(" ", "-"));
      mkdirSync(directory);
      writeFileSync(
        path.join(directory, "lane.json"),
        JSON.stringify(
          receipts === null
            ? { schema: "vize.unknown.v1", receipts: [second] }
            : { schema: V1_EXACT_CANDIDATE_BUNDLE_SCHEMA, receipts },
        ),
      );
      const result = admitRun(temporary, directory, false);
      assert.notEqual(result.status, 0, label);
      assert.match(result.stderr, error, label);
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});
