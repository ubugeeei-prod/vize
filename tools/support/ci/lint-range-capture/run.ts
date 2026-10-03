import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { closeSync, openSync } from "node:fs";
import { join } from "node:path";
import { constants } from "node:os";
import { isMain, cliArgs, directory, existingJson, save } from "./common.ts";
import { projects } from "./projects.ts";
import { verifyRuntime } from "./preflight.ts";

export const reporterArgs = [
  "tools/commands/fixtures/lint-divergence-report.rs",
  "--project",
  projects.map((p) => p.id).join(","),
  "--preset",
  "ecosystem",
  "--shard-index",
  "0",
  "--shard-count",
  "1",
  "--measure-coverage-gap",
  "--budget-mode",
  "enforce",
  "--vize-bin",
  "target/ci/vize",
  "--timeout-ms",
  "600000",
  "--output-dir",
  "eslint-invalid-range-capture/reports",
];
export function originalExit(record: { status: number | null; signal: NodeJS.Signals | null }) {
  if (record.status !== null) {
    assert.ok(Number.isInteger(record.status) && record.status >= 0 && record.status <= 255);
    return record.status;
  }
  return record.signal ? 128 + constants.signals[record.signal] : 1;
}
export async function capture(root: string, expected: string) {
  verifyRuntime(root, expected);
  const out = directory(root),
    stdout = openSync(join(out, "reporter.stdout.log"), "wx"),
    stderr = openSync(join(out, "reporter.stderr.log"), "wx");
  const startedAt = new Date().toISOString();
  const result = await new Promise<{
    status: number | null;
    signal: NodeJS.Signals | null;
    error: string | null;
  }>((resolve) => {
    const child = spawn("rust-script", reporterArgs, {
      cwd: root,
      env: process.env,
      stdio: ["ignore", stdout, stderr],
    });
    let error: string | null = null;
    child.on("error", (value) => {
      error = value.message;
    });
    child.on("close", (status, signal) =>
      resolve({ status: status !== null && status >= 0 ? status : null, signal, error }),
    );
  });
  closeSync(stdout);
  closeSync(stderr);
  save(root, "process-exit.json", {
    expectedSource: expected,
    executable: "rust-script",
    arguments: reporterArgs,
    startedAt,
    completedAt: new Date().toISOString(),
    ...result,
    acceptance: false,
  });
  return originalExit(result);
}
if (isMain(import.meta.url)) {
  const { mode, root, expected } = cliArgs();
  assert.ok(["run", "exit"].includes(mode));
  if (mode === "run") process.exitCode = await capture(root, expected);
  else {
    const record = existingJson(root, "process-exit.json");
    assert.equal(record?.expectedSource, expected, "No original producer exit record");
    process.exitCode = originalExit(record);
  }
}
