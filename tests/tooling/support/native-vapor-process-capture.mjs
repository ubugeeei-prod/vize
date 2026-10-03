// Preserve raw child frames before parsing/asserting; no failure becomes a result.
import { appendFileSync } from "node:fs";
export function captureVaporProcess(label, inputs, result, expectedFailure = false) {
  const path = process.env.VIZE_NATIVE_VAPOR_PROCESS_CAPTURE;
  if (path)
    appendFileSync(
      path,
      JSON.stringify({
        label,
        expectedFailure,
        input: JSON.stringify(inputs),
        stdout: result.stdout ?? null,
        stderr: result.stderr ?? null,
        exit: result.status,
        signal: result.signal,
        error: result.error
          ? {
              name: result.error.name,
              message: result.error.message,
              stack: result.error.stack,
              code: result.error.code,
              errno: result.error.errno,
              syscall: result.error.syscall,
              spawnargs: result.error.spawnargs,
            }
          : null,
      }) + "\n",
    );
  return result;
}
