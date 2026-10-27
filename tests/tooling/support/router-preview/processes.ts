import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";

export type Observation = {
  label: string;
  command: string;
  args: string[];
  cwd: string;
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
  stdoutBase64: string;
  stderrBase64: string;
  elapsedMs: number;
};

export function processRecorder(output: string) {
  const observations: Observation[] = [];
  fs.mkdirSync(output, { recursive: true });
  const run = (label: string, command: string, args: string[], cwd: string, timeout = 120_000) => {
    const start = performance.now();
    const result = spawnSync(command, args, {
      cwd,
      timeout,
      maxBuffer: 64 * 1024 * 1024,
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
    });
    const stdout = result.stdout ?? Buffer.alloc(0);
    const stderr = result.stderr ?? Buffer.alloc(0);
    const observation: Observation = {
      label,
      command,
      args,
      cwd,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: stdout.toString("utf8"),
      stderr: stderr.toString("utf8"),
      stdoutBase64: stdout.toString("base64"),
      stderrBase64: stderr.toString("base64"),
      elapsedMs: performance.now() - start,
    };
    observations.push(observation);
    fs.writeFileSync(
      path.join(output, "processes.json"),
      `${JSON.stringify(observations, null, 2)}\n`,
    );
    return observation;
  };
  return { observations, run };
}
