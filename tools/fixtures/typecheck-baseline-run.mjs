import { runTypecheckCommand } from "./typecheck-command-runner.mjs";

export async function runVueTscBaseline({ vueTsc, args, cwd, timeoutMs, label }) {
  const startedAt = Date.now();
  const result = await runTypecheckCommand(vueTsc.path, args, {
    cwd,
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 1024 * 1024 * 1024,
    timeoutMs,
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
