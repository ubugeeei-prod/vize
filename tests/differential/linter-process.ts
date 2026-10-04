import { spawnSync } from "node:child_process";
import { sha256 } from "./harness.mjs";

// Capture both fresh attempts before any classification. A thrown invocation
// records an honest empty stream/error and cannot suppress the second attempt.
export function collectLinterAttempts(
  binaryPath: string,
  argv: string[],
  input: Buffer,
  invoke = spawnSync,
) {
  const attempts = [];
  for (let pass = 0; pass < 2; pass++) {
    let stdout: Buffer = Buffer.alloc(0);
    let stderr: Buffer = Buffer.alloc(0);
    let exitStatus: number | null = null;
    let signal: string | null = null;
    let processError: string | null = null;
    try {
      const result = invoke(binaryPath, argv, {
        input,
        timeout: 30_000,
        maxBuffer: 8 * 1024 * 1024,
      });
      stdout = result.stdout ?? Buffer.alloc(0);
      stderr = result.stderr ?? Buffer.alloc(0);
      exitStatus = result.status;
      signal = result.signal;
      processError = result.error?.message ?? null;
    } catch (error) {
      processError = error instanceof Error ? error.message : String(error);
    }
    attempts.push({
      stdoutBase64: stdout.toString("base64"),
      stdoutSha256: sha256(stdout),
      stderrBase64: stderr.toString("base64"),
      stderrSha256: sha256(stderr),
      exitStatus,
      signal,
      processError,
    });
  }
  return attempts;
}
