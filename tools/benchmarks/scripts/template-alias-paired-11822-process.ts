import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncOptions, type SpawnSyncReturns } from "node:child_process";
import { closeSync, openSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export const campaignBudget = Object.freeze({
  hardMinutes: 60,
  softMinutes: 53,
  artifactReserveMinutes: 7,
});

export function remainingBudget(
  environment: NodeJS.ProcessEnv = process.env,
  now = Date.now(),
): number {
  const value = environment.TEMPLATE_ALIAS_DEADLINE_MS;
  assert.ok(value && /^\d+$/.test(value), "an explicit campaign soft deadline is required");
  const deadline = Number(value);
  assert.ok(
    Number.isSafeInteger(deadline) && deadline > now,
    "campaign soft deadline exhausted; retain raw receipts",
  );
  const start = environment.TEMPLATE_ALIAS_STARTED_MS;
  assert.ok(start && /^\d+$/.test(start), "an explicit campaign start is required");
  const started = Number(start);
  assert.ok(
    Number.isSafeInteger(started) &&
      started <= now &&
      deadline - started === campaignBudget.softMinutes * 60_000,
    "campaign deadline must retain the approved processing window",
  );
  return deadline - now;
}

interface StageOptions {
  directory: string;
  id: string;
  command: string;
  args: string[];
  cwd: string;
  environment: NodeJS.ProcessEnv;
  sourceSha: string;
  maxMilliseconds: number;
  run?: (command: string, args: string[], options: SpawnSyncOptions) => SpawnSyncReturns<Buffer>;
  budget?: () => number;
}

export function recordedStage(options: StageOptions): string {
  const { directory, id, command, args, cwd, environment, sourceSha } = options;
  const stdoutPath = join(directory, id + ".stdout");
  const stderrPath = join(directory, id + ".stderr");
  const processPath = join(directory, id + ".process.json");
  const stdout = openSync(stdoutPath, "wx"),
    stderr = openSync(stderrPath, "wx");
  const receipt = {
    command,
    args,
    cwd,
    sourceSha,
    started: new Date().toISOString(),
    ended: null as string | null,
    environmentKeys: Object.keys(environment).sort(),
    environment,
    stdoutPath,
    stderrPath,
    timeoutMilliseconds: 0,
    status: null as number | null,
    signal: null as NodeJS.Signals | null,
    timedOut: false,
    error: null as string | null,
  };
  const save = () => writeFileSync(processPath, JSON.stringify(receipt, null, 2) + "\n");
  save();
  try {
    receipt.timeoutMilliseconds = Math.min(
      options.maxMilliseconds,
      (options.budget ?? remainingBudget)(),
    );
    assert.ok(receipt.timeoutMilliseconds > 0);
    save();
    const run = options.run ?? ((command, args, settings) => spawnSync(command, args, settings));
    const result = run(
      "/usr/bin/timeout",
      [
        "--signal=TERM",
        "--kill-after=5s",
        `${receipt.timeoutMilliseconds / 1000}s`,
        command,
        ...args,
      ],
      {
        cwd,
        env: environment,
        stdio: ["ignore", stdout, stderr],
        timeout: receipt.timeoutMilliseconds + 15_000,
        killSignal: "SIGKILL",
      },
    );
    receipt.status = result.status;
    receipt.signal = result.signal;
    receipt.error = result.error ? String(result.error) : null;
    receipt.timedOut =
      result.status === 124 ||
      result.status === 137 ||
      result.signal === "SIGKILL" ||
      result.error?.message.includes("ETIMEDOUT") === true;
  } catch (error) {
    receipt.error = String(error);
  } finally {
    closeSync(stdout);
    closeSync(stderr);
    receipt.ended = new Date().toISOString();
    save();
  }
  assert.equal(receipt.error, null, `${id} failed; start/argv and original streams retained`);
  assert.equal(receipt.signal, null);
  assert.equal(receipt.status, 0, `${id} failed or timed out; original streams retained`);
  return readFileSync(stdoutPath, "utf8");
}

export function assertCampaignEnvironment(): void {
  assert.equal(process.platform, "linux", "performance campaign is Linux Actions only");
  assert.equal(process.arch, "x64");
  assert.equal(process.env.GITHUB_ACTIONS, "true", "performance campaign is Actions only");
  assert.equal(process.env.TEMPLATE_ALIAS_RUNNER, "blacksmith-32vcpu-ubuntu-2404");
  for (const key of [
    "TEMPLATE_ALIAS_CAPTURE",
    "VIZE_COLD_CAPTURE",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
    "NODE_PATH",
    "LD_PRELOAD",
    "NODE_OPTIONS",
    "VIZE_CHILD_ALLOCATION_OBSERVATION",
    "VIZE_ROUTE_MEMORY_OBSERVATION",
    "VIZE_OWNED_READ_BUFFER_TRACE",
  ]) {
    assert.equal(process.env[key], undefined, `${key} would instrument primary measurements`);
  }
}
