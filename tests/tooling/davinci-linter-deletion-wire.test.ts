import { readToolingProgram } from "./support/program-path.ts";
import { automationShellPath, readAutomationPrograms } from "./support/automation-source.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { deletionReadinessAdapters } from "../differential/deletion-readiness-adapters.ts";
import { recordLinterDeletionShard } from "../differential/linter-deletion-recorder.ts";
import type { RegistryProject } from "../differential/linter-deletion-schema.ts";
import { linterProjects } from "../differential/linter-deletion-verifier.ts";

void test("Matrix records the blocked preflight without registering linter deletion credit", () => {
  const workflow = parse(
      readAutomationPrograms(
        new URL("../../.github/workflows/real-project-matrix.yml", import.meta.url),
      ),
    ) as { jobs: Record<string, { steps: Array<Record<string, unknown>> }> },
    steps = workflow.jobs["real-project-matrix"].steps,
    divergenceIndex = steps.findIndex(
      (step) => step.name === "Measure real-project lint divergence",
    ),
    verifierIndex = steps.findIndex(
      (step) => step.name === "Record linter deletion-readiness evidence",
    ),
    uploadIndex = steps.findIndex((step) => step.name === "Upload shard report");
  assert(divergenceIndex >= 0, "lint divergence step is missing");
  assert(verifierIndex > divergenceIndex, "recorder must run after lint divergence");
  assert(uploadIndex > verifierIndex, "artifact upload must run after the recorder");
  const recorder = steps[verifierIndex];
  assert.equal(recorder.if, "${{ !cancelled() }}");
  const upload = steps[uploadIndex];
  assert.equal(
    typeof upload.uses === "string" && upload.uses.startsWith("actions/upload-artifact@"),
    true,
  );
  assert.equal((upload.with as Record<string, unknown>).path, "${{ env.FIXTURE_REPORT_DIR }}");

  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-linter-deletion-wire-"));
  try {
    const bin = path.join(root, "bin"),
      argumentsPath = path.join(root, "recorder-arguments.txt"),
      reportDirectory = "real-project-results/shard-7";
    fs.mkdirSync(bin);
    fs.writeFileSync(
      path.join(bin, "node"),
      readToolingProgram("commands/davinci-linter-deletion-wire/node.sh"),
      { mode: 0o755 },
    );
    const result = spawnSync(
      "bash",
      ["-euo", "pipefail", automationShellPath(String(recorder.run))],
      {
        cwd: root,
        encoding: "utf8",
        env: {
          ...process.env,
          FIXTURE_REPORT_DIR: reportDirectory,
          PATH: `${bin}:${process.env.PATH}`,
          RECORDER_ARGUMENTS: argumentsPath,
        },
      },
    );
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(fs.readFileSync(argumentsPath, "utf8").trim().split("\n"), [
      "tests/differential/linter-deletion-recorder.ts",
      reportDirectory,
    ]);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
  assert.deepEqual(Object.keys(deletionReadinessAdapters), ["formatter"]);
});

void test("the recorder fails closed on a missing or corrupt project lint artifact", () => {
  const root = path.resolve(import.meta.dirname, "../.."),
    registry = JSON.parse(
      fs.readFileSync(path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
    ) as { projects: RegistryProject[] },
    selected = linterProjects(registry.projects, 0, 22),
    reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-linter-deletion-missing-")),
    environment = ["GITHUB_SHA", "FIXTURE_SHARD_INDEX", "FIXTURE_SHARD_COUNT"] as const,
    saved = environment.map((name) => process.env[name]);
  assert(selected.length > 1);
  const absent = selected[selected.length - 1].id;
  try {
    Object.assign(process.env, {
      GITHUB_SHA: "a".repeat(40),
      FIXTURE_SHARD_INDEX: "0",
      FIXTURE_SHARD_COUNT: "22",
    });
    fs.writeFileSync(
      path.join(reportDir, "summary.json"),
      JSON.stringify({
        projects: selected.map((project) => ({
          id: project.id,
          runs: [{ tool: "linter", outputPath: path.join(reportDir, `${project.id}-linter.json`) }],
        })),
      }),
    );
    for (const project of selected) {
      fs.writeFileSync(path.join(reportDir, `${project.id}-linter.json`), "{}\n");
      if (project.id !== absent)
        fs.writeFileSync(path.join(reportDir, `${project.id}-lint-divergence.json`), "{}\n");
    }
    assert.throws(
      () => recordLinterDeletionShard(root, reportDir),
      new RegExp(`^Error: ${absent}: lint divergence artifact is missing: `, "u"),
    );
    fs.writeFileSync(path.join(reportDir, `${absent}-lint-divergence.json`), "{");
    assert.throws(
      () => recordLinterDeletionShard(root, reportDir),
      new RegExp(`^Error: ${absent}: lint divergence artifact is not valid JSON: `, "u"),
    );
    assert.equal(fs.existsSync(path.join(reportDir, "linter-deletion-readiness.json")), false);
  } finally {
    environment.forEach((name, index) => {
      if (saved[index] === undefined) delete process.env[name];
      else process.env[name] = saved[index];
    });
    fs.rmSync(reportDir, { recursive: true, force: true });
  }
});
