import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

const root = path.resolve(import.meta.dirname, "../..");
const sourceSha = "1".repeat(40);
const candidateSha = "2".repeat(40);

void test("the reusable Action publishes the same plan or admission artifact", () => {
  const actionSource = readFileSync(
    path.join(root, ".github/actions/v1-five-case-receipts/action.yml"),
    "utf8",
  );
  const action = parse(actionSource) as {
    inputs: Record<string, { required?: boolean }>;
    outputs: Record<string, { value: string }>;
    runs: {
      steps: Array<{
        id?: string;
        env?: Record<string, string>;
        run?: string;
        uses?: string;
        with?: Record<string, string>;
      }>;
    };
  };
  for (const input of [
    "operation",
    "source-sha",
    "candidate-sha",
    "mode",
    "run-id",
    "run-attempt",
    "artifact-name",
  ])
    assert.equal(action.inputs[input]?.required, true, input);
  assert.equal(action.outputs["output-path"]?.value, "${{ steps.driver.outputs.output_path }}");

  const driver = action.runs.steps.find(({ id }) => id === "driver")!;
  assert.equal(driver.env?.SOURCE_SHA, "${{ inputs.source-sha }}");
  assert.equal(driver.env?.CANDIDATE_SHA, "${{ inputs.candidate-sha }}");
  assert.equal(driver.env?.RUN_IDENTITY, "${{ inputs.run-id }}");
  assert.equal(driver.env?.RUN_ATTEMPT, "${{ inputs.run-attempt }}");

  const upload = action.runs.steps.find(({ uses }) =>
    uses?.startsWith("actions/upload-artifact@"),
  )!;
  assert.equal(upload.uses, "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a");
  assert.equal(upload.with?.path, "${{ steps.driver.outputs.output_path }}");
  assert.equal(upload.with?.["if-no-files-found"], "error");

  const temporary = mkdtempSync(path.join(os.tmpdir(), "vize-five-case-action-"));
  try {
    const binaryDirectory = path.join(temporary, "bin");
    const bundlesDirectory = path.join(temporary, "bundles");
    const argumentsLog = path.join(temporary, "node-arguments.txt");
    const output = path.join(temporary, "github-output.txt");
    mkdirSync(binaryDirectory);
    mkdirSync(bundlesDirectory);
    writeFileSync(
      path.join(binaryDirectory, "git"),
      '#!/usr/bin/env bash\nset -euo pipefail\ntest "$1" = rev-parse\ntest "$2" = HEAD\nprintf "%s\\n" "$SOURCE_SHA"\n',
    );
    writeFileSync(
      path.join(binaryDirectory, "node"),
      '#!/usr/bin/env bash\nset -euo pipefail\nprintf "%s\\n" "$@" > "$NODE_ARGUMENTS_LOG"\narguments=("$@")\nrequired=(--source --candidate --mode)\ncase "$2" in\n  plan) required+=(--run --attempt --shards --output) ;;\n  validate-bundles) required+=(--bundles) ;;\n  admit) required+=(--bundles --output) ;;\n  *) exit 2 ;;\nesac\nfor required_argument in "${required[@]}"; do\n  found=false\n  for ((index = 0; index < ${#arguments[@]}; index++)); do\n    if [ "${arguments[$index]}" = "$required_argument" ]; then\n      value_index=$((index + 1))\n      test -n "${arguments[$value_index]:-}"\n      found=true\n    fi\n  done\n  $found\ndone\n',
    );
    chmodSync(path.join(binaryDirectory, "git"), 0o755);
    chmodSync(path.join(binaryDirectory, "node"), 0o755);

    const runDriver = (script: string, operation: string, overrides: Record<string, string> = {}) =>
      spawnSync("bash", ["-euo", "pipefail", "-c", script], {
        cwd: temporary,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${binaryDirectory}:${process.env.PATH ?? ""}`,
          OPERATION: operation,
          SOURCE_SHA: sourceSha,
          CANDIDATE_SHA: candidateSha,
          MODE: "pr",
          RUN_IDENTITY: "44000000000",
          RUN_ATTEMPT: "1",
          SHARD_COUNT: "8",
          BUNDLES_DIRECTORY: bundlesDirectory,
          ENFORCE_COMPLETE: "false",
          GITHUB_OUTPUT: output,
          NODE_ARGUMENTS_LOG: argumentsLog,
          ...overrides,
        },
      });

    for (const [operation, command, outputPath] of [
      ["plan", "plan", ".qualification/v1-five-case/plan.json"],
      ["validate", "validate-bundles", bundlesDirectory],
      ["admit", "admit", ".qualification/v1-five-case/admission.json"],
    ] as const) {
      writeFileSync(output, "");
      const result = runDriver(driver.run!, operation);
      assert.equal(result.status, 0, `${operation}: ${result.stderr}`);
      const arguments_ = readFileSync(argumentsLog, "utf8").split("\n");
      const valueAfter = (name: string) => arguments_[arguments_.indexOf(name) + 1];
      assert.equal(arguments_[0], "tools/commands/release/v1-five-case-receipts.ts");
      assert.equal(arguments_[1], command);
      assert.equal(valueAfter("--source"), sourceSha, operation);
      assert.equal(valueAfter("--candidate"), candidateSha, operation);
      assert.equal(valueAfter("--mode"), "pr", operation);
      if (operation === "plan") {
        assert.equal(valueAfter("--run"), "44000000000", operation);
        assert.equal(valueAfter("--attempt"), "1", operation);
        assert.equal(valueAfter("--shards"), "8", operation);
        assert.equal(valueAfter("--output"), outputPath, operation);
      } else {
        assert.equal(valueAfter("--bundles"), bundlesDirectory, operation);
        if (operation === "admit") assert.equal(valueAfter("--output"), outputPath, operation);
      }
      assert.equal(readFileSync(output, "utf8").trim(), `output_path=${outputPath}`);
    }

    const invalidOperation = runDriver(driver.run!, "not-an-operation");
    assert.notEqual(invalidOperation.status, 0);
    assert.match(invalidOperation.stderr, /operation must be plan, validate, or admit/u);

    const missingCandidate = runDriver(driver.run!, "plan", { CANDIDATE_SHA: "" });
    assert.notEqual(missingCandidate.status, 0);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});
