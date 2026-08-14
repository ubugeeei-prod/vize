//! Reading and executing the `check-vue-parity` composite action (#4126).
//!
//! The workflow-shape tests assert two different things about this action: the
//! step graph it declares, and what its runner-profile branch actually emits.
//! The second needs the step's script executed the way the runner executes it,
//! so the harness lives here rather than in either test file.

import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { parse } from "yaml";

import { readRepoFile } from "./github-workflows.ts";

export type CompositeActionStep = {
  env?: Record<string, string>;
  if?: string;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, number | string>;
};

export type CompositeAction = {
  runs?: { steps?: CompositeActionStep[]; using?: string };
};

export const vueParityAction = (): CompositeAction =>
  parse(readRepoFile(".github", "actions", "check-vue-parity", "action.yml")) as CompositeAction;

/** Parse `key=value` lines: the shape of both `GITHUB_ENV` and the baseline. */
export const keyValues = (contents: string): Map<string, string> =>
  new Map(
    contents
      .split("\n")
      .filter((line) => line.includes("="))
      .map((line) => {
        const separator = line.indexOf("=");
        return [line.slice(0, separator), line.slice(separator + 1)] as [string, string];
      }),
  );

export type RunnerBaselineRun = {
  /** Values the step exported to later steps through `GITHUB_ENV`. */
  readonly githubEnv: Map<string, string>;
  /** Facts the step wrote to the always-uploaded topology artifact. */
  readonly baseline: Map<string, string>;
};

/**
 * Run the runner-baseline script for one runner profile and read back both
 * artifacts later steps depend on.
 *
 * The runner profile branch is only worth what it emits, so this executes the
 * script the way the runner does (`bash -e -o pipefail`, a scratch workspace, a
 * fresh `GITHUB_ENV` file) instead of matching its source text.
 */
export const recordRunnerBaseline = (
  script: string,
  runnerEnvironment: string,
): RunnerBaselineRun => {
  const workdir = mkdtempSync(join(tmpdir(), "vize-vue-parity-baseline-"));
  try {
    const scriptPath = join(workdir, "record-runner-baseline.sh");
    const githubEnvPath = join(workdir, "github-env");
    writeFileSync(scriptPath, script);
    writeFileSync(githubEnvPath, "");
    // A cap inherited from this test process would mask an accidental
    // hosted-only value on the self-hosted branch, so start from a clean slate.
    const env = {
      ...process.env,
      RAYON_NUM_THREADS: undefined,
      GOMAXPROCS: undefined,
      RUNNER_ENVIRONMENT: runnerEnvironment,
      GITHUB_ENV: githubEnvPath,
    };
    execFileSync("bash", ["--noprofile", "--norc", "-e", "-o", "pipefail", scriptPath], {
      cwd: workdir,
      env,
      stdio: "pipe",
    });
    return {
      githubEnv: keyValues(readFileSync(githubEnvPath, "utf8")),
      baseline: keyValues(
        readFileSync(
          join(workdir, "target/vize-tests/metrics/check-fixtures-topology/runner-baseline.txt"),
          "utf8",
        ),
      ),
    };
  } finally {
    rmSync(workdir, { recursive: true, force: true });
  }
};
