import { spawnSync } from "node:child_process";

export function runVueTscBaseline({ vueTsc, args, cwd, timeoutMs, label }) {
  const startedAt = Date.now();
  const result = spawnSync(vueTsc.path, args, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 1024 * 1024 * 1024,
    timeout: timeoutMs,
  });
  const durationMs = Date.now() - startedAt;
  if (result.error != null) {
    throw new Error(`${label} failed to run: ${errorMessage(result.error)}`);
  }
  if (![0, 1, 2].includes(result.status)) {
    throw new Error(`${label} exited with unsupported status ${result.status}`);
  }
  const stdout = result.stdout ?? "";
  const stderr = result.stderr ?? "";
  return {
    durationMs,
    exitCode: result.status,
    output: `${stdout}\n${stderr}`,
    stderr,
    stdout,
  };
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}
