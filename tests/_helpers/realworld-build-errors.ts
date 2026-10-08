import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { sourceRevision } from "../differential/build-receipt.ts";
import { repoRoot } from "./realworld-patch.ts";

export type CompilerOutput = {
  code: string;
  css: string | null;
  errors: string[];
  filename: string;
  macro_artifacts: unknown[];
  script_lang: string;
  warnings: string[];
};

export type CompilerObservation = { output: CompilerOutput; outputText: string };

export type BuildResult = CompilerObservation & {
  files: string[];
  outputRoot: string;
  status: number | null;
  stderr: string;
  stdout: string;
};

export function observeCompilerResult(
  workspaceDir: string,
  appPath: string,
  cliBinary: string,
): CompilerObservation {
  const executable = path.join(
    path.dirname(path.resolve(repoRoot, cliBinary)),
    "examples",
    `cli_build_result_oracle${process.platform === "win32" ? ".exe" : ""}`,
  );
  assert.ok(fs.existsSync(executable), `build the SFC API oracle beside ${cliBinary}`);
  const input = {
    source: fs.readFileSync(path.join(workspaceDir, appPath), "utf8"),
    filename: path.basename(appPath),
    sourceId: appPath,
  };
  const result = spawnSync(executable, [], {
    cwd: workspaceDir,
    encoding: "utf8",
    input: JSON.stringify(input),
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 64 * 1024 * 1024,
    timeout: 120_000,
  });
  capture("sfc-api", {
    examplePath: path.relative(repoRoot, executable),
    exampleSha256: createHash("sha256").update(fs.readFileSync(executable)).digest("hex"),
    buildAuthority: "same existing CI CLI build step and its complete build log",
    input,
    status: result.status,
    signal: result.signal,
    stdout: result.stdout,
    stderr: result.stderr,
    processError: result.error?.message ?? null,
  });
  if (result.error != null) throw result.error;
  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.equal(result.stderr, "");
  return { output: JSON.parse(result.stdout) as CompilerOutput, outputText: result.stdout };
}

export function assertFailedBuild(result: BuildResult, appPath: string, errors: string[]): void {
  capture("cli-failure", { appPath, ...result });
  assert.equal(result.status, 1, result.stderr || result.stdout);
  assert.equal(result.stdout, "");
  assert.deepEqual(result.files, []);
  assert.equal(result.outputText, "");
  assert.deepEqual(result.output, {});
  assert.equal(fs.existsSync(result.outputRoot), false, "a failed build must emit no artifacts");
  // Elapsed time is variable; retain its exact four-decimal grammar and every other byte.
  const elapsed = /, 0 compiled in ([0-9]+\.[0-9]{4})s\n$/.exec(result.stderr);
  assert.ok(elapsed, result.stderr);
  const lines = errors
    .join("\n")
    .split("\n")
    .map((line) => `      ${line}\n`)
    .join("");
  assert.equal(
    result.stderr,
    `\n\x1b[31m✗ 1 error(s) occurred:\x1b[0m\n\n  \x1b[31mCompile errors (1):\x1b[0m\n    \x1b[1m${appPath}\x1b[0m\n${lines}\n\x1b[31m✗ 1 file(s) failed\x1b[0m, 0 compiled in ${elapsed[1]}s\n`,
  );
}

function capture(kind: string, observation: Record<string, unknown>): void {
  const root = path.join(repoRoot, "target/differential/cli-build-error-oracles");
  fs.mkdirSync(root, { recursive: true });
  const directory = fs.mkdtempSync(path.join(root, `${kind}-${process.pid}-`));
  fs.writeFileSync(
    path.join(directory, "observation.json"),
    `${JSON.stringify({ sourceRevision: sourceRevision(repoRoot), ...observation }, null, 2)}\n`,
  );
}
